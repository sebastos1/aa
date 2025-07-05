use anyhow::{Context, Result};
use std::collections::HashMap;
use crate::load::{archive::*, strip_mc_prefix};
use std::collections::{HashSet};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum LangType {
    Advancement,
    Biome,
    Block,
    Effect,
    Enchantment,
    Entity,
    Instrument,
    Item,
    Stat,
    StatType,
    TrimMaterial,
    TrimPattern,
}

#[derive(Debug, Clone)]
pub struct LangEntry {
    pub lang_type: LangType,
    pub display_name: String,
    pub shared_id: bool,
}

pub struct LoadingContext {
    pub lang_map: HashMap<String, LangEntry>,
    pub tag_map: HashMap<String, TagData>,
    pub recipe_map: HashMap<String, String>,
}

impl LoadingContext {
    pub fn from_archives(archives: &mut [Box<dyn Archive>]) -> Result<Self> {
        println!("[LOAD] Initializing loading context from {} archives...", archives.len());
        
        let mut lang_map = HashMap::new();
        let mut tag_map = HashMap::new();
        let mut recipe_map = HashMap::new();
        
        let categories = vec![
            FileCategory::Language,
            FileCategory::Tags,
            FileCategory::Recipe,
        ];
        
        // load in order
        for archive in archives {
            let relevant_files = archive.list_files(&categories)?;
            
            for file_path in relevant_files {
                let content = archive.read_file(&file_path)?;
                
                if FileCategory::Language.matches(&file_path) {
                    load_language_file(&mut lang_map, &file_path, &content, archive.name())?;
                } else if FileCategory::Tags.matches(&file_path) {
                    load_tag_from_content(&mut tag_map, &file_path, &content)?;
                } else if FileCategory::Recipe.matches(&file_path) {
                    load_recipe_file(&mut recipe_map, &file_path, &content)?;
                }
            }
        }
        
        println!("Loaded {} lang entries, {} recipes, {} tags", lang_map.len(), recipe_map.len(), tag_map.len());
        Ok(LoadingContext {
            lang_map,
            tag_map,
            recipe_map,
        })
    }
    
    pub fn translate(&self, key: &str) -> Option<String> {
        self.lang_map.get(key).map(|entry| entry.display_name.clone())
    }
}






// lang

fn parse_lang_key(key: &str) -> Option<(LangType, String)> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    
    let lang_type = match parts[0] {
        "advancements" => LangType::Advancement,
        "biome" => LangType::Biome,
        "block" => LangType::Block,
        "effect" => LangType::Effect,
        "enchantment" => LangType::Enchantment,
        "entity" => LangType::Entity,
        "instrument" => LangType::Instrument,
        "item" => LangType::Item,
        "stat" => LangType::Stat,
        "stat_type" => LangType::StatType,
        "trim_material" => LangType::TrimMaterial,
        "trim_pattern" => LangType::TrimPattern,
        _ => return None,
    };
    
    // For advancement keys, keep the original key format
    let id = if lang_type == LangType::Advancement {
        key.to_string()
    } else if parts.len() >= 3 {
        let mut item = parts[2..].join(".");
        if parts[1] != "minecraft" {
            item = format!("{}:{}", parts[1], item)
        }
        item
    } else {
        return None;
    };
    
    Some((lang_type, id))
}

fn load_language_file(
    lang_map: &mut HashMap<String, LangEntry>, 
    file_path: &str,
    content: &str,
    archive_name: &str
) -> Result<()> {
    println!("[LOAD] Reading language file from {}: {}", archive_name, file_path);
    let archive_lang_map: HashMap<String, String> = serde_json::from_str(content)?;
    
    let mut loaded_count = 0;
    
    for (key, display_name) in archive_lang_map {
        if let Some((lang_type, id)) = parse_lang_key(&key) {
            if let Some(existing) = lang_map.get_mut(&id) {
                if (lang_type == LangType::Item && existing.lang_type == LangType::Block) ||
                   (lang_type == LangType::Block && existing.lang_type == LangType::Item) {
                    existing.shared_id = true;
                    continue;
                }
            }
            
            let entry = LangEntry {
                lang_type,
                display_name,
                shared_id: false,
            };
            lang_map.insert(id, entry);
            loaded_count += 1;
        }
    }
    
    println!("[LOAD] Loaded {} relevant language entries from {} (total in map: {})", loaded_count, archive_name, lang_map.len());
    Ok(())
}

// RECIPES

