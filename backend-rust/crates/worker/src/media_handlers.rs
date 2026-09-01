use std::collections::HashMap;

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::ApiEnvelope;
use hesocial_core::auth::{UserRow, new_uuid_v4};
use hesocial_core::media::{
    EVENT_DOCUMENTS_FIELD, EVENT_IMAGES_FIELD, MAX_MULTIPART_BODY_SIZE, MediaRow, UploadFile,
    VENUE_IMAGES_FIELD, is_image_mime, key_from_stored_path, media_json, parse_uploads, r2_key,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::HttpMetadata;
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{authenticate, internal_error};
use crate::db::{self, Val};

const MEDIA_BUCKET_BINDING: &str = "MEDIA";
const DEFAULT_PUBLIC_URL: &str = "https://media.hesocial.com";

const EVENT_MEDIA_SELECT: &str = "SELECT id, event_id, NULL AS venue_id, type, file_path, thumbnail_path, original_filename, file_size, mime_type, uploaded_by, created_at FROM event_media WHERE event_id = ?";
const VENUE_MEDIA_SELECT: &str = "SELECT id, NULL AS event_id, venue_id, type, file_path, thumbnail_path, original_filename, file_size, mime_type, uploaded_by, created_at FROM venue_media WHERE venue_id = ? ORDER BY created_at DESC";
const DELETE_MEDIA_SELECT: &str = "SELECT em.file_path, em.thumbnail_path, em.uploaded_by, e.organizer_id FROM event_media em JOIN events e ON em.event_id = e.id WHERE em.id = ? UNION ALL SELECT vm.file_path, vm.thumbnail_path, vm.uploaded_by, NULL AS organizer_id FROM venue_media vm WHERE vm.id = ? LIMIT 1";

#[derive(Deserialize)]
struct EventOwner {
    organizer_id: String,
}

#[derive(Deserialize)]
struct ExistingVenue {
    #[allow(dead_code)]
    id: i64,
}

#[derive(Deserialize)]
struct DeleteMediaRow {
    file_path: String,
    thumbnail_path: Option<String>,
    uploaded_by: Option<String>,
    organizer_id: Option<String>,
}

#[derive(Clone, Copy)]
enum MediaParent {
    Event,
    Venue,
}

impl MediaParent {
    fn category(self) -> &'static str {
        match self {
            Self::Event => "events",
            Self::Venue => "venues",
        }
    }
}

fn id_bind(id: &str) -> Value {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => Val::from_f64(number),
        None => Val::from_str(id),
    }
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

fn public_url(state: &AppState, key: &str) -> String {
    let root = state
        .env
        .var("R2_PUBLIC_URL")
        .map(|value| value.to_string())
        .unwrap_or_else(|_| DEFAULT_PUBLIC_URL.to_owned());
    format!("{}/{key}", root.trim_end_matches('/'))
}

async fn request_uploads(
    request: Request<Body>,
    failure_message: &str,
) -> Result<Vec<UploadFile>, Response> {
    let content_type = request
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, failure_message))?;
    let bytes = to_bytes(request.into_body(), MAX_MULTIPART_BODY_SIZE)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, failure_message))?;
    parse_uploads(&content_type, &bytes)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, failure_message))
}

async fn delete_keys(state: &AppState, keys: &[String]) {
    let Ok(bucket) = state.env.bucket(MEDIA_BUCKET_BINDING) else {
        return;
    };
    for key in keys {
        let _ = bucket.delete(key).await;
    }
}

async fn put_object(
    state: &AppState,
    key: &str,
    bytes: &[u8],
    mime_type: &str,
    original_filename: &str,
    category: &str,
    media_type: &str,
) -> Result<(), ()> {
    let bucket = state.env.bucket(MEDIA_BUCKET_BINDING).map_err(|_| ())?;
    let metadata = HashMap::from([
        ("originalName".to_owned(), original_filename.to_owned()),
        ("category".to_owned(), category.to_owned()),
        ("type".to_owned(), media_type.to_owned()),
    ]);
    bucket
        .put(key, bytes.to_vec())
        .http_metadata(HttpMetadata {
            content_type: Some(mime_type.to_owned()),
            ..HttpMetadata::default()
        })
        .custom_metadata(metadata)
        .execute()
        .await
        .map_err(|_| ())?;
    Ok(())
}

