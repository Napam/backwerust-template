use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/static/"]
struct StaticAssets;

const CACHE_CONTROL: &str = if cfg!(debug_assertions) {
    "no-store"
} else {
    "public, max-age=0, must-revalidate"
};

pub async fn serve(Path(path): Path<String>, headers: HeaderMap) -> Response {
    let Some(file) = StaticAssets::get(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = etag(&file.metadata.sha256_hash());
    let not_modified = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == etag));

    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let content_type = mime_guess::from_path(&path).first_or_octet_stream();
        let mut response = Response::new(Body::from(file.data.into_owned()));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type.as_ref())
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        );
        response
    };

    let response_headers = response.headers_mut();
    response_headers.insert(
        header::ETAG,
        HeaderValue::from_str(&etag).expect("etag is ascii"),
    );
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL),
    );
    response
}

/// First 16 hex characters of the file's SHA-256, quoted for use as an ETag.
fn etag(hash: &[u8]) -> String {
    let hex: String = hash
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("\"{hex}\"")
}

#[cfg(test)]
mod tests {
    use super::etag;

    #[test]
    fn etag_is_a_quoted_prefix_of_sha256() {
        let hash = [0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14];

        assert_eq!(etag(&hash), "\"e3b0c44298fc1c14\"");
        assert_eq!(etag(&hash).len(), 18);
    }
}
