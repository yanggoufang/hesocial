//! libSQL (Turso) data layer over the Hrana HTTP v2 pipeline endpoint.
//!
//! Mirrors the slice of the D1 API the handlers use — `prepare`/`bind`/`first`/
//! `all`/`run`/`batch` plus `changes`/`last_row_id` — so porting off the `DB`
//! binding stayed a bind-value change at the call sites rather than a rewrite.
//!
//! Two Hrana behaviours drive the shape here and were verified against a local
//! `turso dev` before this was written:
//!
//! 1. Values arrive tagged (`{"type":"integer","value":"1"}`) with integers as
//!    *strings*. [`decode_value`] untags them back into plain JSON so the
//!    existing `#[derive(Deserialize)]` row structs deserialize unchanged.
//! 2. A plain pipeline does not abort on a failed step — a following `COMMIT`
//!    still commits the steps that did succeed. Atomic [`Db::batch`]
//!    therefore uses the `batch` request type, guarding every step on its
//!    predecessor and trailing a `ROLLBACK` guarded on the `COMMIT` failing.

use std::rc::Rc;

use serde::de::DeserializeOwned;
use serde_json::{Map, Number, Value, json};
use worker::wasm_bindgen::JsValue;
use worker::{Env, Fetch, Headers, Method, RequestInit};

/// `[vars]` entry holding the database URL (`libsql://…`, or `http://…` for a
/// local `turso dev`).
pub const URL_VAR: &str = "TURSO_URL";
/// Secret holding the group auth token. Absent for a local `turso dev`, which
/// requires no authentication.
pub const AUTH_TOKEN_VAR: &str = "TURSO_AUTH_TOKEN";

/// Every failure mode here is "the query did not come back usable"; the
/// handlers all collapse it to their own 500 envelope, so it carries no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;

/// Largest integer an f64 round-trips exactly; past it, treat as float.
const MAX_EXACT_INT: f64 = 9_007_199_254_740_992.0;

/// SQL bind placeholder for a NULL, named to match the `JsValue::NULL` it replaced.
pub const NULL: Value = Value::Null;

/// Bind-value constructors mirroring the `JsValue` ones the D1 call sites used.
pub struct Val;

impl Val {
    pub fn from_str(value: &str) -> Value {
        Value::String(value.to_owned())
    }

    /// D1 bound JS numbers, which carry no int/float distinction — an integral
    /// one reaches SQLite as an INTEGER. Hrana makes us pick, so pick the same.
    pub fn from_f64(value: f64) -> Value {
        if value.fract() == 0.0 && value.abs() <= MAX_EXACT_INT {
            Value::Number(Number::from(value as i64))
        } else {
            Number::from_f64(value).map_or(Value::Null, Value::Number)
        }
    }

    pub fn from_bool(value: bool) -> Value {
        Value::Bool(value)
    }
}

struct Inner {
    endpoint: String,
    auth: Option<String>,
}

/// Handle to one libSQL database. Cheap to clone; every call is a fresh
/// stateless pipeline request, so there is no connection to pool.
#[derive(Clone)]
pub struct Db(Rc<Inner>);

impl Db {
    pub fn from_env(env: &Env) -> Result<Self, Error> {
        let url = env.var(URL_VAR).map_err(|_| Error)?.to_string();
        let auth = env
            .secret(AUTH_TOKEN_VAR)
            .ok()
            .map(|secret| secret.to_string())
            .filter(|token| !token.is_empty());
        Self::new(&url, auth)
    }

    pub fn new(url: &str, auth: Option<String>) -> Result<Self, Error> {
        let base = http_base(url).map_err(|()| Error)?;
        Ok(Self(Rc::new(Inner {
            endpoint: format!("{base}/v2/pipeline"),
            auth,
        })))
    }

    /// Generic over the query type to match the D1 signature the call sites
    /// were written against, which accepted an owned `format!` result.
    pub fn prepare(&self, sql: impl Into<String>) -> PreparedStatement {
        PreparedStatement {
            db: self.clone(),
            sql: sql.into(),
            args: Vec::new(),
        }
    }