async fn upload_one(
    state: &AppState,
    parent: MediaParent,
    parent_id: &str,
    user: &UserRow,
    file: &UploadFile,
    image_suffixes: &[&str],
) -> Result<Value, ()> {
    let media_id = new_uuid_v4().map_err(|_| ())?;
    let key_uuid = new_uuid_v4().map_err(|_| ())?;
    let category = parent.category();
    let is_image = !image_suffixes.is_empty();
    let media_type = if is_image { "image" } else { "document" };
    let original_key = r2_key(
        &file.original_filename,
        category,
        &key_uuid,
        is_image.then_some("original"),
    );
    put_object(
        state,
        &original_key,
        &file.bytes,
        &file.mime_type,
        &file.original_filename,
        category,
        if is_image { "original" } else { "document" },
    )
    .await?;

    let mut stored_keys = vec![original_key.clone()];
    let mut thumbnails = Map::new();

    // `sharp` cannot run inside a Worker isolate and workers-rs 0.8.5 has no
    // typed Images binding. Keep the existing independently deletable R2
    // variants and response contract, while storing the valid original bytes
    // in each variant. The roadmap records this pixel-dimension deviation.
    for suffix in image_suffixes {
        let variant_uuid = match new_uuid_v4() {
            Ok(uuid) => uuid,
            Err(_) => {
                delete_keys(state, &stored_keys).await;
                return Err(());
            }
        };
        let key = r2_key(
            &file.original_filename,
            category,
            &variant_uuid,
            Some(suffix),
        );
        if put_object(
            state,
            &key,
            &file.bytes,
            &file.mime_type,
            &file.original_filename,
            category,
            suffix,
        )
        .await
        .is_err()
        {
            delete_keys(state, &stored_keys).await;
            return Err(());
        }
        stored_keys.push(key.clone());
        thumbnails.insert((*suffix).to_owned(), json!(public_url(state, &key)));
    }

    let file_path = if is_image {
        public_url(state, &original_key)
    } else {
        original_key.clone()
    };
    let thumbnail_path = is_image.then(|| Value::Object(thumbnails.clone()).to_string());
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => {
            delete_keys(state, &stored_keys).await;
            return Err(());
        }
    };
    let statement = match parent {
        MediaParent::Event => {
            "INSERT INTO event_media (id, event_id, type, file_path, thumbnail_path, original_filename, file_size, mime_type, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        }
        MediaParent::Venue => {
            "INSERT INTO venue_media (id, venue_id, type, file_path, thumbnail_path, original_filename, file_size, mime_type, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        }
    };
    let binds = [
        Val::from_str(&media_id),
        id_bind(parent_id),
        Val::from_str(media_type),
        Val::from_str(&file_path),
        thumbnail_path.as_deref().map_or(db::NULL, Val::from_str),
        Val::from_str(&file.original_filename),
        Val::from_f64(file.bytes.len() as f64),
        Val::from_str(&file.mime_type),
        Val::from_str(&user.id),
    ];
    let inserted = db
        .prepare(statement)
        .bind(&binds)
        .map_err(|_| ())?
        .run()
        .await;
    if inserted.is_err() {
        delete_keys(state, &stored_keys).await;
        return Err(());
    }

    let mut result = Map::new();
    result.insert("id".to_owned(), json!(media_id));
    result.insert("type".to_owned(), json!(media_type));
    result.insert("filePath".to_owned(), json!(file_path));
    if is_image {
        result.insert("thumbnails".to_owned(), Value::Object(thumbnails));
    }
    result.insert("originalFilename".to_owned(), json!(file.original_filename));
    result.insert("fileSize".to_owned(), json!(file.bytes.len()));
    result.insert("mimeType".to_owned(), json!(file.mime_type));
    Ok(Value::Object(result))
}

async fn event_owner(state: &AppState, event_id: &str) -> Result<Option<EventOwner>, ()> {
    let db = db::Db::from_env(&state.env).map_err(|_| ())?;
    db.prepare("SELECT organizer_id FROM events WHERE id = ? AND status <> 'archived'")
        .bind(&[id_bind(event_id)])
        .map_err(|_| ())?
        .first(None)
        .await
        .map_err(|_| ())
}

async fn venue_exists(state: &AppState, venue_id: &str) -> Result<bool, ()> {
    let db = db::Db::from_env(&state.env).map_err(|_| ())?;
    let venue: Option<ExistingVenue> = db
        .prepare("SELECT id FROM venues WHERE id = ? AND is_active = 1")
        .bind(&[id_bind(venue_id)])
        .map_err(|_| ())?
        .first(None)
        .await
        .map_err(|_| ())?;
    Ok(venue.is_some())
}

pub async fn upload_event_images(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
    request: Request<Body>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(upload_event_images_inner(state, user, event_id, request)).await
}

async fn upload_event_images_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    request: Request<Body>,
) -> Response {
    let files = match request_uploads(request, "Failed to upload images").await {
        Ok(files) => files,
        Err(response) => return response,
    };
    let files: Vec<_> = files
        .iter()
        .filter(|file| file.field_name == EVENT_IMAGES_FIELD)
        .collect();
    if files.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "No images provided");
    }

    let owner = match event_owner(&state, &event_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Event not found"),
        Err(()) => return internal_error("Failed to upload images"),
    };
    if user.role != "super_admin" && user.role != "admin" && owner.organizer_id != user.id {
        return json_error(StatusCode::FORBIDDEN, "Permission denied");
    }

    let mut uploaded = Vec::new();
    for file in files {
        // Express accepts every allowlisted MIME in Multer, then Sharp skips
        // non-images inside the per-file catch.
        if !is_image_mime(&file.mime_type) {
            continue;
        }
        if let Ok(value) = upload_one(
            &state,
            MediaParent::Event,
            &event_id,
            &user,
            file,
            &["thumb", "medium"],
        )
        .await
        {
            uploaded.push(value);
        }
    }

    Json(json!({
        "success": true,
        "data": {
            "eventId": event_id,
            "uploadedImages": uploaded,
            "count": uploaded.len(),
        }
    }))
    .into_response()
}

