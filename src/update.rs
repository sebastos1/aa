use anyhow::Result;
use notify::{RecursiveMode, Watcher};
use tokio::sync::mpsc;
use crate::structs::Player;
use std::{path::{Path, PathBuf}, time::Duration};
use crate::{load::{self}, PlayerUpdateEvent, SharedState};

pub fn file_watcher(state: SharedState) -> Result<()> {
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
                        if let Err(e) = handle_player_update(update_state.clone(), &path_clone, &uuid).await {
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

        let advancements_path = Path::new(crate::WORLD_PATH).join("advancements");
        watcher.watch(&advancements_path, RecursiveMode::NonRecursive).expect("Failed to start watching advancements directory");
        
        println!("[WATCHER] Now watching for changes in: {:?}", advancements_path);
        loop { std::thread::sleep(Duration::from_secs(60)); }
    });

    Ok(())
}

pub async fn handle_player_update(state: SharedState, path: &Path, uuid: &str) -> Result<()> {
    // give the game time to finish up writing, should be plenty
    tokio::time::sleep(Duration::from_millis(150)).await;

    // get new advancement progress
    let new_progress = load::load_advancement_progress(path)?;
    
    // stats
    let stats_path = path.parent().unwrap().parent().unwrap()
        .join("stats").join(format!("{}.json", uuid));
    let new_stats = load::load_player_stats(&stats_path)?;

    let mut app = state.write().await;

    let player = {
        let player = app.data.players
        .entry(uuid.to_string())
        .or_insert_with(|| Player {
            uuid: uuid.to_string(),
            ..Default::default()
        });

        player.advancement_progress = new_progress;
        player.stats = new_stats;

        player.clone()
    };

    app.update_tx.send(PlayerUpdateEvent {
        uuid: uuid.to_string(),
        player,
    }).ok();

    let (etag, data_bytes) = crate::build_response_bytes(&app.data);
    app.etag = etag;
    app.data_bytes = data_bytes;
    
    println!("[UPDATE] Processed full update for player {}", uuid);
    Ok(())
}