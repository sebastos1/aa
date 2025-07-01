use rust_embed::RustEmbed;
use axum::{http::StatusCode, response::IntoResponse};
use reqwest::header;


#[derive(RustEmbed)]
#[folder = "web/build"]
struct Static;

pub async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    // Get the path from the request URI.
    let mut path = uri.path().trim_start_matches("/").to_string();

    // If the path is empty, they're asking for the root, so serve index.html.
    if path.is_empty() {
        path = "index.html".to_string();
    }

    // Try to get the requested file from our embedded assets.
    match Static::get(path.as_str()) {
        Some(content) => {
            // If the file is found, guess its MIME type and serve it.
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime_type.as_ref())], content.data).into_response()
        }
        None => {
            // If the file is NOT found (e.g., a deep link like /adv/some-slug),
            // we must serve the SPA's main index.html file.
            // SvelteKit's client-side router will then handle the route.
            if let Some(content) = Static::get("index.html") {
                (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], content.data).into_response()
            } else {
                // This will only happen if index.html is missing from your build folder.
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}
