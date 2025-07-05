use super::*;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::Serialize;
use std::{collections::HashMap, fs, io::Read, path::Path};
use crate::structs::*;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub name: String,
    pub version: String,
    pub icon_path: Option<String>,

    #[serde(skip)]
    pub enabled_datapacks: Vec<String>,
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read JSON file at {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("Invalid JSON in file {}", path.display()))
}

pub fn read(world_path: &Path) -> Result<World> {
    let level_path = world_path.join("level.dat");
    let mut file = fs::File::open(&level_path)
        .with_context(|| format!("Failed to open level.dat at {}", level_path.display()))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let decompressed_data = if buffer.len() >= 2 && buffer[0] == 0x1f && buffer[1] == 0x8b {
        let mut decoder = GzDecoder::new(&buffer[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        decompressed
    } else {
        buffer
    };

    let mut cursor = std::io::Cursor::new(decompressed_data);
    let nbt_data = crab_nbt::Nbt::read(&mut cursor)?;

    let data = nbt_data
        .root_tag
        .get_compound("Data")
        .with_context(|| "Missing Data compound")?;

    let name = data
        .get("LevelName")
        .and_then(|tag| {
            if let crab_nbt::NbtTag::String(s) = tag {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "World".to_string());

    let version = data
        .get_compound("Version")
        .and_then(|ver| ver.get("Name"))
        .and_then(|tag| {
            if let crab_nbt::NbtTag::String(s) = tag {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());

        let enabled_datapacks = data
            .get_compound("DataPacks")
            .and_then(|datapacks| datapacks.get_list("Enabled"))
            .map(|list| {
                let file_datapacks: Vec<String> = list.iter()
                    .filter_map(|tag| {
                        if let crab_nbt::NbtTag::String(s) = tag {
                            // Only keep file/ datapacks (custom ones)
                            if s.starts_with("file/") && !s.ends_with(".zip") {
                                Some(s.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                // Handle duplicates - keep only the latest occurrence of each name
                let mut seen_names = std::collections::HashMap::new();
                let mut filtered = Vec::new();
                
                // Process in reverse to find the latest occurrence first
                for datapack in file_datapacks.into_iter().rev() {
                    let name_for_dedup = datapack
                        .strip_prefix("file/")
                        .unwrap()
                        .strip_suffix(".zip")
                        .unwrap_or(datapack.strip_prefix("file/").unwrap());
                        
                    if !seen_names.contains_key(name_for_dedup) {
                        seen_names.insert(name_for_dedup.to_string(), ());
                        // Remove file/ prefix but keep .zip if present
                        let cleaned = datapack.strip_prefix("file/").unwrap().to_string();
                        filtered.push(cleaned);
                    }
                }
                
                // Restore original order (latest wins)
                filtered.reverse();
                filtered
            })
            .unwrap_or_default();

    let icon_path = crate::cache::cache_world_icon(world_path)?;

    println!("{:?}", enabled_datapacks);

    Ok(World {
        name,
        version,
        icon_path,
        enabled_datapacks
    })
}

pub fn read_players(world_path: &Path) -> Result<(HashMap<String, Player>, HashMap<String, HashMap<String, AdvancementProgress>>)> {
    let mut players = HashMap::new();
    let mut progress: HashMap<String, HashMap<String, AdvancementProgress>> = HashMap::new();
    let advancements_dir = world_path.join("advancements");
    let stats_dir = world_path.join("stats");

    if !stats_dir.exists() {
        return Ok((players, progress));
    }

    // entries are named player uuid
    for stat_entry in fs::read_dir(&stats_dir)? {
        let stat_path = stat_entry?.path();
        if !stat_path.is_file() || stat_path.extension().map_or(true, |ext| ext != "json") {
            continue;
        }

        // filenames are the uuid
        let uuid = stat_path.file_stem().unwrap().to_str().unwrap().to_string();
        let stats = read_player_stats(&stat_path)?;

        players.insert(
            uuid.clone(),
            Player {
                uuid: uuid.clone(),
                stats,
                name: None, avatar_url: None,
            },
        );

        if advancements_dir.exists() {
            let advancements_path = advancements_dir.join(format!("{}.json", uuid));
            let player_progress = read_player_advancement_progress(&advancements_path)?;
            for (adv_key, prog_details) in player_progress {
                progress
                    .entry(adv_key)
                    .or_insert_with(HashMap::new)
                    .insert(uuid.clone(), prog_details);
            }
        }
    }

    Ok((players, progress))
}


pub fn read_player_advancement_progress(path: &Path) -> Result<HashMap<String, AdvancementProgress>> {
    let json = match read_json(path) {
        Ok(json) => json,
        Err(_) => return Ok(HashMap::new()), // If file doesn't exist, return empty map
    };
    let mut progress_map = HashMap::new();

    /*
        {
            "blazeandcave:animal/foal_play": {
                "criteria": {
                    "horse": "2025-06-19 23:36:15 +0200"
                },
                "done": true
            },
        }
    */
    if let Some(object) = json.as_object() {
        for (key, value) in object {
            // we don't care about recipes
            if key == "DataVersion" || key.contains("minecraft:recipes") {
                continue;
            }
    
            if let Ok(mut progress) = serde_json::from_value::<AdvancementProgress>(value.clone()) {

                progress.requirement_progress = progress
                    .requirement_progress
                    .into_iter()
                    .map(|(req_key, date)| (strip_mc_prefix(&req_key).to_string(), date))
                    .collect();

                // remove minecraft:, keep any other prefixes
                let key = strip_mc_prefix(key);
                progress_map.insert(key.to_string(), progress);
            }
        }
    }
    
    Ok(progress_map)
}

pub fn read_player_stats(path: &Path) -> Result<PlayerStats> {
    let json = match read_json(path) {
        Ok(json) => json,
        Err(_) => return Ok(PlayerStats::default()),
    };
    let mut stats = HashMap::new();

    /*
        {
            "stats": {
                "minecraft:used": {
                    "minecraft:end_stone": 388,
            ...
    */
    if let Some(categories) = json.get("stats").and_then(|stats| stats.as_object()) {
        for (category, stats_map) in categories {
            let mut category_stats = HashMap::new();
            if let Some(stats_map) = stats_map.as_object() {
                for (stat_key, stat_value) in stats_map {
                    category_stats.insert(strip_mc_prefix(stat_key).to_string(), stat_value.as_i64().unwrap_or(0));
                }
            }
            stats.insert(strip_mc_prefix(category).to_string(), category_stats);
        }
    }
    Ok(PlayerStats { stats: stats })
}