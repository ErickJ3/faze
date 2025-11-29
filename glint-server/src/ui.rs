use axum::{
    body::Body,
    http::{HeaderValue, Response, StatusCode, Uri, header},
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use mime_guess::MimeGuess;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "../ui/dist"]
struct UiAssets;

pub fn fallback_service() -> MethodRouter {
    get(serve_embedded_asset)
}

async fn serve_embedded_asset(uri: Uri) -> impl IntoResponse {
    match resolve_response(uri.path(), true) {
        Some(response) => response,
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

fn resolve_response(path: &str, include_body: bool) -> Option<Response<Body>> {
    resolve_asset(path).map(|(asset_path, content)| {
        let mime = guess_mime(&asset_path).first_or_octet_stream();

        let builder = Response::builder().status(StatusCode::OK).header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref()).unwrap(),
        );

        if include_body {
            builder.body(Body::from(content.into_owned())).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
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