pub async fn upload_event_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
    request: Request<Body>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(upload_event_documents_inner(state, user, event_id, request)).await
}

async fn upload_event_documents_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    request: Request<Body>,
) -> Response {
    let files = match request_uploads(request, "Failed to upload documents").await {
        Ok(files) => files,
        Err(response) => return response,
    };
    let files: Vec<_> = files
        .iter()
        .filter(|file| file.field_name == EVENT_DOCUMENTS_FIELD)
        .collect();
    if files.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "No documents provided");
    }

    let owner = match event_owner(&state, &event_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Event not found"),
        Err(()) => return internal_error("Failed to upload documents"),
    };
    if user.role != "super_admin" && user.role != "admin" && owner.organizer_id != user.id {
        return json_error(StatusCode::FORBIDDEN, "Permission denied");
    }

    let mut uploaded = Vec::new();
    for file in files {
        if let Ok(value) = upload_one(&state, MediaParent::Event, &event_id, &user, file, &[]).await
        {
            uploaded.push(value);
        }
    }

    Json(json!({
        "success": true,
        "data": {
            "eventId": event_id,
            "uploadedDocuments": uploaded,
            "count": uploaded.len(),
        }
    }))
    .into_response()
}

pub async fn upload_venue_images(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(venue_id): Path<String>,
    request: Request<Body>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if user.role != "admin" && user.role != "super_admin" {
        return json_error(StatusCode::FORBIDDEN, "Admin access required");
    }
    SendFuture::new(upload_venue_images_inner(state, user, venue_id, request)).await
}

async fn upload_venue_images_inner(
    state: AppState,
    user: UserRow,
    venue_id: String,
    request: Request<Body>,
) -> Response {
    let files = match request_uploads(request, "Failed to upload venue images").await {
        Ok(files) => files,
        Err(response) => return response,
    };
    let files: Vec<_> = files
        .iter()
        .filter(|file| file.field_name == VENUE_IMAGES_FIELD)
        .collect();
    if files.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "No images provided");
    }
    match venue_exists(&state, &venue_id).await {
        Ok(true) => {}
        Ok(false) => return json_error(StatusCode::NOT_FOUND, "Venue not found"),
        Err(()) => return internal_error("Failed to upload venue images"),
    }

    let mut uploaded = Vec::new();
    for file in files {
        if !is_image_mime(&file.mime_type) {
            continue;
        }
        if let Ok(value) = upload_one(
            &state,
            MediaParent::Venue,
            &venue_id,
            &user,
            file,
            &["thumb", "medium", "large"],
        )
        .await
        {
            uploaded.push(value);
        }
    }

    Json(json!({
        "success": true,
        "data": {
            "venueId": venue_id,
            "uploadedImages": uploaded,
            "count": uploaded.len(),
        }
    }))
    .into_response()
}

pub async fn get_event_media(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(get_event_media_inner(state, event_id, params)).await
}

