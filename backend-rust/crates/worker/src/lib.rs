use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderName;
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::json;
use tower_service::Service;
use worker::send::SendFuture;
use worker::{Context, Env, HttpRequest, Result, event};

mod auth;
mod auth_handlers;
mod handlers;

const DEFAULT_CORS_ORIGIN: &str = "http://localhost:3000";
const EXTRA_CORS_ORIGINS: [&str; 2] = ["http://127.0.0.1:3000", "http://localhost:5000"];

const RATE_LIMITER_BINDING: &str = "RATE_LIMITER";
const RATE_LIMITER_DISABLED_ENV: &str = "AUTH_RATE_LIMIT_DISABLED";
const RATE_LIMITER_KEY_HEADER: HeaderName = HeaderName::from_static("cf-connecting-ip");
const RATE_LIMITER_UNKEYED_CLIENT: &str = "unknown";

#[derive(Clone)]
pub struct AppState {
    allowed_origins: Vec<String>,
    pub env: Env,
}

impl AppState {
    fn from_env(env: &Env) -> Self {
        let configured_origins = env
            .var("CORS_ORIGINS")
            .map(|value| value.to_string())
            .unwrap_or_else(|_| DEFAULT_CORS_ORIGIN.to_owned());
        let mut allowed_origins: Vec<String> = configured_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(str::to_owned)
            .collect();

        for origin in EXTRA_CORS_ORIGINS {
            if !allowed_origins.iter().any(|allowed| allowed == origin) {
                allowed_origins.push(origin.to_owned());
            }
        }

        Self {
            allowed_origins,
            env: env.clone(),
        }
    }
}

fn cors_headers(request_headers: &HeaderMap, state: &AppState) -> HeaderMap {
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET,POST,PUT,DELETE,PATCH,OPTIONS"),
    );
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type,Authorization,X-Requested-With"),
    );
    response_headers.insert(VARY, HeaderValue::from_static("Origin"));

    if let Some(origin) = request_headers
        .get(ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|origin| {
            state
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin)
        })
        .and_then(|origin| HeaderValue::from_str(origin).ok())
    {
        response_headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    }

    response_headers
}

async fn cors_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let cors = cors_headers(request.headers(), &state);
    let mut response = if request.method() == Method::OPTIONS {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NO_CONTENT;
        response
    } else {
        next.run(request).await
    };

    {
        let headers = response.headers_mut();
        for (key, value) in &cors {
            headers.insert(key, value.clone());
        }
    }

    response
}

fn rate_limiter_disabled(state: &AppState) -> bool {
    state
        .env
        .var(RATE_LIMITER_DISABLED_ENV)
        .is_ok_and(|value| value.to_string() == "true")
}

fn too_many_requests() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "success": false,
            "error": "Too many requests, please try again later."
        })),
    )
        .into_response()
}

async fn apply_auth_rate_limit(state: AppState, request: Request<Body>, next: Next) -> Response {
    if rate_limiter_disabled(&state) {
        return next.run(request).await;
    }

    let Ok(limiter) = state.env.rate_limiter(RATE_LIMITER_BINDING) else {
        return next.run(request).await;
    };

    let client = request
        .headers()
        .get(RATE_LIMITER_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| RATE_LIMITER_UNKEYED_CLIENT.to_owned(), str::to_owned);

    match limiter.limit(client).await {
        Ok(outcome) if !outcome.success => too_many_requests(),
        _ => next.run(request).await,
    }
}

async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    SendFuture::new(apply_auth_rate_limit(state, request, next)).await
}

fn auth_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        .route("/profile", get(auth_handlers::profile))
        .route("/refresh", post(auth_handlers::refresh))
        .route("/logout", post(auth_handlers::logout))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        ))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/health/status", get(handlers::health_status))
        .route("/api/events", get(handlers::list_events))
        .route("/api/categories", get(handlers::list_categories))
        .route("/api/venues", get(handlers::list_venues))
        .nest("/api/auth", auth_routes(&state))
        .fallback(handlers::fallback)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cors_middleware,
        ))
        .with_state(state)
}

#[event(fetch)]
async fn fetch(
    request: HttpRequest,
    env: Env,
    _context: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router(AppState::from_env(&env)).call(request).await?)
}