    /// All-or-nothing execution, replacing D1's `batch()`. Any failing
    /// statement — or a failing `COMMIT` — rolls the whole set back and
    /// surfaces as `Err`.
    pub async fn batch(&self, statements: Vec<PreparedStatement>) -> Result<Vec<QueryResult>, ()> {
        let count = statements.len();
        let mut steps = vec![json!({ "stmt": { "sql": "BEGIN" } })];
        for (offset, statement) in statements.iter().enumerate() {
            steps.push(json!({
                "condition": { "type": "ok", "step": offset },
                "stmt": statement.stmt_json(),
            }));
        }
        let commit_step = count + 1;
        steps.push(json!({
            "condition": { "type": "ok", "step": count },
            "stmt": { "sql": "COMMIT" },
        }));
        steps.push(json!({
            "condition": { "type": "not", "cond": { "type": "ok", "step": commit_step } },
            "stmt": { "sql": "ROLLBACK" },
        }));

        let results = self
            .pipeline(json!([
                { "type": "batch", "batch": { "steps": steps } },
                { "type": "close" },
            ]))
            .await?;
        let batch = step_result(results.first().ok_or(())?)?;

        let errors = batch
            .get("step_errors")
            .and_then(Value::as_array)
            .ok_or(())?;
        // BEGIN, every statement, and COMMIT must all have succeeded; ROLLBACK
        // is expected to be skipped, so it is not checked.
        if errors
            .iter()
            .take(commit_step + 1)
            .any(|error| !error.is_null())
        {
            return Err(());
        }

        let step_results = batch
            .get("step_results")
            .and_then(Value::as_array)
            .ok_or(())?;
        step_results
            .iter()
            .skip(1)
            .take(count)
            .map(QueryResult::from_stmt_result)
            .collect()
    }

    /// POST one Hrana pipeline and hand back its `results` array.
    async fn pipeline(&self, requests: Value) -> Result<Vec<Value>, ()> {
        let headers = Headers::new();
        headers
            .set("Content-Type", "application/json")
            .map_err(|_| ())?;
        if let Some(token) = &self.0.auth {
            headers
                .set("Authorization", &format!("Bearer {token}"))
                .map_err(|_| ())?;
        }

        let body = json!({ "requests": requests }).to_string();
        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(JsValue::from_str(&body)));
        let request = worker::Request::new_with_init(&self.0.endpoint, &init).map_err(|_| ())?;
        let mut response = Fetch::Request(request).send().await.map_err(|_| ())?;
        if response.status_code() != 200 {
            return Err(());
        }
        let payload = response.json::<Value>().await.map_err(|_| ())?;
        payload
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .ok_or(())
    }
}

/// A SQL statement plus its bind values, awaiting execution.
pub struct PreparedStatement {
    db: Db,
    sql: String,
    args: Vec<Value>,
}

impl PreparedStatement {
    /// Fallible to match the D1 signature the call sites already handle.
    pub fn bind(mut self, values: &[Value]) -> Result<Self, ()> {
        self.args = values.to_vec();
        Ok(self)
    }

    pub async fn all(&self) -> Result<QueryResult, ()> {
        let results = self
            .db
            .pipeline(json!([
                { "type": "execute", "stmt": self.stmt_json() },
                { "type": "close" },
            ]))
            .await?;
        QueryResult::from_stmt_result(step_result(results.first().ok_or(())?)?)
    }

    pub async fn run(&self) -> Result<QueryResult, ()> {
        self.all().await
    }

    /// `column` picks a single column out of the first row; `None` deserializes
    /// the whole row, which is what every call site does.
    pub async fn first<T>(&self, column: Option<&str>) -> Result<Option<T>, ()>
    where
        T: DeserializeOwned,
    {
        self.all().await?.first_row(column)
    }

    fn stmt_json(&self) -> Value {
        let args: Vec<Value> = self.args.iter().map(encode_value).collect();
        json!({ "sql": self.sql, "args": args })
    }
}

/// Row metadata, mirroring D1's `meta()` payload.
pub struct ResultMeta {
    pub changes: usize,
    pub last_row_id: Option<i64>,
}

/// Rows plus metadata from one executed statement.
pub struct QueryResult {
    rows: Vec<Value>,
    changes: usize,
    last_row_id: Option<i64>,
}

