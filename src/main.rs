mod structs;
mod load;
mod update;
mod requirements;
mod cache;
mod outbound;
mod embed;

use bytes;
use structs::Data;
use anyhow::Result;
use futures::Stream;
use reqwest::header;
use sha2::{Sha256, Digest};
use tower_http::services::ServeDir;
use tokio::sync::{broadcast,RwLock};
use axum::{extract::State, http::{HeaderMap, StatusCode}, response::{sse::{Event, Sse}, IntoResponse}, routing::get, Router};
use std::{collections::{HashSet}, sync::Arc};

use crate::cache::Cache;

const SERVER_ADDR: &str = "127.0.0.1:3000";

pub struct AppState {
    data: Data,
    etag: String, // hash for the data
    data_bytes: bytes::Bytes, // skip that expensive cloning
    
    update_tx: broadcast::Sender<crate::update::ProgressUpdateEvent>,
    processing_uuids: HashSet<String>,
    _cache: cache::Cache,
}
type SharedState = Arc<RwLock<AppState>>;

fn build_response_bytes(data: &Data) -> (String, bytes::Bytes) {
    let data_bytes = bytes::Bytes::from(serde_json::to_vec(&data).unwrap());
    let mut hasher = Sha256::new();
    hasher.update(&data_bytes);
    let etag = hex::encode(hasher.finalize());

    (etag, data_bytes)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("cash");
    let cache = Cache::new().await?;

    println!("[1/4] Loading initial world data...");
    let data = load::load(&cache).await?;

    let (etag, data_bytes) = build_response_bytes(&data);

    println!("[2/4] Initializing application state...");
    let (update_tx, _) = broadcast::channel(32);
    let state = Arc::new(RwLock::new(AppState {
        data, data_bytes, etag,
        update_tx: update_tx.clone(),
        processing_uuids: HashSet::new(),
        _cache: cache,
    }));

    println!("[3/4] Starting background file watcher...");
    update::file_watcher(state.clone())?;

    println!("[4/4] Starting web server...");
    let app = Router::new()
        .route("/api/init", get(init))
        .route("/api/events", get(event))
        .nest_service(format!("/{}", cache::CACHE_DIR).as_str(), ServeDir::new(cache::CACHE_DIR))
        .fallback(embed::static_handler) 
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await?;
    println!("-> Server listening on http://{}", SERVER_ADDR);
    axum::serve(listener, app).await?;
    Ok(())
}

// --- API Handlers ---
async fn init(State(state): State<SharedState>, headers: HeaderMap) -> impl IntoResponse {
    let etag = headers.get(header::IF_NONE_MATCH).and_then(|value| value.to_str().ok());
    let app = state.read().await;

    if etag == Some(app.etag.as_str()) {
        return StatusCode::NOT_MODIFIED.into_response();
    }

    (StatusCode::OK, [
        (header::CONTENT_TYPE, "application/json"),
        (header::CACHE_CONTROL, "public, max-age=0, must-revalidate"),
        (header::ETAG, &app.etag.clone()),
    ], app.data_bytes.clone()).into_response()
}

async fn event(State(state): State<SharedState>) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = state.read().await.update_tx.subscribe();
    let stream = async_stream::stream! {
        loop {
            if let Ok(update) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&update) {
                    yield Ok(Event::default().data(json));
                }
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}