//! Media upload contracts shared by the host-native tests and Worker glue.
//!
//! Express uses one `multer.fields()` instance for every media upload route.
//! This module keeps its field names/counts, 10 MiB per-file ceiling, MIME
//! allowlist, filename sanitising, and multipart validation independent of
//! Cloudflare bindings so the boundary stays directly testable on the host.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
pub const MAX_MULTIPART_BODY_SIZE: usize = 101 * 1024 * 1024;

pub const EVENT_IMAGES_FIELD: &str = "eventImages";
pub const EVENT_DOCUMENTS_FIELD: &str = "eventDocuments";
pub const VENUE_IMAGES_FIELD: &str = "venueImages";
pub const PROFILE_PICTURE_FIELD: &str = "profilePicture";

pub const ALLOWED_MIME_TYPES: [&str; 9] = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadFile {
    pub field_name: String,
    pub original_filename: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MultipartError {
    InvalidContentType,
    Malformed,
    UnexpectedField,
    TooManyFiles,
    FileTooLarge,
    InvalidMimeType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRow {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none", alias = "event_id")]
    pub event_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "venue_id")]
    pub venue_id: Option<i64>,
    #[serde(rename = "type")]
    pub media_type: String,
    #[serde(alias = "file_path")]
    pub file_path: String,
    #[serde(alias = "thumbnail_path")]
    pub thumbnail_path: Option<String>,
    #[serde(alias = "original_filename")]
    pub original_filename: String,
    #[serde(alias = "file_size")]
    pub file_size: i64,
    #[serde(alias = "mime_type")]
    pub mime_type: String,
    #[serde(alias = "uploaded_by")]
    pub uploaded_by: Option<String>,
    #[serde(alias = "created_at")]
    pub created_at: String,
}

pub fn is_allowed_mime(mime_type: &str) -> bool {
    ALLOWED_MIME_TYPES.contains(&mime_type)
}

pub fn is_image_mime(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg" | "image/png" | "image/webp" | "image/gif"
    )
}

fn max_count(field_name: &str) -> Option<usize> {
    match field_name {
        EVENT_IMAGES_FIELD => Some(10),
        EVENT_DOCUMENTS_FIELD => Some(5),
        VENUE_IMAGES_FIELD => Some(8),
        PROFILE_PICTURE_FIELD => Some(1),
        _ => None,
    }
}

fn boundary(content_type: &str) -> Option<&str> {
    if !content_type
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("multipart/form-data"))
    {
        return None;
    }

    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.trim().split_once('=')?;
        if !name.trim().eq_ignore_ascii_case("boundary") {
            return None;
        }
        let value = value.trim().trim_matches('"');
        (!value.is_empty()).then_some(value)
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    (!needle.is_empty())
        .then(|| {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        })
        .flatten()
}

fn disposition_parameter<'a>(header: &'a str, parameter: &str) -> Option<&'a str> {
    header.split(';').skip(1).find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        name.trim()
            .eq_ignore_ascii_case(parameter)
            .then(|| value.trim().trim_matches('"'))
    })
}

