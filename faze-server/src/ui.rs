use axum::{
    body::Body,
    http::{HeaderValue, Response, StatusCode, Uri, header},
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use mime_guess::MimeGuess;
use rust_embed::RustEmbed;
use std::borrow::Cow;

const FALLBACK_MIME: HeaderValue = HeaderValue::from_static("application/octet-stream");

#[derive(RustEmbed)]
#[folder = "../ui/dist"]
struct UiAssets;

/// Build the Axum fallback service that serves the embedded UI bundle.
pub fn fallback_service() -> MethodRouter {
    get(serve_embedded_asset)
}

async fn serve_embedded_asset(uri: Uri) -> impl IntoResponse {
    resolve_response(uri.path(), true).unwrap_or_else(|| {
        let mut response = Response::new(Body::from("Not Found"));
        *response.status_mut() = StatusCode::NOT_FOUND;
        response
    })
}

fn resolve_response(path: &str, include_body: bool) -> Option<Response<Body>> {
    resolve_asset(path).map(|(asset_path, content)| {
        let mime = guess_mime(&asset_path).first_or_octet_stream();
        let mime_header =
            HeaderValue::from_str(mime.as_ref()).unwrap_or_else(|_| FALLBACK_MIME.clone());

        let body = if include_body {
            Body::from(content.into_owned())
        } else {
            Body::empty()
        };

        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::OK;
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, mime_header);
        response
    })
}

fn resolve_asset(path: &str) -> Option<(String, Cow<'static, [u8]>)> {
    let trimmed = path.trim_start_matches('/');

    if trimmed.contains("..") {
        return None;
    }

    let mut requested = if trimmed.is_empty() {
        "index.html".to_owned()
    } else {
        trimmed.to_owned()
    };

    if requested.ends_with('/') {
        requested.push_str("index.html");
    }

    if let Some(content) = UiAssets::get(&requested) {
        return Some((requested, content.data));
    }

    if !requested.contains('.')
        && let Some(content) = UiAssets::get("index.html")
    {
        return Some(("index.html".to_owned(), content.data));
    }

    None
}

fn guess_mime(asset_path: &str) -> MimeGuess {
    mime_guess::from_path(asset_path)
}
