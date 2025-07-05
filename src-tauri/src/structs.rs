
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use crate::load::Advancement;

#[derive(Debug, Serialize, Default)]
pub struct Data {
    pub world: crate::load::World,
    pub players: HashMap<String, Player>,
    pub advancements: HashMap<String, Advancement>,
    pub categories: HashMap<String, AdvancementCategory>,
    pub classes: Vec<String>, // from the spreadsheet

    // below this is subject to change after startup
    pub progress: HashMap<String, HashMap<String, AdvancementProgress>>, // advancement, playerid, player progress
}



#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub uuid: String,
    pub stats: PlayerStats,
    pub name: Option<String>, // online
    pub avatar_url: Option<String>, // online
}



#[derive(Debug, Clone, Serialize, Default)]
pub struct PlayerStats {
    pub stats: HashMap<String, HashMap<String, i64>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Icon {
    Item { 
        name: String, 
        #[serde(skip_serializing_if = "is_false", default)]
        shimmering: bool
    },
    PlayerHead { texture_id: String },
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AdvancementType {
    Root,
    Task,
    Goal,
    Challenge,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdvancementProgress {
    #[serde(alias = "criteria")]
    pub requirement_progress: HashMap<String, String>,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancementCategory {
    pub key: String,
    pub display_name: String,
    pub icon: Icon,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadsheetInfo {
    pub class: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub requirement_details: Option<String>,
}