/// Parse the browser multipart shape used by `MediaUploader.tsx`.
///
/// Text fields are ignored, as they are by the media controllers. File fields
/// are validated with the same global Multer field declarations before an
/// individual route selects its own field.
pub fn parse_uploads(content_type: &str, body: &[u8]) -> Result<Vec<UploadFile>, MultipartError> {
    let boundary = boundary(content_type).ok_or(MultipartError::InvalidContentType)?;
    let marker = format!("--{boundary}").into_bytes();
    let mut cursor = 0;
    let mut uploads = Vec::new();
    let mut counts = Map::<String, Value>::new();

    while let Some(relative_start) = find_bytes(&body[cursor..], &marker) {
        let mut part_start = cursor + relative_start + marker.len();
        if body.get(part_start..part_start + 2) == Some(b"--") {
            break;
        }
        if body.get(part_start..part_start + 2) != Some(b"\r\n") {
            return Err(MultipartError::Malformed);
        }
        part_start += 2;

        let next_marker_prefix = format!("\r\n--{boundary}").into_bytes();
        let relative_end = find_bytes(&body[part_start..], &next_marker_prefix)
            .ok_or(MultipartError::Malformed)?;
        let part_end = part_start + relative_end;
        cursor = part_end + 2;

        let header_end = find_bytes(&body[part_start..part_end], b"\r\n\r\n")
            .ok_or(MultipartError::Malformed)?;
        let headers = std::str::from_utf8(&body[part_start..part_start + header_end])
            .map_err(|_| MultipartError::Malformed)?;
        let data_start = part_start + header_end + 4;

        let mut disposition = None;
        let mut mime_type = None;
        for line in headers.split("\r\n") {
            let Some((name, value)) = line.split_once(':') else {
                return Err(MultipartError::Malformed);
            };
            if name.eq_ignore_ascii_case("content-disposition") {
                disposition = Some(value.trim());
            } else if name.eq_ignore_ascii_case("content-type") {
                mime_type = Some(value.trim());
            }
        }

        let disposition = disposition.ok_or(MultipartError::Malformed)?;
        let Some(filename) = disposition_parameter(disposition, "filename") else {
            continue;
        };
        let field_name =
            disposition_parameter(disposition, "name").ok_or(MultipartError::Malformed)?;
        let max = max_count(field_name).ok_or(MultipartError::UnexpectedField)?;
        let mime_type = mime_type.ok_or(MultipartError::InvalidMimeType)?;
        if !is_allowed_mime(mime_type) {
            return Err(MultipartError::InvalidMimeType);
        }

        let bytes = &body[data_start..part_end];
        if bytes.len() > MAX_FILE_SIZE {
            return Err(MultipartError::FileTooLarge);
        }

        let count = counts.get(field_name).and_then(Value::as_u64).unwrap_or(0) as usize + 1;
        if count > max {
            return Err(MultipartError::TooManyFiles);
        }
        counts.insert(field_name.to_owned(), Value::from(count as u64));
        uploads.push(UploadFile {
            field_name: field_name.to_owned(),
            original_filename: filename.to_owned(),
            mime_type: mime_type.to_owned(),
            bytes: bytes.to_vec(),
        });
    }

    Ok(uploads)
}

fn basename(filename: &str) -> &str {
    filename.rsplit('/').next().unwrap_or(filename)
}

fn stem_and_extension(filename: &str) -> (&str, &str) {
    let filename = basename(filename);
    match filename.rfind('.') {
        Some(index) if index > 0 => (&filename[..index], &filename[index..]),
        _ => (filename, ""),
    }
}

