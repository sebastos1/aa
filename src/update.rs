use anyhow::Result;
use crate::structs::Player;
use std::{path::Path, time::Duration};
use crate::{load::{self}, PlayerUpdateEvent, SharedState};

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
    
    println!("[UPDATE] Processed full update for player {}", uuid);
    Ok(())
}