fn load_recipe_file(
    recipe_map: &mut HashMap<String, String>,
    file_path: &str,
    content: &str
) -> Result<()> {
    if let Ok(recipe_json) = serde_json::from_str::<serde_json::Value>(content) {
        let recipe_id = extract_recipe_id(file_path);
        
        if let Some(item_id) = parse_recipe_result(&recipe_json, &recipe_id) {
            recipe_map.insert(recipe_id, item_id);
        }
    }
    Ok(())
}

/// get recipe id from path
fn extract_recipe_id(file_path: &str) -> String {
    // "data/minecraft/recipe/stone_sword.json" -> "minecraft:stone_sword"
    // "data/other/recipe/item.json" -> "other:item"
    
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() >= 4 && parts[0] == "data" && parts[2] == "recipe" {
        let namespace = parts[1];
        let recipe_name = parts[3]
            .strip_suffix(".json")
            .unwrap_or(parts[3]);
        
        format!("{}:{}", namespace, recipe_name)
    } else {
        file_path.strip_suffix(".json").unwrap_or(file_path).to_string()
    }
}

fn parse_recipe_result(recipe_json: &serde_json::Value, recipe_id: &str) -> Option<String> {
    if let Some(result) = recipe_json.get("result") {
        match result {
            serde_json::Value::String(item_id) => {
                return Some(strip_mc_prefix(item_id).to_string());
            }
            serde_json::Value::Object(result_obj) => {
                if let Some(item_id) = result_obj.get("id").and_then(|v| v.as_str()) {
                    return Some(strip_mc_prefix(item_id).to_string());
                }
            }
            _ => {}
        }
    }
    
    if recipe_json.get("type").and_then(|v| v.as_str()) == Some("minecraft:smithing_trim") {
        if let Some(template) = recipe_json.get("template").and_then(|v| v.as_str()) {
            return Some(strip_mc_prefix(template).to_string());
        }
    }
    
    let recipe_name = recipe_id.split(':').last().unwrap_or(recipe_id);
    if recipe_name == "decorated_pot" || recipe_name == "tipped_arrow" {
        return Some(recipe_name.to_string());
    }
    
    None
}


// TAGS
#[derive(Deserialize, Debug)]
pub struct TagData {
    pub values: Vec<String>,
}

/// Load a single tag from file content and path into the tag map
pub fn load_tag_from_content(
    tag_map: &mut HashMap<String, TagData>, 
    file_path: &str, 
    content: &str
) -> Result<()> {
    let tag_data: TagData = serde_json::from_str(content).with_context(|| format!("Failed to parse tag JSON: {}", file_path))?;
    
    let tag_name = extract_tag_name(file_path)?;
    
    // Later archives override earlier ones (datapack behavior)
    tag_map.insert(tag_name, tag_data);
    
    Ok(())
}

pub fn tag_lookup(
    tag_map: &HashMap<String, TagData>, 
    tag_id: &str, 
    visited: &mut HashSet<String>
) -> Vec<String> {
    let mut id = tag_id;
    if tag_id.starts_with("#") {
        id = tag_id.strip_prefix("#").unwrap();
    }
    if visited.contains(id) {
        return Vec::new(); // Prevent circular references
    }
    visited.insert(id.to_string());
    
    let Some(tag_data) = tag_map.get(id) else {
        visited.remove(id);
        return Vec::new();
    };
    
    let mut result = Vec::new();
    for value in &tag_data.values {
        if value.starts_with('#') {
            // Recursive tag reference
            result.extend(tag_lookup(tag_map, &value, visited));
        } else {
            // Direct item ID
            result.push(strip_mc_prefix(value).to_string());
        }
    }
    
    visited.remove(id);
    result
}

fn extract_tag_name(file_path: &str) -> Result<String> {
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() >= 5 && parts[0] == "data" && parts[2] == "tags" {
        let namespace = parts[1];
        // Get the tag name (may be nested in subdirectories)
        let tag_path = parts[4..]
            .join("/")
            .strip_suffix(".json")
            .unwrap_or("")
            .to_string();
        
        // Store without category - just the natural tag name
        let full_tag_name = if namespace == "minecraft" {
            format!("minecraft:{}", tag_path)  // minecraft:beehives
        } else {
            format!("{}:{}", namespace, tag_path)  // mod:beehives
        };
        
        Ok(full_tag_name)
    } else {
        Err(anyhow::anyhow!("Invalid tag file path format: {}", file_path))
    }
}