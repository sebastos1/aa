use anyhow::{Context, Result};
use base64::Engine;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::{collections::HashMap, path::Path};
use crate::structs::*;
use crate::load::*;

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
    pub requirements: BTreeMap<String, Vec<Subject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_subjects: Option<Vec<Subject>>,
    pub display_name: String,
    pub description: String,
    pub spreadsheet_info: SpreadsheetInfo,
}

pub fn load_all_advancements(jar_path: &Path, world_path: &Path, enabled_datapacks: &Vec<String>) -> Result<HashMap<String, Advancement>> {

    requirements::init_debug();

    let mut archives: Vec<Box<dyn Archive>> = vec![
        archive_from_jar_path(jar_path)?
    ];

    for datapack_name in enabled_datapacks {
        let datapack_path = world_path.join("datapacks").join(datapack_name);
        if datapack_path.exists() {
            match open_archive(&datapack_path, datapack_name.clone()) {
                Ok(archive) => {
                    println!("[LOAD] Opened datapack: {}", datapack_name);
                    archives.push(archive);
                }
                Err(e) => {
                    println!("[WARN] Failed to open datapack {}: {:?}", datapack_name, e);
                }
            }
        } else {
            println!("[WARN] Datapack not found: {}", datapack_path.display());
        }
    }

    let context = LoadingContext::from_archives(&mut archives)?;

    let categories = vec![FileCategory::Advancement];
    let mut advancements = HashMap::new();
    for mut archive in archives {
        let advancement_files = archive.list_files(&categories)?;
            
        if !advancement_files.is_empty() {
            println!("[LOAD] Found {} advancement files in {}", 
                     advancement_files.len(), archive.name());
        }
        
        for file_path in advancement_files {
            let advancement_id = extract_advancement_id(&file_path);
            
            match load_advancement(&mut archive, &file_path, &advancement_id, &context) {
                Ok(advancement) => {
                    // Later archives override earlier ones (datapack override behavior)
                    advancements.insert(advancement_id.clone(), advancement);
                }
                Err(_) => {}
            }
        }
    }
    
    println!("[LOAD] Successfully loaded {} total advancements", advancements.len());

    requirements::write_debug_files();
    Ok(advancements)
}

fn load_advancement(
    archive: &mut Box<dyn Archive>,
    file_path: &str,
    advancement_id: &str,
    context: &LoadingContext
) -> Result<Advancement> {
    let content = archive.read_file(file_path).with_context(|| format!("Failed to read advancement file: {}", file_path))?;
    let json: AdvancementJson = serde_json::from_str(&content).with_context(|| format!("Failed to parse advancement JSON: {}", file_path))?;
    json_to_advancement(context, &json, advancement_id).with_context(|| format!("Failed to convert advancement: {}", advancement_id))
}

#[derive(Deserialize)]
struct AdvancementJson {
    display: Option<DisplayJson>,
    parent: Option<String>,
    criteria: HashMap<String, Criteria>,
    requirements: Option<Vec<Vec<String>>>,
}

#[derive(Deserialize)]
pub struct DisplayJson {
    pub title: Value,
    pub description: Value,
    pub icon: Value,
    pub frame: Option<String>,
}

fn json_to_advancement(context: &LoadingContext, json: &AdvancementJson, id: &str) -> Result<Advancement> {
    let Some(display) = json.display.as_ref() else {
        return Err(anyhow::anyhow!(""));
    };

    let (parent, advancement_type) = match &json.parent {
        Some(parent) => (
            Some(strip_mc_prefix(&parent.clone()).to_string()),
            match display.frame.as_deref() {
                Some("challenge") => AdvancementType::Challenge,
                Some("goal") => AdvancementType::Goal,
                _ => AdvancementType::Task,
            }
        ),
        None => (None, AdvancementType::Root)
    };

    // let requirements = old_reqs::old_get_requirements(&json.criteria);
    // let requirements = requirements::get_requirements(&json.criteria, json.requirements.clone(), &id, context);
    let (requirements, common_subjects) = requirements::get_requirements(&json.criteria, json.requirements.clone(), &id, context);

    fn translate(value: &serde_json::Value, context: &LoadingContext) -> String {
        match value {
            serde_json::Value::String(s) => {
                context.translate(s).unwrap_or_else(|| s.clone())
            },
            serde_json::Value::Object(obj) => {
                if let Some(translate_key) = obj.get("translate").and_then(|v| v.as_str()) {
                    context.translate(translate_key).unwrap_or_else(|| translate_key.to_string())
                } else if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                    text.to_string()
                } else {
                    "UNKNOWN".to_string()
                }
            }
            _ => "UNKNOWN".to_string(),
        }
    }

    let source = if id.contains(":") {
        id.split(":").next().unwrap().to_string()
    } else {
        "minecraft".to_string()
    };

    Ok(Advancement {
        key: id.to_string(),
        display_name: translate(&display.title, &context),
        description: translate(&display.description, &context),
        icon: json_to_icon(&display.icon),
        source,parent, advancement_type, requirements, common_subjects,

        // later
        category: String::new(),
        spreadsheet_info: SpreadsheetInfo { class: "".to_string(), requirement_details: None },
    })
}

fn json_to_icon(json: &serde_json::Value) -> Icon {
    if let Some(id) = json.get("id").and_then(|id| id.as_str()) {
        // get the player head skin component if its a head
        if id == "minecraft:player_head" {
            if let Ok(icon) = get_player_head(json) {
                return icon;
            }
        }
        // todo: shimmer detection
        return Icon::Item {
            name: strip_mc_prefix(id).to_string(),
            shimmering: false,
        };
    }
    Icon::Item {
        name: "barrier".to_string(),
        shimmering: false,
    }
}

fn get_player_head(json: &serde_json::Value) -> Result<Icon> {
    let properties = json.get("components")
        .context("Missing 'components'")?
        .get("profile")
        .context("Missing 'profile'")?
        .get("properties")
        .context("Missing 'properties'")?
        .as_array()
        .context("Properties is not an array")?;

    for property in properties {
        if property.get("name").and_then(|n| n.as_str()) == Some("textures") {
            let b64 = property.get("value").and_then(|v| v.as_str()).context("Missing texture value")?;
            let decoded = base64::prelude::BASE64_STANDARD.decode(b64)?;
            let texture_data: serde_json::Value = serde_json::from_slice(&decoded)?;

            let skin_url = texture_data.get("textures")
                .and_then(|t| t.get("SKIN"))
                .and_then(|s| s.get("url"))
                .and_then(|u| u.as_str())
                .context("Could not find SKIN url in texture data")?;
            
            let texture_id = skin_url.split("/").last().context("Invalid skin URL format")?.to_string();
            return Ok(Icon::PlayerHead { texture_id });
        }
    }
    Err(anyhow::anyhow!("No player head texture found in icon data"))
}