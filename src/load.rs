use anyhow::{Context, Result};
use base64::Engine;
use flate2::read::GzDecoder;
use serde::{Deserialize};
use std::{collections::{HashMap, HashSet}, fs, io::Read, path::Path};
use walkdir::WalkDir;
use zip::ZipArchive;
use crate::structs::*;

// const WORLD_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\New World (1)";
pub const WORLD_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\New World";
// pub const WORLD_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\profiles\\My BAC pack\\saves\\test";
const SPREADSHEET_PATH: &str = "spreadsheet_list.csv";
const MINECRAFT_JAR_PATH: &str = "C:\\Users\\Sebastian\\AppData\\Roaming\\ModrinthApp\\meta\\versions\\1.21.5-21.5.75\\1.21.5-21.5.75.jar";
const PLAYER_ADVANCEMENT_REL_PATH: &str = "advancements"; // crazy biz
pub const PLAYER_STATS_REL_PATH: &str = "stats";
const LANGUAGE_PATH: &str = "assets/minecraft/lang/en_us.json"; // jar:
const ADVANCEMENT_PATH: &str = "data/minecraft/advancement/"; // jar:
const STRIP_MC_PREFIX: bool = true;

pub fn strip_mc_prefix(s: &str) -> &str {
    if !STRIP_MC_PREFIX { return s }
    s.strip_prefix("minecraft:").unwrap_or(s)
}

pub async fn load(cache: &crate::cache::Cache) -> Result<Data> {
    let world_path = Path::new(WORLD_PATH);

    let world = load_world(world_path)?;
    let mut progress = HashMap::new();
    let mut players = load_player_data(world_path, &mut progress)?;

    // grab user names and faces, tries to fetch them if we don't have them
    crate::cache::init_profiles(cache, &mut players).await;

    // load jar
    let mut minecraft_jar = ZipArchive::new(std::io::BufReader::new(fs::File::open(MINECRAFT_JAR_PATH)?))?;

    let mut advancements = HashMap::new();
    load_vanilla_advancements(&mut advancements, &mut minecraft_jar)?;
    load_datapack_advancements(&mut advancements, world_path)?;

    let (spreadsheet_data, classes) = load_spreadsheet()?;
    assign_spreadsheet_info(&mut advancements, &spreadsheet_data);
    let categories = assign_categories(&mut advancements);

    // invert player progress to query by advancement instead

    println!("{:#?}", progress);

    Ok(Data {
        world,
        players,
        advancements,
        categories,
        classes,
        progress,
    })
}

