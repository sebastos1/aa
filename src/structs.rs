
use serde::{Deserialize, Serialize};
use std::{collections::{HashMap}};


#[derive(Debug, Serialize)]
pub struct Data {
    pub world: World,
    pub players: HashMap<String, Player>,
    pub advancements: HashMap<String, Advancement>,
    pub categories: HashMap<String, AdvancementCategory>,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Player {
    pub uuid: String,
    pub advancement_progress: HashMap<String, AdvancementProgress>,
    pub stats: PlayerStats,

    // online
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PlayerStats {
    pub stats: HashMap<String, HashMap<String, i64>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advancement {
    pub key: String,
    pub icon: Icon,
    pub advancement_type: AdvancementType,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parent: Option<String>,
    pub category: String,
    
    pub requirements: HashMap<String, Vec<crate::criteria::Criterion>>,

    pub display_name: String,
    pub description: String,
    pub spreadsheet_info: SpreadsheetInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AdvancementProgress {
    pub criteria: HashMap<String, String>, // todo
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct World {
    pub name: String,
    pub version: String,
    pub icon_path: Option<String>,
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