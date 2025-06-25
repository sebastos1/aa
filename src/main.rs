mod structs;
mod load;
mod update;
mod criteria;
mod cache;
mod outbound;

use anyhow::Result;
use futures::Stream;
use serde::Serialize;
use tower_http::services::ServeDir;
use notify::{RecursiveMode, Watcher};
use tokio::sync::{broadcast, mpsc, RwLock};
use structs::{Data, Player};
use axum::{extract::State, response::{sse::{Event, Sse}, Json}, routing::get, Router};
use std::{collections::{HashSet}, path::{Path, PathBuf}, sync::Arc, time::Duration};

use crate::cache::Cache;

// const WORLD_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\New World (1)";
const WORLD_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\New World";
const SERVER_ADDR: &str = "127.0.0.1:3000";

pub struct AppState {
    data: Data,
    update_tx: broadcast::Sender<PlayerUpdateEvent>,
    processing_uuids: HashSet<String>,
    _cache: cache::Cache,
}
type SharedState = Arc<RwLock<AppState>>;

#[derive(Serialize, Clone)]
struct PlayerUpdateEvent {
    uuid: String,
    player: Player,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("cash");
    let cache = Cache::new().await?;

    println!("[1/4] Loading initial world data...");
    let data = load::load(WORLD_PATH, &cache).await?;

    println!("[2/4] Initializing application state...");
    let (update_tx, _) = broadcast::channel(32);
    let state = Arc::new(RwLock::new(AppState {
        data,
        update_tx: update_tx.clone(),
        processing_uuids: HashSet::new(),
        _cache: cache,
    }));

    println!("[3/4] Starting background file watcher...");
    file_watcher(state.clone())?;

    println!("[4/4] Starting web server...");
    let app = Router::new()
        .route("/api/init", get(init))
        .route("/api/events", get(event))
        .nest_service(format!("/{}", cache::CACHE_DIR).as_str(), ServeDir::new(cache::CACHE_DIR))
        .fallback_service(ServeDir::new("web/build"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await?;
    println!("-> Server listening on http://{}", SERVER_ADDR);
    axum::serve(listener, app).await?;
    Ok(())
}

// --- API Handlers ---
async fn init(State(state): State<SharedState>) -> Json<Data> {
    let app = state.read().await;

    Json(Data {
        world: app.data.world.clone(),
        advancements: app.data.advancements.clone(),
        players: app.data.players.clone(),
        categories: app.data.categories.clone(),
        classes: app.data.classes.clone(),
    })
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

fn file_watcher(state: SharedState) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<PathBuf>(100);

    tokio::spawn(async move {
        while let Some(path) = rx.recv().await {
            if let Some(uuid_str) = path.file_stem().and_then(|s| s.to_str()) {
                let uuid = uuid_str.to_string();

                // debounce
                let mut app_write = state.write().await;
                if !app_write.processing_uuids.insert(uuid.clone()) { continue }

                let uuid_clone = uuid.clone();
                let update_state = state.clone();
                let path_clone = path.clone();
                tokio::spawn({
                    let update_state = update_state.clone();
                    async move {
                        if let Err(e) = update::handle_player_update(update_state.clone(), &path_clone, &uuid).await {
                            eprintln!("[ERROR] Failed to process update for {:?}: {}", path_clone, e);
                        }

                        let mut app_write = update_state.write().await;
                        app_write.processing_uuids.remove(&uuid_clone);
                    }
                });
            }
        }
    });

    std::thread::spawn(move || {
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
            if let Ok(event) = res {
                if matches!(event.kind, notify::EventKind::Modify(_) | notify::EventKind::Create(_)) {
                    for path in event.paths {
                        if tx.blocking_send(path).is_err() { break; }
                    }
                }
            }
        }).expect("Failed to create file watcher");

        let advancements_path = Path::new(WORLD_PATH).join("advancements");
        watcher.watch(&advancements_path, RecursiveMode::NonRecursive).expect("Failed to start watching advancements directory");
        
        println!("[WATCHER] Now watching for changes in: {:?}", advancements_path);
        loop { std::thread::sleep(Duration::from_secs(60)); }
    });

    Ok(())
}