pub fn safe_filename_stem(filename: &str) -> String {
    let (stem, _) = stem_and_extension(filename);
    stem.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

pub fn r2_key(filename: &str, category: &str, uuid: &str, suffix: Option<&str>) -> String {
    let (_, extension) = stem_and_extension(filename);
    let stem = safe_filename_stem(filename);
    match suffix {
        Some(suffix) => format!("{category}/{stem}-{uuid}-{suffix}{extension}"),
        None => format!("{category}/{stem}-{uuid}{extension}"),
    }
}

pub fn key_from_stored_path(path: &str) -> Option<String> {
    if let Some((_, remainder)) = path.split_once("://") {
        let (_, path) = remainder.split_once('/')?;
        let key = path.split(['?', '#']).next().unwrap_or(path);
        return (!key.is_empty()).then(|| key.to_owned());
    }

    let key = path.trim_start_matches('/');
    (!key.is_empty()).then(|| key.to_owned())
}

pub fn media_json(row: &MediaRow, expose_document_path: bool) -> Result<Value, serde_json::Error> {
    let mut value = serde_json::to_value(row)?;
    let object = value
        .as_object_mut()
        .expect("MediaRow serializes as an object");
    if row.media_type == "document" && !expose_document_path {
        object.insert("filePath".to_owned(), Value::Null);
    }
    let thumbnail = match row.thumbnail_path.as_deref() {
        Some(raw) => serde_json::from_str(raw)?,
        None => Value::Null,
    };
    object.insert("thumbnailPath".to_owned(), thumbnail);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn multipart(parts: &[(&str, &str, &str, &[u8])]) -> Vec<u8> {
        let mut body = Vec::new();
        for (field, filename, mime, bytes) in parts {
            body.extend_from_slice(b"--contract-boundary\r\n");
            body.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"{field}\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n"
                )
                .as_bytes(),
            );
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(b"--contract-boundary--\r\n");
        body
    }

    #[test]
    fn multipart_keeps_binary_file_fields() {
        let body = multipart(&[
            (
                EVENT_IMAGES_FIELD,
                "Party Photo.PNG",
                "image/png",
                b"\0\r\nimage",
            ),
            (
                EVENT_DOCUMENTS_FIELD,
                "brief.pdf",
                "application/pdf",
                b"%PDF",
            ),
        ]);
        let uploads = parse_uploads("multipart/form-data; boundary=contract-boundary", &body)
            .expect("valid multipart");

        assert_eq!(uploads.len(), 2);
        assert_eq!(uploads[0].original_filename, "Party Photo.PNG");
        assert_eq!(uploads[0].bytes, b"\0\r\nimage");
        assert_eq!(uploads[1].mime_type, "application/pdf");
    }

    #[test]
    fn multipart_enforces_multer_limits_and_mimes() {
        let files: Vec<_> = (0..11)
            .map(|_| (EVENT_IMAGES_FIELD, "x.png", "image/png", b"x".as_slice()))
            .collect();
        assert_eq!(
            parse_uploads(
                "multipart/form-data; boundary=contract-boundary",
                &multipart(&files),
            ),
            Err(MultipartError::TooManyFiles)
        );

        assert_eq!(
            parse_uploads(
                "multipart/form-data; boundary=contract-boundary",
                &multipart(&[(EVENT_IMAGES_FIELD, "x.svg", "image/svg+xml", b"x")]),
            ),
            Err(MultipartError::InvalidMimeType)
        );

        let oversized = vec![0; MAX_FILE_SIZE + 1];
        assert_eq!(
            parse_uploads(
                "multipart/form-data; boundary=contract-boundary",
                &multipart(&[(
                    EVENT_DOCUMENTS_FIELD,
                    "x.pdf",
                    "application/pdf",
                    &oversized
                )]),
            ),
            Err(MultipartError::FileTooLarge)
        );
    }

    #[test]
    fn keys_match_node_sanitising_and_url_extraction() {
        assert_eq!(
            r2_key(
                "/tmp/My Event (Final).PNG",
                "events",
                "00000000-0000-4000-8000-000000000000",
                Some("original"),
            ),
            "events/my-event--final--00000000-0000-4000-8000-000000000000-original.PNG"
        );
        assert_eq!(
            key_from_stored_path("https://media.hesocial.com/events/photo-id.png"),
            Some("events/photo-id.png".to_owned())
        );
        assert_eq!(
            key_from_stored_path("events/private-id.pdf"),
            Some("events/private-id.pdf".to_owned())
        );
    }

    #[test]
    fn list_shape_hides_documents_and_parses_thumbnail_json() {
        let row = MediaRow {
            id: "m1".to_owned(),
            event_id: Some(2),
            venue_id: None,
            media_type: "document".to_owned(),
            file_path: "events/private.pdf".to_owned(),
            thumbnail_path: Some("{\"thumb\":\"https://media/thumb\"}".to_owned()),
            original_filename: "private.pdf".to_owned(),
            file_size: 4,
            mime_type: "application/pdf".to_owned(),
            uploaded_by: Some("u1".to_owned()),
            created_at: "2026-08-31T00:00:00.000Z".to_owned(),
        };
        let value = media_json(&row, false).expect("valid row");
        assert_eq!(value["filePath"], Value::Null);
        assert_eq!(value["thumbnailPath"]["thumb"], "https://media/thumb");
    }
}
