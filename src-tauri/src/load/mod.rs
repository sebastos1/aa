pub mod requirements;
pub use requirements::*;
pub mod advancements;
pub use advancements::*;
pub mod world;
pub use world::*;
mod context;
use context::*;
pub mod archive;
use archive::*;

use anyhow::Result;
use serde::Deserialize;
use std::{collections::{HashMap, HashSet}, fs, path::Path};
use crate::{cache::Cache, structs::*};


const STRIP_MC_PREFIX: bool = true;
pub fn strip_mc_prefix(s: &str) -> &str {
    if !STRIP_MC_PREFIX { return s }
    s.strip_prefix("minecraft:").unwrap_or(s)
}

pub async fn load() -> Result<Data> {
    // these will be obtained somehow
    let minecraft_jar_path = Path::new("C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\meta\\versions\\1.21.5-21.5.75\\1.21.5-21.5.75.jar");
    let world_path = Path::new("C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\New World");

    let world = world::read(world_path)?;
    let (mut players, advancement_progress) = world::read_players(world_path)?;

    // grab user names and faces, tries to fetch them if we don't have them
    let cache = Cache::new().await?;
    let profile_task = {
        let cache = cache.clone();
        let player_uuids: Vec<String> = players.keys().cloned().collect();
        tokio::spawn(async move {
            for uuid in player_uuids {
                if let Some((_name, _avatar_url)) = cache.get_cached_or_fetch(&uuid).await {
                    // Cache automatically updates as each completes
                }
            }
        })
    };

    let mut advancements = load_all_advancements(minecraft_jar_path, world_path, &world.enabled_datapacks)?;

    let (spreadsheet_data, classes) = load_spreadsheet()?;
    assign_spreadsheet_info(&mut advancements, &spreadsheet_data);
    let categories = assign_categories(&mut advancements);

    for (uuid, player) in players.iter_mut() {
        if let Some(profile) = cache.get_player(uuid).await {
            player.name = Some(profile.name);
            player.avatar_url = Some(format!("data:image/png;base64,{}", profile.face));
        }
    }

    drop(profile_task);

    Ok(Data {
        world,
        players,
        advancements,
        categories,
        classes,
        progress: advancement_progress,
    })
}





fn load_spreadsheet() -> Result<(HashMap<String, SpreadsheetInfo>, Vec<String>)> {
    #[derive(Debug, Deserialize)]
    struct CsvRow {
        #[serde(rename = "Actual Name")]
        id: String,
        #[serde(rename = "Class")]
        class: String,
        #[serde(rename = "Actual Requirements (if different)")]
        actual_requirements: String,
    }

    let spreadsheet_path = "spreadsheet_list.csv";
    if !Path::new(spreadsheet_path).exists() {
        return Ok((HashMap::new(), vec!["Unknown".to_string()]));
    }

    let mut spreadsheet = HashMap::new();
    let mut classes = HashSet::new();

    let content = fs::read_to_string(spreadsheet_path)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    for result in reader.deserialize() {
        let row: CsvRow = result?;
        if row.id.is_empty() || row.class.is_empty() {
            continue;
        }

        let class = &row.class;
        classes.insert(class.to_string());

        let requirement_details = if row.actual_requirements.is_empty() {
            None
        } else {
            Some(row.actual_requirements)
        };

        spreadsheet.insert(
            row.id.to_string(),
            SpreadsheetInfo {
                class: class.to_string(),
                requirement_details: requirement_details,
            },
        );
    }

    Ok((spreadsheet, classes.into_iter().collect()))
}



fn assign_spreadsheet_info(advancements: &mut HashMap<String, Advancement>, spreadsheet_data: &HashMap<String, SpreadsheetInfo>) {
    for (id, info) in spreadsheet_data {
        if let Some(advancement) = advancements.get_mut(id) {
            advancement.spreadsheet_info = info.clone();
        }
    }
}

fn assign_categories(advancements: &mut HashMap<String, Advancement>) -> HashMap<String, AdvancementCategory> {
    let mut categories = HashMap::new();
    // whatever, its fast
    for (id, advancement) in advancements.iter() {
        if advancement.advancement_type == AdvancementType::Root {
            categories.insert(
                id.clone(),
                AdvancementCategory {
                    key: id.clone(),
                    display_name: advancement.display_name.clone(),
                    icon: advancement.icon.clone(),
                },
            );
        }
    }

    let all_ids: Vec<String> = advancements.keys().cloned().collect();
    for id in all_ids {
        let mut current_id_opt: Option<String> = Some(id.clone());
        let mut found_category_id: Option<String> = None;
        let mut visited = HashSet::new();

        while let Some(current_id) = current_id_opt {
            // this should never happen
            if !visited.insert(current_id.clone()) {
                eprintln!("[WARN] Advancement parent cycle detected starting at '{}'. Cannot assign category.", current_id);
                found_category_id = None;
                break;
            }

            if categories.contains_key(&current_id) {
                found_category_id = Some(current_id);
                break;
            }

            current_id_opt = advancements.get(&current_id).and_then(|adv| adv.parent.clone());
        }

        if let Some(advancement_to_update) = advancements.get_mut(&id) {
            if let Some(cat_id) = found_category_id {
                advancement_to_update.category = cat_id;
            } else {
                advancement_to_update.category = categories.keys().next().unwrap().clone();
            }
        }
    }

    categories
}
