use serde::Serialize;
use std::collections::HashMap;
use crate::structs::AdvancementProgress;

mod update;
pub use update::*;

use crate::structs::Player;

#[derive(Clone, Serialize)]
pub enum UpdateEvent {
    // for frontend
    ProgressUpdate {
        uuid: String,
        player: Player, // stats
        updated_progress: HashMap<String, AdvancementProgress>
    },
    // ProfileUpdate {
    //     uuid: String,
    //     name: String,
    //     avatar_url: String,
    // },

    // // for backend
    // ProfileAdded {
    //     uuid: String,
    // }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateEvent {
    pub uuid: String,
    pub name: String,
    pub avatar_url: String,
}