fn load_world(world_path: &Path) -> Result<World> {
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

    let icon_path = crate::cache::cache_world_icon(world_path)?;

    Ok(World {
        name,
        version,
        icon_path,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read JSON file at {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("Invalid JSON in file {}", path.display()))
}

fn load_player_data(
    world_path: &Path,
    progress: &mut HashMap<String, HashMap<String, AdvancementProgress>>
) -> Result<HashMap<String, Player>> {
    let mut players = HashMap::new();
    let advancements_dir = world_path.join(PLAYER_ADVANCEMENT_REL_PATH);
    let stats_dir = world_path.join(PLAYER_STATS_REL_PATH);

    if !advancements_dir.exists() || !stats_dir.exists() {
        return Ok(players);
    }

    // basically just each player
    for stat_entry in fs::read_dir(&stats_dir)? {
        let stat_path = stat_entry?.path();
        if !stat_path.is_file() || stat_path.extension().map(|ext| ext != "json").unwrap_or(true) {
            continue;
        }

        // filenames are the uuid
        let uuid = stat_path.file_stem().unwrap().to_str().unwrap().to_string();
        let stats = load_player_stats(&stat_path)?;

        players.insert(
            uuid.clone(),
            Player {
                uuid: uuid.clone(),
                stats,
                name: None, avatar_url: None,
            },
        );

        let advancements_path = advancements_dir.join(format!("{}.json", uuid));
        let player_progress = load_advancement_progress(&advancements_path)?;

        for (advancement_key, progress_details) in player_progress {
            progress.entry(advancement_key).or_default().insert(uuid.clone(), progress_details);
        }
    }

    Ok(players)
}

pub fn load_advancement_progress(path: &Path) -> Result<HashMap<String, AdvancementProgress>> {
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

pub fn load_player_stats(path: &Path) -> Result<PlayerStats> {
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

    if !Path::new(SPREADSHEET_PATH).exists() {
        return Ok((HashMap::new(), vec!["Unknown".to_string()]));
    }

    let mut spreadsheet = HashMap::new();
    let mut classes = HashSet::new();

    let content = fs::read_to_string(SPREADSHEET_PATH)?;
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


fn load_vanilla_advancements(advancements: &mut HashMap<String, Advancement>, minecraft_jar: &mut ZipArchive<std::io::BufReader<fs::File>>) -> Result<()> {
    println!("Getting vanilla advancements");

    // need to get language file for this one
    let lang_map: HashMap<String, String> = {
        println!("[LOAD] Reading language file ({})", LANGUAGE_PATH);
        let mut lang_file = minecraft_jar.by_name(LANGUAGE_PATH)?;
        let mut content = String::new();
        lang_file.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };
    println!("[LOAD] Successfully loaded {} language entries.", lang_map.len());

    // goofy ahh zip thing gives a list of files
    for i in 0..minecraft_jar.len() {
        let mut file = minecraft_jar.by_index(i)?;
        let path = file.name();
        if !path.starts_with(ADVANCEMENT_PATH) || !path.ends_with(".json") { continue }

        // the id is the path
        let id = format!("minecraft:{}", 
            path.strip_prefix(ADVANCEMENT_PATH).unwrap().strip_suffix(".json").unwrap().to_string());
        
        if path.starts_with("recipe") { continue }

        // read it like this because it's zipped
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(advancement) = json_to_advancement(&id, &json,  Some(&lang_map)) {
            advancements.insert(strip_mc_prefix(&id).to_string(), advancement);
        }
    }

    println!(
        "[LOAD] Finished. Successfully loaded {} vanilla advancements.",
        advancements.len()
    );
    Ok(())
}

fn load_datapack_advancements(advancements: &mut HashMap<String, Advancement>, world_path: &Path) -> Result<()> {
    let datapacks_dir = world_path.join("datapacks");
    if !datapacks_dir.exists() { return Ok(()); }

    for entry in WalkDir::new(datapacks_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if !path.is_file() || !path.extension().is_some_and(|ext| ext == "json") {
            continue
        }

        let components: Vec<_> = path.components().collect();
        let Some(data_index) = components.iter().position(|&c| c.as_os_str() == "data") else {
            continue
        };

        if components.len() <= data_index + 2 {
            continue;
        }

        let advancements_folder_name = components[data_index + 2].as_os_str();
        if advancements_folder_name != "advancement" {
            continue
        }

        let Some(namespace) = components[data_index + 1].as_os_str().to_str() else { continue };

        // println!("debug: Processing file: {}", path.display());

        let id = components[data_index + 3..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
            .strip_suffix(".json").unwrap().to_string();

        let id = format!("{}:{}", namespace, id);

        let json = match read_json(path) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("[WARN] Failed to read datapack advancement {}: {}", id, e);
                continue
            }
        };

        if let Some(advancement) = json_to_advancement(&id, &json, None) {
            advancements.insert(strip_mc_prefix(&id).to_string(), advancement);
        }
    }
    Ok(())
}


fn json_to_advancement(id: &str, json: &serde_json::Value, lang: Option<&HashMap<String, String>>) -> Option<Advancement> {
    let display = json.get("display")?;

    let get_text = |value: &serde_json::Value| -> String {
        if let Some(s) = value.as_str() { return s.to_string(); }
        if let Some(obj) = value.as_object() {
            if let Some(key) = obj.get("translate").and_then(|v| v.as_str()) {
                if let Some(lang_map) = lang {
                    return lang_map.get(key).cloned().unwrap_or_else(|| key.to_string());
                } else {
                    return key.to_string();
                }
            }
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                return text.to_string();
            }
        }
        "Unknown".to_string()
    };

    let display_name = get_text(&display["title"]);
    let description = get_text(&display["description"]);
    let icon = json_to_icon(&display["icon"]);
    let parent = json.get("parent").and_then(|p| p.as_str())
        .map(|p| strip_mc_prefix(p).to_string());
    
    let advancement_type = if parent.is_none() {
        AdvancementType::Root
    } else {
        match display.get("frame").and_then(|frame| frame.as_str()).unwrap_or("task") {
            "challenge" => AdvancementType::Challenge,
            "goal" => AdvancementType::Goal,
            _ => AdvancementType::Task,
        }
    };

    let source = id.split(":").next().unwrap_or("minecraft").to_string();

    // get criteria here
    let requirements = crate::requirements::get_requirements(&json);
    // println!("debug: Requirements for {}: {:?}", id, requirements);

    Some(Advancement {
        key: strip_mc_prefix(id).to_string(),
        display_name, description, icon,
        parent, advancement_type, source, 
        requirements,

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
    /*
        "icon": {
            "id": "minecraft:player_head",
            "components": {"profile":{"id":[-350135654,1482441980,-1270006103,864641199],"properties":[{"name":"textures","value":"eyJ0ZXh0dXJlcyI6eyJTS0lOIjp7InVybCI6Imh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOGM5MDdhN2Y0ZjBjM2E2YzljOWI3OTM0OGY3YmYxOTkzNjk1YjZkMmVhZTJmYWI3MDRhMWE0ZDliODI4OGNiZSJ9fX0="}]}}
        },

        decoded:
        {
            "textures": {
                "SKIN": {
                    "url": "http://textures.minecraft.net/texture/50c410fad8d9d8825ad56b0e443e2777a6b46bfa20dacd1d2f55edc71fbeb06d"
                }
            }
        }
    */
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
