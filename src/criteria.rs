use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use crate::load::strip_mc_prefix;

// requirements
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Criterion {
    Item(ItemReq),
    Entity(EntityReq),
    Location(LocationReq),
    Stat(StatReq),
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemReq {
    pub ids: Vec<String>, // there can be multiple options. Different types of logs for example
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityReq {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationReq {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub biomes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub structures: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatReq {
    pub id: String,
    pub goal: i64,
}



// trigger type exceptions
struct ReadConfig {
    items: bool,
    entities: bool,
    biomes: bool,
    stats: bool,
}

pub fn get_criteria(json: &Value) -> HashMap<String, Vec<Criterion>> {
    let mut criteria_map = HashMap::new();

    let Some(criteria) = json.get("criteria").and_then(|criteria| criteria.as_object()) else {
        return criteria_map;
    };
    
    for (key, criterion_json) in criteria {
        let Some(trigger) = criterion_json.get("trigger").and_then(|t| t.as_str()) else { continue };
        let Some(conditions) = criterion_json.get("conditions") else { continue };

        let config = ReadConfig {
            items: true,
            entities: true,
            biomes: true,
            stats: true,
        };

        match trigger {
            // "minecraft:player_interacted_with_entity" => config.items = false,
            // "minecraft:player_killed_entity" => config.items = false,
            // "minecraft:thrown_item_picked_up_by_player" => config.entities = false,
            _ => {}
        }

        let mut found_reqs = Vec::new();
        read_conditions(conditions, &mut found_reqs, &config);

        if !found_reqs.is_empty() {
            criteria_map.insert(strip_mc_prefix(key).to_string(), found_reqs);
        }
    }
    criteria_map
}

fn read_conditions(json: &Value, found: &mut Vec<Criterion>, config: &ReadConfig) {
    if let Some(obj) = json.as_object() {
        let mut parsed = false;

        if config.items && (obj.contains_key("items") || obj.contains_key("blocks"))  {
            if let Some(req) = get_item_reqs(json) {
                found.push(Criterion::Item(req));
                parsed = true;
            }
        }

        if config.entities && obj.contains_key("type") {
            if let Some(req) = get_entity_reqs(json) {
                found.push(Criterion::Entity(req)); 
                parsed = true;
            }
        }

        if config.biomes && (obj.contains_key("biomes") || obj.contains_key("structures")) {
            if let Some(req) = get_location_reqs(json) {
                found.push(Criterion::Location(req));
                parsed = true;
            }
        }

        if config.stats && obj.contains_key("stats") {
            if let Some(req) = get_stat_reqs(json) {
                found.push(Criterion::Stat(req));
                parsed = true;
            }
        }

        if parsed {
            return;
        }
        
        // If it's not a parsable requirement itself, recurse into its children.
        for (key, value) in obj {
            if key == "killing_blow" {
                continue;
            }

            read_conditions(value, found, config);
        }
    } else if let Some(arr) = json.as_array() {
        // If it's an array, recurse into each element.
        for item in arr {
            read_conditions(item, found, config);
        }
    }
}

fn get_item_reqs(item_json: &serde_json::Value) -> Option<ItemReq> {
    let mut item_ids = Vec::new();
    
    // "item": "minecraft:stone"
    if let Some(id_str) = item_json.get("item").and_then(|v| v.as_str()) {
        item_ids.push(strip_mc_prefix(id_str).to_string());
        
    // "items": ["minecraft:oak_log", "minecraft:birch_log"]
    } else if let Some(id_arr) = item_json.get("items").and_then(|v| v.as_array()) {
        for id_val in id_arr {
            if let Some(id_str) = id_val.as_str() {
                item_ids.push(strip_mc_prefix(id_str).to_string());
            }
        }
    }

    // blocks
    if let Some(id_str) = item_json.get("block").and_then(|v| v.as_str()) {
        item_ids.push(strip_mc_prefix(id_str).to_string());
    }
    if let Some(id_arr) = item_json.get("blocks").and_then(|v| v.as_array()) {
        for id_val in id_arr {
            if let Some(id_str) = id_val.as_str() { item_ids.push(strip_mc_prefix(id_str).to_string()); }
        }
    }

    if item_ids.is_empty() {
        return None
    }

    Some(ItemReq {
        ids: item_ids,
        amount: item_json.get("count").and_then(|c| c.as_i64()),
    })
}

fn get_entity_reqs(json: &serde_json::Value) -> Option<EntityReq> {
    let entity_type = json.get("type")?.as_str()?;

    // player checks grr
    if entity_type == "minecraft:player" || entity_type == "player" {
        return None
    }
    if let Some(type_specific) = json.get("type_specific") {
        if let Some(specific_type) = type_specific.get("type").and_then(|t| t.as_str()) {
            if specific_type == "player" {
                return None;
            }
        }
    }

    Some(EntityReq {
        id: strip_mc_prefix(entity_type).to_string(),
    })
}

fn get_location_reqs(json: &serde_json::Value) -> Option<LocationReq> {
    let mut biomes = Vec::new();
    let mut structures = Vec::new();
    if let Some(b) = json.get("biomes") {
        if let Some(s) = b.as_str() { biomes.push(strip_mc_prefix(s).to_string()); }
        else if let Some(arr) = b.as_array() {
            for item in arr { if let Some(s) = item.as_str() { biomes.push(strip_mc_prefix(s).to_string()); } }
        }
    }

    if let Some(s) = json.get("structures").and_then(|v| v.as_str()) {
        structures.push(strip_mc_prefix(s).to_string());
    }

    if biomes.is_empty() && structures.is_empty() {
        None 
    } else { 
        Some(LocationReq { biomes, structures })
    }
}

fn get_stat_reqs(json: &serde_json::Value) -> Option<StatReq> {
    let stats_array = json.get("stats")?.as_array()?;
    let first_stat = stats_array.get(0)?;
    
    let stat_id = first_stat.get("stat")?.as_str()?;
    let min_value = first_stat.get("value")?.get("min")?.as_i64()?;

    Some(StatReq {
        id: strip_mc_prefix(stat_id).to_string(),
        goal: min_value,
    })
}