impl QueryResult {
    pub fn results<T>(&self) -> Result<Vec<T>, ()>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(Value::Array(self.rows.clone())).map_err(|_| ())
    }

    pub fn meta(&self) -> ResultMeta {
        ResultMeta {
            changes: self.changes,
            last_row_id: self.last_row_id,
        }
    }

    fn first_row<T>(&self, column: Option<&str>) -> Result<Option<T>, ()>
    where
        T: DeserializeOwned,
    {
        let Some(row) = self.rows.first() else {
            return Ok(None);
        };
        let value = match column {
            Some(name) => row.get(name).cloned().ok_or(())?,
            None => row.clone(),
        };
        serde_json::from_value(value).map(Some).map_err(|_| ())
    }

    /// Build from Hrana's `StmtResult`: `cols` names zipped onto each row's
    /// positional, tagged values.
    fn from_stmt_result(result: &Value) -> Result<Self, ()> {
        let names: Vec<String> = result
            .get("cols")
            .and_then(Value::as_array)
            .ok_or(())?
            .iter()
            .map(|col| {
                col.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();

        let mut rows = Vec::new();
        for row in result.get("rows").and_then(Value::as_array).ok_or(())? {
            let cells = row.as_array().ok_or(())?;
            let mut object = Map::new();
            for (name, cell) in names.iter().zip(cells) {
                object.insert(name.clone(), decode_value(cell));
            }
            rows.push(Value::Object(object));
        }

        Ok(Self {
            rows,
            changes: result
                .get("affected_row_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
            last_row_id: result.get("last_insert_rowid").and_then(as_i64),
        })
    }
}

/// Unwrap one pipeline/batch step, turning Hrana's `{"type":"error"}` into `Err`.
fn step_result(step: &Value) -> Result<&Value, ()> {
    if step.get("type").and_then(Value::as_str) != Some("ok") {
        return Err(());
    }
    step.get("response")
        .and_then(|response| response.get("result"))
        .ok_or(())
}

/// Plain JSON bind value -> Hrana's tagged form.
fn encode_value(value: &Value) -> Value {
    match value {
        Value::Null => json!({ "type": "null" }),
        // SQLite has no boolean type; D1 bound JS `true` as 1.
        Value::Bool(flag) => json!({ "type": "integer", "value": i64::from(*flag).to_string() }),
        Value::Number(number) => match number.as_i64() {
            Some(integer) => json!({ "type": "integer", "value": integer.to_string() }),
            None => json!({ "type": "float", "value": number.as_f64().unwrap_or_default() }),
        },
        Value::String(text) => json!({ "type": "text", "value": text }),
        // No call site binds a composite; send it as text rather than silently drop it.
        other => json!({ "type": "text", "value": other.to_string() }),
    }
}

/// Hrana's tagged form -> plain JSON, so the row structs deserialize as they did
/// off D1. Integers arrive as strings and must be widened back to numbers.
fn decode_value(value: &Value) -> Value {
    match value.get("type").and_then(Value::as_str) {
        Some("integer") => value
            .get("value")
            .and_then(as_i64)
            .map_or(Value::Null, |integer| Value::Number(Number::from(integer))),
        Some("float") => value
            .get("value")
            .and_then(Value::as_f64)
            .and_then(Number::from_f64)
            .map_or(Value::Null, Value::Number),
        Some("text") => value
            .get("value")
            .and_then(Value::as_str)
            .map_or(Value::Null, |text| Value::String(text.to_owned())),
        // Blobs are base64 in Hrana; no column in this schema reads one back.
        Some("blob") => value
            .get("base64")
            .and_then(Value::as_str)
            .map_or(Value::Null, |text| Value::String(text.to_owned())),
        _ => Value::Null,
    }
}

/// Hrana writes 64-bit integers as strings to survive JSON; accept both.
fn as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

/// `libsql://` is the SDK scheme for what is an HTTPS endpoint; a local
/// `turso dev` is plain `http://`.
fn http_base(url: &str) -> Result<String, ()> {
    let trimmed = url.trim().trim_end_matches('/');
    if let Some(host) = trimmed.strip_prefix("libsql://") {
        return Ok(format!("https://{host}"));
    }
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        return Ok(trimmed.to_owned());
    }
    Err(())
}