async fn get_event_media_inner(
    state: AppState,
    event_id: String,
    params: HashMap<String, String>,
) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get event media"),
    };
    let mut statement = EVENT_MEDIA_SELECT.to_owned();
    let mut binds = vec![id_bind(&event_id)];
    if let Some(media_type) = params
        .get("type")
        .filter(|value| value.as_str() == "image" || value.as_str() == "document")
    {
        statement.push_str(" AND type = ?");
        binds.push(Val::from_str(media_type));
    }
    statement.push_str(" ORDER BY created_at DESC");
    let query = match db.prepare(statement).bind(&binds) {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to get event media"),
    };
    let rows = match query
        .all()
        .await
        .and_then(|result| result.results::<MediaRow>())
    {
        Ok(rows) => rows,
        Err(_) => return internal_error("Failed to get event media"),
    };
    let media: Result<Vec<_>, _> = rows
        .iter()
        // The Express route does not install optional-auth middleware, so its
        // document paths are always hidden even if an Authorization header is
        // present. Preserve that live behavior.
        .map(|row| media_json(row, false))
        .collect();
    match media {
        Ok(media) => Json(json!({ "success": true, "data": media })).into_response(),
        Err(_) => internal_error("Failed to get event media"),
    }
}

pub async fn get_venue_media(
    State(state): State<AppState>,
    Path(venue_id): Path<String>,
) -> Response {
    SendFuture::new(get_venue_media_inner(state, venue_id)).await
}

async fn get_venue_media_inner(state: AppState, venue_id: String) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get venue media"),
    };
    let query = match db.prepare(VENUE_MEDIA_SELECT).bind(&[id_bind(&venue_id)]) {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to get venue media"),
    };
    let rows = match query
        .all()
        .await
        .and_then(|result| result.results::<MediaRow>())
    {
        Ok(rows) => rows,
        Err(_) => return internal_error("Failed to get venue media"),
    };
    let media: Result<Vec<_>, _> = rows.iter().map(|row| media_json(row, true)).collect();
    match media {
        Ok(media) => Json(json!({ "success": true, "data": media })).into_response(),
        Err(_) => internal_error("Failed to get venue media"),
    }
}

pub async fn delete_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(media_id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(delete_media_inner(state, user, media_id)).await
}

async fn delete_media_inner(state: AppState, user: UserRow, media_id: String) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to delete media"),
    };
    let bind = Val::from_str(&media_id);
    let row: Option<DeleteMediaRow> = match db
        .prepare(DELETE_MEDIA_SELECT)
        .bind(&[bind.clone(), bind.clone()])
    {
        Ok(query) => match query.first(None).await {
            Ok(row) => row,
            Err(_) => return internal_error("Failed to delete media"),
        },
        Err(_) => return internal_error("Failed to delete media"),
    };
    let Some(row) = row else {
        return json_error(StatusCode::NOT_FOUND, "Media not found");
    };

    if user.role != "super_admin"
        && user.role != "admin"
        && row.organizer_id.as_deref() != Some(&user.id)
        && row.uploaded_by.as_deref() != Some(&user.id)
    {
        return json_error(StatusCode::FORBIDDEN, "Permission denied");
    }

    let bucket = match state.env.bucket(MEDIA_BUCKET_BINDING) {
        Ok(bucket) => bucket,
        Err(_) => return internal_error("Failed to delete media"),
    };
    let Some(main_key) = key_from_stored_path(&row.file_path) else {
        return internal_error("Failed to delete media");
    };
    if bucket.delete(&main_key).await.is_err() {
        return internal_error("Failed to delete media");
    }

    if let Some(raw) = row.thumbnail_path {
        if let Ok(Value::Object(thumbnails)) = serde_json::from_str::<Value>(&raw) {
            for path in thumbnails.values().filter_map(Value::as_str) {
                if let Some(key) = key_from_stored_path(path) {
                    // Express intentionally treats thumbnail cleanup as best
                    // effort after the primary object is gone.
                    let _ = bucket.delete(key).await;
                }
            }
        }
    }

    let event_delete = db
        .prepare("DELETE FROM event_media WHERE id = ?")
        .bind(std::slice::from_ref(&bind));
    let venue_delete = db
        .prepare("DELETE FROM venue_media WHERE id = ?")
        .bind(std::slice::from_ref(&bind));
    let (Ok(event_delete), Ok(venue_delete)) = (event_delete, venue_delete) else {
        return internal_error("Failed to delete media");
    };
    if event_delete.run().await.is_err() || venue_delete.run().await.is_err() {
        return internal_error("Failed to delete media");
    }

    Json(json!({
        "success": true,
        "message": "Media deleted successfully",
    }))
    .into_response()
}
