use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use crate::load::{context::LoadingContext, archive::expand_tag, strip_mc_prefix};

// ===== DATA MODEL =====

#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    #[serde(flatten)]
    pub base: BaseSubject,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supplements: Vec<Supplement>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum BaseSubject {
    Item { 
        ids: Vec<String>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        count: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_name: Option<String>
    },
    Block { 
        ids: Vec<String>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        loot_table: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<String>
    },
    Entity { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_name: Option<String>
    },
    Location { 
        #[serde(skip_serializing_if = "Vec::is_empty")]
        biomes: Vec<String>, 
        #[serde(skip_serializing_if = "Vec::is_empty")]
        structures: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dimension: Option<String>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        x: Option<serde_json::Value>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<serde_json::Value>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        z: Option<serde_json::Value> 
    },
    Effect { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        amplifier: Option<i32> 
    },
    Advancement { id: String },
    Stat { stat_type: String, target: String, value: i32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Supplement {
    Enchantment { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        level: Option<i32>, 
        #[serde(skip_serializing_if = "Option::is_none")]
        stored: Option<bool> 
    },
    Effect { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        amplifier: Option<i32> 
    },
    Entity { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<String> 
    },
    Biome { id: String },
}

// ===== TAG EXPANSION =====

fn expand_id_or_tag(id: &str, context: &LoadingContext) -> Vec<String> {
    if id.starts_with('#') {
        expand_tag(&context.tag_map, id)
    } else {
        vec![strip_mc_prefix(id).to_string()]
    }
}

fn expand_id_list(ids: &[String], context: &LoadingContext) -> Vec<String> {
    ids.iter()
        .flat_map(|id| expand_id_or_tag(id, context))
        .collect()
}

// ===== MAIN EXTRACTION PIPELINE =====

pub fn extract_subjects(conditions: &serde_json::Value, trigger: Option<&str>, context: &LoadingContext) -> Vec<Subject> {
    let Some(obj) = conditions.as_object() else {
        return Vec::new();
    };
    
    let mut subjects = Vec::new();
    
    // extract subjects based on trigger type
    if let Some(trigger_str) = trigger {
        subjects.extend(extract_trigger_subjects(trigger_str));
    }
    
    // iterate through all condition fields and extract relevant subjects
    for (key, value) in obj {
        match key.as_str() {
            "items" | "item" | "fired_from_weapon" => {
                subjects.extend(extract_items(value, context));
            },
            "blocks" | "block" => {
                subjects.extend(extract_blocks(value, context));
            },
            "entity" | "source" | "cause" | "bystander" | "lightning" | "victims" | 
            "parent" | "partner" | "child" | "projectile" => {
                subjects.extend(extract_entities(value, context));
            },
            "villager" => {
                subjects.extend(extract_villagers(value, context));
            },
            "player" => {
                subjects.extend(extract_players(value, context));
            },
            "damage" | "killing_blow" => {
                subjects.extend(extract_damage(value, context));
            },
            "effects" => {
                subjects.extend(extract_effects(value));
            },
            "potion" => {
                subjects.extend(extract_stored_potions(value));
            },
            "location" => {
                subjects.extend(extract_location(value, context));
            },
            "recipe_id" => {
                subjects.extend(extract_recipes(value, context));
            },
            "loot_table" => {
                subjects.extend(extract_loot_tables(value));
            },
            "advancement" => {
                subjects.extend(extract_advancement_conditions(value));
            },
            "rod" => {
                // fishing rod conditions - trigger already handles the rod
            },
            "projectile_count" => {
                subjects.extend(extract_crossbow_projectiles());
            },
            _ => {} // ignore unknown fields
        }
    }
    
    subjects
}

fn extract_trigger_subjects(trigger: &str) -> Vec<Subject> {
    let trigger_clean = strip_mc_prefix(trigger);
    
    match trigger_clean {
        "fishing_rod_hooked" => {
            vec![create_item_subject(vec!["fishing_rod"], None, None, None)]
        },
        "cured_zombie_villager" => {
            vec![create_entity_subject("zombie_villager", None, None)]
        },
        _ => Vec::new()
    }
}

// ===== ITEM EXTRACTION =====

fn extract_items(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    ensure_array(value)
        .into_iter()
        .filter_map(|item_obj| extract_single_item(item_obj, context))
        .collect()
}

fn extract_single_item(item_obj: &serde_json::Value, context: &LoadingContext) -> Option<Subject> {
    let mut item_ids = Vec::new();
    
    // extract item IDs from various formats
    if let Some(items_value) = item_obj.get("items") {
        item_ids.extend(extract_and_expand_ids(items_value, context));
    }
    if let Some(item_str) = item_obj.get("item").and_then(|v| v.as_str()) {
        item_ids.extend(expand_id(item_str, context));
    }
    
    // extract basic properties
    let count = extract_min_value(item_obj.get("count"));
    let (variant, custom_name) = extract_item_properties(item_obj);
    
    // extract supplements
    let mut supplements = Vec::new();
    if let Some(predicates) = item_obj.get("predicates") {
        supplements.extend(extract_item_predicates(predicates));
        
        // check for trim materials if no item IDs
        if item_ids.is_empty() {
            if let Some(trim_material) = extract_trim_material(predicates) {
                item_ids.push(trim_material);
            }
        }
    }
    if let Some(components) = item_obj.get("components") {
        supplements.extend(extract_item_components(components));
    }
    
    // fallback for enchanted books
    if item_ids.is_empty() && supplements.iter().any(|s| matches!(s, Supplement::Enchantment { .. })) {
        item_ids.push("enchanted_book".to_string());
    }
    
    if item_ids.is_empty() {
        return None;
    }
    
    Some(Subject {
        base: BaseSubject::Item { ids: item_ids, count, variant, custom_name },
        supplements,
    })
}

fn extract_item_properties(item_obj: &serde_json::Value) -> (Option<String>, Option<String>) {
    let mut variant = None;
    let mut custom_name = None;
    
    // extract variant from components (goat horns)
    if let Some(components) = item_obj.get("components") {
        if let Some(instrument) = components.get("minecraft:instrument").and_then(|v| v.as_str()) {
            variant = Some(strip_mc_prefix(instrument).to_string());
        }
        if let Some(name) = components.get("minecraft:custom_name").and_then(|v| v.as_str()) {
            custom_name = Some(name.to_string());
        }
    }
    
    // extract custom name from predicates
    if custom_name.is_none() {
        if let Some(predicates) = item_obj.get("predicates") {
            if let Some(name) = predicates.get("minecraft:custom_name").and_then(|v| v.as_str()) {
                custom_name = Some(name.to_string());
            }
        }
    }
    
    (variant, custom_name)
}

fn extract_item_predicates(predicates: &serde_json::Value) -> Vec<Supplement> {
    let mut supplements = Vec::new();
    
    // extract enchantments
    for enchant_type in ["enchantments", "stored_enchantments"] {
        if let Some(enchant_array) = predicates.get(enchant_type).and_then(|v| v.as_array()) {
            for enchantment in enchant_array {
                if let Some(enchant_id) = enchantment.get("enchantments").and_then(|v| v.as_str()) {
                    let id = strip_mc_prefix(enchant_id).to_string();
                    let level = extract_min_value(enchantment.get("levels"));
                    let stored = if enchant_type == "stored_enchantments" { Some(true) } else { None };
                    
                    supplements.push(Supplement::Enchantment { id, level, stored });
                }
            }
        }
    }
    
    // extract potion effects
    if let Some(potion_contents) = predicates.get("potion_contents") {
        if let Some(potion_id) = potion_contents.as_str() {
            supplements.push(Supplement::Effect { 
                id: strip_mc_prefix(potion_id).to_string(), 
                amplifier: None 
            });
        } else if let Some(potion_obj) = potion_contents.as_object() {
            if let Some(potion_id) = potion_obj.get("potion").and_then(|v| v.as_str()) {
                supplements.push(Supplement::Effect { 
                    id: strip_mc_prefix(potion_id).to_string(), 
                    amplifier: None 
                });
            }
        }
    }
    
    supplements
}

fn extract_item_components(components: &serde_json::Value) -> Vec<Supplement> {
    let mut supplements = Vec::new();
    
    // suspicious stew effects
    if let Some(stew_effects) = components.get("minecraft:suspicious_stew_effects").and_then(|v| v.as_array()) {
        for effect in stew_effects {
            if let Some(effect_id) = effect.get("id").and_then(|v| v.as_str()) {
                let id = strip_mc_prefix(effect_id).to_string();
                let amplifier = extract_min_value(effect.get("amplifier"));
                supplements.push(Supplement::Effect { id, amplifier });
            }
        }
    }
    
    // potion contents
    if let Some(potion_contents) = components.get("minecraft:potion_contents") {
        if let Some(potion_id) = potion_contents.get("potion").and_then(|v| v.as_str()) {
            supplements.push(Supplement::Effect { 
                id: strip_mc_prefix(potion_id).to_string(), 
                amplifier: None 
            });
        }
    }
    
    supplements
}

fn extract_trim_material(predicates: &serde_json::Value) -> Option<String> {
    predicates.get("minecraft:trim")
        .and_then(|trim_data| trim_data.get("material"))
        .and_then(|v| v.as_str())
        .map(|material| strip_mc_prefix(material).to_string())
}

// ===== BLOCK EXTRACTION =====

fn extract_blocks(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    // Handle simple string/array cases
    let expanded_ids = extract_and_expand_ids(value, context);
    if !expanded_ids.is_empty() {
        return vec![create_block_subject(expanded_ids, None, None)];
    }
    
    // Handle complex object cases
    ensure_array(value)
        .into_iter()
        .filter_map(|block_obj| extract_single_block(block_obj, context))
        .collect()
}

fn extract_single_block(block_obj: &serde_json::Value, context: &LoadingContext) -> Option<Subject> {
    let mut block_ids = Vec::new();
    
    if let Some(blocks_value) = block_obj.get("blocks") {
        block_ids.extend(extract_and_expand_ids(blocks_value, context));
    }
    if let Some(block_str) = block_obj.get("block").and_then(|v| v.as_str()) {
        block_ids.extend(expand_id(block_str, context));
    }
    
    if block_ids.is_empty() {
        return None;
    }
    
    Some(create_block_subject(block_ids, None, None))
}

// ===== ENTITY EXTRACTION =====

fn extract_villagers(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    extract_entities_with_type(value, Some("villager"), context)
}

fn extract_entities(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    extract_entities_with_type(value, None, context)
}

fn extract_entities_with_type(value: &serde_json::Value, hardcoded_type: Option<&str>, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    for entity_obj in ensure_array(value) {
        if is_inverted_condition(entity_obj) {
            continue;
        }
        
        let entity_types = if let Some(fixed_type) = hardcoded_type {
            vec![fixed_type.to_string()]
        } else {
            find_entity_types(entity_obj, context)
        };
        
        if !entity_types.is_empty() {
            // process entities with known types
            for entity_type in entity_types {
                if let Some(subject) = extract_single_entity(entity_obj, &entity_type) {
                    subjects.push(subject);
                }
                
                // extract additional subjects from NBT
                subjects.extend(extract_nbt_subjects(entity_obj, &entity_type));
            }
        } else {
            // handle entities without types but with effects
            subjects.extend(extract_standalone_effects(entity_obj));
        }
        
        // extract location and equipment data
        subjects.extend(extract_entity_context(entity_obj, context));
    }
    
    subjects
}

fn extract_single_entity(entity_obj: &serde_json::Value, entity_type: &str) -> Option<Subject> {
    let mut variant = extract_entity_variant(entity_obj);
    let mut custom_name = None;
    let mut supplements = Vec::new();
    
    // extract effects
    let effects_source = if entity_obj.get("condition").and_then(|v| v.as_str()) == Some("minecraft:entity_properties") {
        entity_obj.get("predicate")
    } else {
        Some(entity_obj)
    };
    
    if let Some(effects_obj) = effects_source.and_then(|src| src.get("effects")) {
        supplements.extend(extract_effects_from_entity(effects_obj));
    }
    
    // extract NBT data
    if let Some(nbt) = get_entity_nbt(entity_obj) {
        if let Some(name) = extract_custom_name_from_nbt(nbt) {
            custom_name = Some(name);
        }
        if let Some(nbt_variant) = extract_variant_from_nbt(nbt, entity_type) {
            variant = Some(nbt_variant);
        }
        
        // extract special NBT supplements
        supplements.extend(extract_nbt_supplements(nbt, entity_type));
    }
    
    Some(Subject {
        base: BaseSubject::Entity { 
            id: entity_type.to_string(), 
            variant, 
            custom_name 
        },
        supplements,
    })
}

fn extract_entity_context(entity_obj: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract location data
    for source in [entity_obj, entity_obj.get("predicate").unwrap_or(&serde_json::Value::Null)] {
        if let Some(location) = source.get("location") {
            subjects.extend(extract_location_predicate(location, context));
            break;
        }
    }
    
    // extract equipment as items
    for source in [entity_obj, entity_obj.get("predicate").unwrap_or(&serde_json::Value::Null)] {
        if let Some(equipment) = source.get("equipment") {
            subjects.extend(extract_equipment(equipment, context));
            break;
        }
    }
    
    subjects
}

fn extract_standalone_effects(entity_obj: &serde_json::Value) -> Vec<Subject> {
    let predicate = entity_obj.get("predicate").unwrap_or(entity_obj);
    
    if let Some(effects_obj) = predicate.get("effects") {
        extract_effects_from_entity(effects_obj)
            .into_iter()
            .map(|supplement| match supplement {
                Supplement::Effect { id, amplifier } => {
                    Subject {
                        base: BaseSubject::Effect { id, amplifier },
                        supplements: Vec::new(),
                    }
                }
                _ => unreachable!(),
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn find_entity_types(value: &serde_json::Value, context: &LoadingContext) -> Vec<String> {
    match value {
        serde_json::Value::Object(obj) => {
            // check for direct "type" field
            if let Some(type_value) = obj.get("type") {
                let types = extract_types_from_value(type_value, context);
                if !types.is_empty() {
                    return types;
                }
            }
            
            // check for component variants
            if let Some(component_types) = extract_types_from_components(obj) {
                if !component_types.is_empty() {
                    return component_types;
                }
            }
            
            // recursively search in nested objects
            for key in ["predicate", "type_specific"] {
                if let Some(nested_value) = obj.get(key) {
                    let nested_types = find_entity_types(nested_value, context);
                    if !nested_types.is_empty() {
                        return nested_types;
                    }
                }
            }
            
            // search other keys
            for (key, nested_value) in obj {
                if !["predicate", "type_specific"].contains(&key.as_str()) {
                    let nested_types = find_entity_types(nested_value, context);
                    if !nested_types.is_empty() {
                        return nested_types;
                    }
                }
            }
        }
        _ => {}
    }
    
    Vec::new()
}

fn extract_types_from_value(type_value: &serde_json::Value, context: &LoadingContext) -> Vec<String> {
    extract_and_expand_ids(type_value, context)
        .into_iter()
        .filter(|type_str| !type_str.contains("player"))
        .collect()
}

fn extract_types_from_components(obj: &serde_json::Map<String, serde_json::Value>) -> Option<Vec<String>> {
    if let Some(components) = obj.get("components").and_then(|v| v.as_object()) {
        for (key, _value) in components {
            if key.ends_with("/variant") {
                let entity_type = key.replace("/variant", "");
                let clean_type = strip_mc_prefix(&entity_type);
                if !clean_type.contains("player") {
                    return Some(vec![clean_type.to_string()]);
                }
            }
        }
    }
    None
}

fn extract_entity_variant(entity_obj: &serde_json::Value) -> Option<String> {
    for source in [entity_obj, entity_obj.get("predicate").unwrap_or(&serde_json::Value::Null)] {
        // check components for /variant keys
        if let Some(components) = source.get("components").and_then(|v| v.as_object()) {
            for (key, value) in components {
                if key.ends_with("/variant") {
                    if let Some(variant_str) = value.as_str() {
                        return Some(strip_mc_prefix(variant_str).to_string());
                    }
                }
            }
        }
        
        // check for direct variant field
        if let Some(variant_str) = source.get("variant").and_then(|v| v.as_str()) {
            return Some(strip_mc_prefix(variant_str).to_string());
        }
        
        // check type_specific variants
        if let Some(type_specific) = source.get("type_specific") {
            if let Some(variant_str) = type_specific.get("variant").and_then(|v| v.as_str()) {
                return Some(strip_mc_prefix(variant_str).to_string());
            }
        }
    }
    
    None
}

// ===== PLAYER EXTRACTION =====

fn extract_players(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    for player_obj in ensure_array(value) {
        if is_inverted_condition(player_obj) {
            continue;
        }
        
        let predicate = player_obj.get("predicate").unwrap_or(player_obj);
        
        // extract player effects as standalone effect subjects
        if let Some(effects_obj) = predicate.get("effects") {
            subjects.extend(extract_effects_from_entity_as_subjects(effects_obj));
        }
        
        // extract equipment as items
        for equipment_source in [player_obj, predicate] {
            if let Some(equipment) = equipment_source.get("equipment") {
                subjects.extend(extract_equipment(equipment, context));
            }
        }
        
        // extract vehicle and context
        if let Some(vehicle) = predicate.get("vehicle") {
            subjects.extend(extract_vehicle_with_location(vehicle, context));
        }
        
        subjects.extend(extract_player_context(player_obj, predicate, context));
        
        // process terms (any_of, all_of, etc.)
        if let Some(terms) = player_obj.get("terms").and_then(|v| v.as_array()) {
            subjects.extend(extract_player_terms(terms, context));
        }
    }
    
    subjects
}

fn extract_player_context(_player_obj: &serde_json::Value, predicate: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract location data
    if let Some(location) = predicate.get("location") {
        subjects.extend(extract_location(location, context));
    }
    
    // extract stepping on blocks
    if let Some(stepping_on) = predicate.get("stepping_on") {
        subjects.extend(extract_stepping_on_blocks(stepping_on, context));
    }
    
    // extract type-specific data
    if let Some(type_specific) = predicate.get("type_specific") {
        subjects.extend(extract_type_specific_data(type_specific, context));
    }
    
    subjects
}

fn extract_player_terms(terms: &[serde_json::Value], context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    for term in terms {
        if is_inverted_condition(term) {
            continue;
        }
        
        if let Some(predicate) = term.get("predicate") {
            // extract location data
            if let Some(location) = predicate.get("location") {
                subjects.extend(extract_location_predicate(location, context));
            }
            
            // extract vehicle and its location
            if let Some(vehicle) = predicate.get("vehicle") {
                subjects.extend(extract_vehicle_with_location(vehicle, context));
            }
            
            // extract equipment
            if let Some(equipment) = predicate.get("equipment") {
                subjects.extend(extract_equipment(equipment, context));
            }
        }
        
        // recursively process nested terms
        if let Some(nested_terms) = term.get("terms").and_then(|v| v.as_array()) {
            subjects.extend(extract_player_terms(nested_terms, context));
        }
    }
    
    subjects
}

fn extract_vehicle_with_location(vehicle: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract the vehicle entity
    if let Some(vehicle_type) = vehicle.get("type").and_then(|v| v.as_str()) {
        for entity_id in expand_id(vehicle_type, context) {
            subjects.push(create_entity_subject(&entity_id, None, None));
        }
    }
    
    // extract location from vehicle
    if let Some(location) = vehicle.get("location") {
        subjects.extend(extract_location_predicate(location, context));
    }
    
    // extract passengers
    if let Some(passenger) = vehicle.get("passenger") {
        if let Some(passenger_type) = passenger.get("type").and_then(|v| v.as_str()) {
            subjects.push(create_entity_subject(&strip_mc_prefix(passenger_type).to_string(), None, None));
        }
    }
    if let Some(passengers) = vehicle.get("passengers") {
        subjects.extend(extract_entities(passengers, context));
    }
    
    // extract equipment from vehicle
    if let Some(equipment) = vehicle.get("equipment") {
        subjects.extend(extract_equipment(equipment, context));
    }
    
    subjects
}

// ===== DAMAGE EXTRACTION =====

fn extract_damage(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract source and direct entities
    for role in ["source_entity", "direct_entity"] {
        if let Some(entity_data) = value.get(role) {
            let initial_count = subjects.len();
            subjects.extend(extract_entities(entity_data, context));
            
            // if no entities were added, extract equipment separately
            if subjects.len() == initial_count {
                if let Some(equipment) = entity_data.get("equipment") {
                    subjects.extend(extract_equipment(equipment, context));
                }
            }
        }
    }
    
    // extract damage type data
    if let Some(type_data) = value.get("type") {
        for role in ["source_entity", "direct_entity"] {
            if let Some(entity_data) = type_data.get(role) {
                let initial_count = subjects.len();
                subjects.extend(extract_entities(entity_data, context));
                
                if subjects.len() == initial_count {
                    if let Some(equipment) = entity_data.get("equipment") {
                        subjects.extend(extract_equipment(equipment, context));
                    }
                }
            }
        }
    }
    
    subjects
}

// ===== EFFECT EXTRACTION =====

fn extract_effects(value: &serde_json::Value) -> Vec<Subject> {
    let Some(effects_obj) = value.as_object() else {
        return Vec::new();
    };
    
    effects_obj
        .iter()
        .filter_map(|(effect_id, props)| {
            let id = strip_mc_prefix(effect_id).to_string();
            let amplifier = extract_min_value(props.get("amplifier"));
            
            Some(Subject {
                base: BaseSubject::Effect { id, amplifier },
                supplements: Vec::new(),
            })
        })
        .collect()
}

fn extract_effects_from_entity(effects_obj: &serde_json::Value) -> Vec<Supplement> {
    let Some(effects_map) = effects_obj.as_object() else {
        return Vec::new();
    };
    
    effects_map
        .iter()
        .filter_map(|(effect_id, props)| {
            let id = strip_mc_prefix(effect_id).to_string();
            let amplifier = extract_min_value(props.get("amplifier"));
            Some(Supplement::Effect { id, amplifier })
        })
        .collect()
}

fn extract_effects_from_entity_as_subjects(effects_obj: &serde_json::Value) -> Vec<Subject> {
    extract_effects_from_entity(effects_obj)
        .into_iter()
        .map(|supplement| match supplement {
            Supplement::Effect { id, amplifier } => Subject {
                base: BaseSubject::Effect { id, amplifier },
                supplements: Vec::new(),
            },
            _ => unreachable!(),
        })
        .collect()
}

fn extract_stored_potions(value: &serde_json::Value) -> Vec<Subject> {
    if let Some(effect_id) = value.as_str() {
        let supplements = vec![Supplement::Effect { 
            id: strip_mc_prefix(effect_id).to_string(), 
            amplifier: None 
        }];
        vec![Subject {
            base: BaseSubject::Item { 
                ids: vec!["potion".to_string()], 
                count: None,
                variant: None,
                custom_name: None
            },
            supplements,
        }]
    } else {
        Vec::new()
    }
}

// ===== LOCATION EXTRACTION =====

fn extract_location(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    for location_obj in ensure_array(value) {
        // handle direct block references
        if let Some(block_id) = location_obj.get("block") {
            if let Some(block_str) = block_id.as_str() {
                let block_ids = expand_id(block_str, context);
                subjects.push(create_block_subject(block_ids, None, None));
            }
        }
        
        // handle different condition types
        let condition = location_obj.get("condition").and_then(|v| v.as_str());
        let predicate = location_obj.get("predicate").unwrap_or(location_obj);
        
        match condition {
            Some("minecraft:match_tool") => {
                if let Some(items) = predicate.get("items") {
                    subjects.extend(extract_items(&serde_json::json!({"items": items}), context));
                }
            },
            Some("minecraft:location_check") => {
                subjects.extend(extract_location_predicate(predicate, context));
            },
            Some("minecraft:inverted") => {
                continue;
            },
            _ => {
                subjects.extend(extract_location_predicate(predicate, context));
            }
        }
        
        // handle location terms
        if let Some(terms) = location_obj.get("terms").and_then(|v| v.as_array()) {
            subjects.extend(extract_location_terms(terms, context));
        }
    }
    
    subjects
}

fn extract_location_predicate(predicate: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract location components
    subjects.extend(extract_location_components(predicate, context));
    
    // extract fluids as blocks
    if let Some(fluid_data) = predicate.get("fluid") {
        if let Some(fluids) = fluid_data.get("fluids") {
            let fluid_ids = extract_and_expand_ids(fluids, context);
            if !fluid_ids.is_empty() {
                subjects.push(create_block_subject(fluid_ids, None, None));
            }
        }
    }
    
    // extract items from location
    if let Some(items) = predicate.get("items") {
        subjects.extend(extract_items(items, context));
    }
    
    // extract blocks with state properties
    if let Some(block_data) = predicate.get("block") {
        subjects.extend(extract_blocks_with_state(block_data, context));
    }
    
    subjects
}

fn extract_location_components(predicate: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    let mut biomes = Vec::new();
    let mut structures = Vec::new();
    let mut dimension = None;
    let mut x = None;
    let mut y = None;
    let mut z = None;
    
    // extract biomes and structures
    if let Some(biome_value) = predicate.get("biomes") {
        biomes.extend(extract_and_expand_ids(biome_value, context));
    }
    if let Some(structure_value) = predicate.get("structures") {
        structures.extend(extract_and_expand_ids(structure_value, context));
    }
    
    // extract dimension
    if let Some(dimension_str) = predicate.get("dimension").and_then(|v| v.as_str()) {
        dimension = Some(strip_mc_prefix(dimension_str).to_string());
    }
    
    // extract position constraints
    if let Some(position) = predicate.get("position") {
        x = position.get("x").cloned();
        y = position.get("y").cloned();
        z = position.get("z").cloned();
    }
    
    // create location subject if we have any location data
    if !biomes.is_empty() || !structures.is_empty() || dimension.is_some() || 
       x.is_some() || y.is_some() || z.is_some() {
        subjects.push(Subject {
            base: BaseSubject::Location { 
                biomes, 
                structures, 
                dimension, 
                x, 
                y, 
                z 
            },
            supplements: Vec::new(),
        });
    }
    
    subjects
}

fn extract_blocks_with_state(block_data: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    if let Some(blocks) = block_data.get("blocks") {
        let block_ids = extract_and_expand_ids(blocks, context);
        if !block_ids.is_empty() {
            let variant = extract_block_state_variant(block_data);
            return vec![create_block_subject(block_ids, None, variant)];
        }
    }
    Vec::new()
}

fn extract_block_state_variant(block_data: &serde_json::Value) -> Option<String> {
    if let Some(state) = block_data.get("state") {
        let mut variant_parts = Vec::new();
        
        if let Some(instrument) = state.get("instrument").and_then(|v| v.as_str()) {
            variant_parts.push(instrument.to_string());
        }
        
        if let Some(note_value) = state.get("note") {
            let note_str = match note_value {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            };
            if let Some(note) = note_str {
                variant_parts.push(note);
            }
        }
        
        if !variant_parts.is_empty() {
            return Some(variant_parts.join("_"));
        }
    }
    None
}

fn extract_location_terms(terms: &[serde_json::Value], context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    for term in terms {
        // handle direct blocks in terms
        if let Some(block_id) = term.get("block") {
            if let Some(block_str) = block_id.as_str() {
                let block_ids = expand_id(block_str, context);
                subjects.push(create_block_subject(block_ids, None, None));
            }
        }
        
        // handle predicate in terms
        if let Some(predicate) = term.get("predicate") {
            if let Some(items) = predicate.get("items") {
                subjects.extend(extract_items(&serde_json::json!({"items": items}), context));
            }
            
            if let Some(block_data) = predicate.get("block") {
                subjects.extend(extract_blocks_with_state(block_data, context));
            }
        }
        
        // handle nested terms recursively
        if let Some(nested_terms) = term.get("terms").and_then(|v| v.as_array()) {
            subjects.extend(extract_location_terms(nested_terms, context));
        }
    }
    
    subjects
}

// ===== MISC EXTRACTION =====

fn extract_recipes(value: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    if let Some(recipe_id) = value.as_str() {
        if let Some(item_id) = context.recipe_map.get(recipe_id) {
            vec![Subject {
                base: BaseSubject::Item { 
                    ids: vec![item_id.clone()], 
                    count: None,
                    variant: None,
                    custom_name: None
                },
                supplements: Vec::new(),
            }]
        } else {
            println!("[WARN] Recipe not found: {}", recipe_id);
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

fn extract_loot_tables(value: &serde_json::Value) -> Vec<Subject> {
    if let Some(loot_table_id) = value.as_str() {
        vec![create_block_subject(
            vec!["chest".to_string()], 
            Some(loot_table_id.to_string()), 
            None
        )]
    } else {
        Vec::new()
    }
}

fn extract_advancement_conditions(value: &serde_json::Value) -> Vec<Subject> {
    ensure_array(value)
        .into_iter()
        .filter_map(|adv_obj| {
            adv_obj.as_str().map(|adv_id| Subject {
                base: BaseSubject::Advancement { id: strip_mc_prefix(adv_id).to_string() },
                supplements: Vec::new(),
            })
        })
        .collect()
}

fn extract_crossbow_projectiles() -> Vec<Subject> {
    vec![create_item_subject(vec!["crossbow"], None, None, None)]
}

fn extract_stepping_on_blocks(stepping_on: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    if let Some(block_data) = stepping_on.get("block") {
        extract_blocks(block_data, context)
    } else {
        Vec::new()
    }
}

fn extract_equipment(equipment: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let Some(equipment_obj) = equipment.as_object() else {
        return Vec::new();
    };
    
    equipment_obj
        .values()
        .filter_map(|data| {
            if let Some(items_value) = data.get("items") {
                extract_single_item(&serde_json::json!({"items": items_value}), context)
            } else if let Some(predicates) = data.get("predicates") {
                if let Some(trim_material) = extract_trim_material(predicates) {
                    Some(create_item_subject(vec![&trim_material], None, None, None))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn extract_type_specific_data(type_specific: &serde_json::Value, context: &LoadingContext) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract advancements
    if let Some(advancements) = type_specific.get("advancements").and_then(|v| v.as_object()) {
        for (adv_id, required) in advancements {
            if required.as_bool() == Some(true) {
                subjects.push(Subject {
                    base: BaseSubject::Advancement { id: strip_mc_prefix(adv_id).to_string() },
                    supplements: Vec::new(),
                });
            }
        }
    }
    
    // extract stats
    if let Some(stats) = type_specific.get("stats").and_then(|v| v.as_array()) {
        for stat in stats {
            if let (Some(stat_type), Some(stat_target), Some(value)) = (
                stat.get("type").and_then(|v| v.as_str()),
                stat.get("stat").and_then(|v| v.as_str()),
                extract_min_value(stat.get("value"))
            ) {
                subjects.push(Subject {
                    base: BaseSubject::Stat {
                        stat_type: strip_mc_prefix(stat_type).to_string(),
                        target: strip_mc_prefix(stat_target).to_string(),
                        value,
                    },
                    supplements: Vec::new(),
                });
            }
        }
    }
    
    // extract looking at entities
    if let Some(looking_at) = type_specific.get("looking_at") {
        subjects.extend(extract_entities(&serde_json::json!([looking_at]), context));
    }
    
    subjects
}

// ===== NBT PROCESSING =====

fn get_entity_nbt(entity_obj: &serde_json::Value) -> Option<&str> {
    for source in [entity_obj, entity_obj.get("predicate").unwrap_or(&serde_json::Value::Null)] {
        if let Some(nbt) = source.get("nbt").and_then(|v| v.as_str()) {
            return Some(nbt);
        }
    }
    None
}

fn extract_nbt_subjects(entity_obj: &serde_json::Value, entity_id: &str) -> Vec<Subject> {
    if let Some(nbt) = get_entity_nbt(entity_obj) {
        extract_additional_subjects_from_nbt(nbt, entity_id)
    } else {
        Vec::new()
    }
}

fn extract_nbt_supplements(nbt: &str, entity_type: &str) -> Vec<Supplement> {
    let mut supplements = Vec::new();
    
    // extract panda hidden gene
    if entity_type == "panda" {
        if let Some(captures) = regex::Regex::new(r#"HiddenGene:\s*["\']([^"\']+)["\']"#)
            .ok()
            .and_then(|re| re.captures(nbt))
        {
            if let Some(hidden_gene) = captures.get(1) {
                let variant = strip_mc_prefix(hidden_gene.as_str()).to_string();
                supplements.push(Supplement::Entity {
                    id: "panda".to_string(),
                    variant: Some(variant)
                });
            }
        }
    }
    
    // extract villager biome type
    if entity_type == "villager" {
        if let Some(captures) = regex::Regex::new(r#"type:\s*["\']([^"\']+)["\']"#)
            .ok()
            .and_then(|re| re.captures(nbt))
        {
            if let Some(biome_type) = captures.get(1) {
                let biome_id = strip_mc_prefix(biome_type.as_str()).to_string();
                supplements.push(Supplement::Biome { id: biome_id });
            }
        }
    }
    
    supplements
}

fn extract_custom_name_from_nbt(nbt: &str) -> Option<String> {
    regex::Regex::new(r#"CustomName:\s*["\']([^"\']+)["\']"#)
        .ok()?
        .captures(nbt)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn extract_variant_from_nbt(nbt: &str, entity_id: &str) -> Option<String> {
    match entity_id {
        "villager" => extract_nbt_string(nbt, r#"profession:\s*["\']([^"\']+)["\']"#),
        "panda" => {
            extract_nbt_string(nbt, r#"MainGene:\s*["\']([^"\']+)["\']"#)
                .or_else(|| extract_nbt_string(nbt, r#"Gene:\s*["\']([^"\']+)["\']"#))
        },
        "fox" => extract_nbt_string(nbt, r#"Type:\s*([^,}]+)"#)
            .map(|s| strip_mc_prefix(&s.trim_matches('"')).to_string()),
        "goat" => {
            if regex::Regex::new(r"IsScreamingGoat:\s*1b?").ok()?.is_match(nbt) {
                Some("screaming".to_string())
            } else {
                None
            }
        },
        "horse" | "tropical_fish" => extract_nbt_number(nbt, r"Variant:\s*(\d+)"),
        "axolotl" | "parrot" | "rabbit" | "cat" | "llama" | "trader_llama" => {
            extract_mapped_variant(nbt, entity_id)
        },
        _ => None,
    }
}

fn extract_nbt_string(nbt: &str, pattern: &str) -> Option<String> {
    regex::Regex::new(pattern)
        .ok()?
        .captures(nbt)?
        .get(1)
        .map(|m| strip_mc_prefix(m.as_str()).to_string())
}

fn extract_nbt_number(nbt: &str, pattern: &str) -> Option<String> {
    regex::Regex::new(pattern)
        .ok()?
        .captures(nbt)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn extract_mapped_variant(nbt: &str, entity_id: &str) -> Option<String> {
    let patterns = [r"Variant:\s*(\d+)", r"RabbitType:\s*(\d+)", r"CatType:\s*(\d+)", r"Type:\s*(\d+)"];
    
    for pattern in patterns.iter() {
        if let Some(captures) = regex::Regex::new(pattern).ok()?.captures(nbt) {
            if let Some(variant_str) = captures.get(1) {
                if let Ok(variant_num) = variant_str.as_str().parse::<i32>() {
                    if let Some(variant_name) = get_entity_variant_name(entity_id, variant_num) {
                        return Some(variant_name);
                    }
                }
            }
        }
    }
    None
}

fn get_entity_variant_name(entity: &str, variant_num: i32) -> Option<String> {
    match entity {
        "axolotl" => match variant_num {
            0 => Some("lucy".to_string()),
            1 => Some("wild".to_string()),
            2 => Some("gold".to_string()),
            3 => Some("cyan".to_string()),
            4 => Some("blue".to_string()),
            _ => None,
        },
        "parrot" => match variant_num {
            0 => Some("red_blue".to_string()),
            1 => Some("blue".to_string()),
            2 => Some("green".to_string()),
            3 => Some("yellow_blue".to_string()),
            4 => Some("gray".to_string()),
            _ => None,
        },
        "rabbit" => match variant_num {
            0 => Some("brown".to_string()),
            1 => Some("white".to_string()),
            2 => Some("black".to_string()),
            3 => Some("white_splotched".to_string()),
            4 => Some("gold".to_string()),
            5 => Some("salt".to_string()),
            99 => Some("evil".to_string()),
            _ => None,
        },
        "cat" => match variant_num {
            0 => Some("white".to_string()),
            1 => Some("tuxedo".to_string()),
            2 => Some("ginger".to_string()),
            3 => Some("siamese".to_string()),
            4 => Some("british_shorthair".to_string()),
            5 => Some("calico".to_string()),
            6 => Some("persian".to_string()),
            7 => Some("ragdoll".to_string()),
            8 => Some("tabby".to_string()),
            9 => Some("black".to_string()),
            10 => Some("jellie".to_string()),
            _ => None,
        },
        "llama" | "trader_llama" => match variant_num {
            0 => Some("creamy".to_string()),
            1 => Some("white".to_string()),
            2 => Some("brown".to_string()),
            3 => Some("gray".to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn extract_additional_subjects_from_nbt(nbt: &str, entity_id: &str) -> Vec<Subject> {
    let mut subjects = Vec::new();
    
    // extract passenger entities
    if nbt.contains("Passengers:") {
        let passenger_matches = regex::Regex::new(r#"id:\s*["\']([^"\']+)["\']"#)
            .ok()
            .map(|re| re.find_iter(nbt).collect::<Vec<_>>())
            .unwrap_or_default();
        
        for passenger_match in passenger_matches {
            let passenger_id = strip_mc_prefix(passenger_match.as_str()
                .trim_start_matches("id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
            ).to_string();
            
            if passenger_id != entity_id {
                subjects.push(create_entity_subject(&passenger_id, None, None));
            }
        }
    }
    
    // extract nested items
    let item_patterns = [
        r#"Item:\s*\{[^}]*id:\s*["\']([^"\']+)["\']"#,
        r#"weapon:\s*\{[^}]*id:\s*["\']([^"\']+)["\']"#,
    ];
    
    for pattern in item_patterns.iter() {
        if let Some(regex) = regex::Regex::new(pattern).ok() {
            for captures in regex.captures_iter(nbt) {
                if let Some(item_id) = captures.get(1) {
                    subjects.push(create_item_subject(
                        vec![strip_mc_prefix(item_id.as_str())], 
                        None, 
                        None, 
                        None
                    ));
                }
            }
        }
    }
    
    // extract nested potions
    if let Some(regex) = regex::Regex::new(r#"potion:\s*["\']([^"\']+)["\']"#).ok() {
        for captures in regex.captures_iter(nbt) {
            if let Some(potion_id) = captures.get(1) {
                let supplements = vec![Supplement::Effect { 
                    id: strip_mc_prefix(potion_id.as_str()).to_string(), 
                    amplifier: None 
                }];
                subjects.push(Subject {
                    base: BaseSubject::Item { 
                        ids: vec!["potion".to_string()], 
                        count: None,
                        variant: None,
                        custom_name: None
                    },
                    supplements,
                });
            }
        }
    }
    
    // extract carried blocks (enderman)
    if entity_id == "enderman" {
        let block_patterns = [
            r#"carriedBlockState:\s*\{[^}]*Name:\s*["\']([^"\']+)["\']"#,
            r#"carriedBlockState:\s*["\']([^"\']+)["\']"#,
        ];
        
        for pattern in block_patterns.iter() {
            if let Some(regex) = regex::Regex::new(pattern).ok() {
                if let Some(captures) = regex.captures(nbt) {
                    if let Some(block_id) = captures.get(1) {
                        subjects.push(create_block_subject(
                            vec![strip_mc_prefix(block_id.as_str()).to_string()], 
                            None, 
                            None
                        ));
                        break;
                    }
                }
            }
        }
    }
    
    subjects
}

// ===== HELPER FUNCTIONS =====

fn ensure_array(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    match value {
        serde_json::Value::Array(arr) => arr.iter().collect(),
        other => vec![other],
    }
}

fn extract_min_value(value: Option<&serde_json::Value>) -> Option<i32> {
    value.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64().map(|x| x as i32),
        serde_json::Value::Object(obj) => obj.get("min").and_then(|min| min.as_i64().map(|x| x as i32)),
        _ => None,
    })
}

fn extract_string_array(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(arr) => {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        }
        _ => Vec::new(),
    }
}

fn is_inverted_condition(obj: &serde_json::Value) -> bool {
    obj.get("condition").and_then(|v| v.as_str()) == Some("minecraft:inverted")
}

// ===== SUBJECT CONSTRUCTORS =====

fn create_item_subject(ids: Vec<&str>, count: Option<i32>, variant: Option<String>, custom_name: Option<String>) -> Subject {
    Subject {
        base: BaseSubject::Item { 
            ids: ids.into_iter().map(|s| s.to_string()).collect(), 
            count, 
            variant, 
            custom_name 
        },
        supplements: Vec::new(),
    }
}

fn create_block_subject(ids: Vec<String>, loot_table: Option<String>, variant: Option<String>) -> Subject {
    Subject {
        base: BaseSubject::Block { ids, loot_table, variant },
        supplements: Vec::new(),
    }
}

fn create_entity_subject(id: &str, variant: Option<String>, custom_name: Option<String>) -> Subject {
    Subject {
        base: BaseSubject::Entity { 
            id: id.to_string(), 
            variant, 
            custom_name 
        },
        supplements: Vec::new(),
    }
}

// ===== DEBUG INTEGRATION =====

#[derive(Serialize)]
struct AdvancementDebugInfo {
    requirements: BTreeMap<String, Vec<Subject>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requirement_groups: Vec<Vec<String>>,
}

static DEBUG_OUTPUT: std::sync::Mutex<Option<BTreeMap<String, AdvancementDebugInfo>>> = std::sync::Mutex::new(None);

pub fn init_debug() {
    *DEBUG_OUTPUT.lock().unwrap() = Some(BTreeMap::new());
}

pub fn write_debug_files() {
    let latest_path = "../reqs_latest.json";
    let old_path = "../reqs_old.json";
    
    if std::path::Path::new(latest_path).exists() {
        let _ = std::fs::copy(latest_path, old_path);
        println!("Backed up previous requirements to {}", old_path);
    }
    
    if let Ok(debug_map) = DEBUG_OUTPUT.lock() {
        if let Some(ref map) = *debug_map {
            if let Ok(json_output) = serde_json::to_string_pretty(map) {
                let _ = std::fs::write(latest_path, json_output);
                println!("Written requirements debug to {} ({} advancements)", latest_path, map.len());
            }
        }
    }
}

// ===== PUBLIC API =====

pub fn get_requirements(
    criteria_json: &HashMap<String, Criteria>, 
    requirement_groups: Option<Vec<Vec<String>>>,
    advancement_id: &str,
    context: &LoadingContext
) -> BTreeMap<String, Vec<Subject>> {
    let mut requirements_map = BTreeMap::new();
    
    // process each criterion and extract subjects
    for (key, criterion) in criteria_json {
        let default_conditions = serde_json::Value::Object(serde_json::Map::new());
        let conditions = criterion.conditions.as_ref().unwrap_or(&default_conditions);
        let subjects = extract_subjects(conditions, Some(&criterion.trigger), context);
        if !subjects.is_empty() {
            requirements_map.insert(strip_mc_prefix(key).to_string(), subjects);
        }
    }
    
    // store debug data
    let debug_info = AdvancementDebugInfo {
        requirements: requirements_map.clone(),
        requirement_groups: requirement_groups.unwrap_or_default(),
    };
    
    if let Ok(mut debug_map) = DEBUG_OUTPUT.lock() {
        if let Some(ref mut map) = *debug_map {
            map.insert(advancement_id.to_string(), debug_info);
        }
    }
    
    requirements_map
}

#[derive(Deserialize)]
pub struct Criteria {
    pub trigger: String,
    pub conditions: Option<serde_json::Value>,
}