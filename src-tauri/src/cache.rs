use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use std::{path::PathBuf};
use tokio::{fs};
use std::fs as std_fs;
use std::{path::Path as StdPath};

pub const CACHE_DIR: &str = "../cached";
pub const CACHE_URL: &str = "cached";

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub face: String, // b64 swag
}

#[derive(Clone)]
pub struct Cache {
    pub profiles: Arc<Mutex<HashMap<String, Profile>>>,
}
impl Cache {
    pub async fn new() -> Result<Self> {
        let profile_path = PathBuf::from(CACHE_DIR).join("profiles.json");
        let initial_profiles = if profile_path.exists() {
            let content = fs::read_to_string(profile_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Cache {profiles: Arc::new(Mutex::new(initial_profiles))})
    }

    pub async fn get_player(&self, uuid: &str) -> Option<Profile> {
        self.profiles.lock().await.get(uuid).cloned()
    }

    pub async fn cache_player(&self, uuid: &str, name: &str, face_bytes: Vec<u8>) -> Result<()> {
        let player_data = Profile {
            name: name.to_string(),
            face: base64::engine::general_purpose::STANDARD.encode(&face_bytes),
        };

        let mut cache = self.profiles.lock().await;
        cache.insert(uuid.to_string(), player_data);

        let profile_path = PathBuf::from(CACHE_DIR).join("profiles.json");
        fs::create_dir_all(CACHE_DIR).await?;
        let json_string = serde_json::to_string_pretty(&*cache)?;
        fs::write(profile_path, json_string).await?;

        println!("[CACHE] Wrote cache for {}", uuid);
        Ok(())
    }

    pub async fn get_cached_or_fetch(&self, uuid: &str) -> Option<(String, String)> {
        if let Some(cached_data) = self.get_player(uuid).await {
            println!("[CACHE] Hit for profile {}", uuid);
            return Some((cached_data.name, format!("data:image/png;base64,{}", cached_data.face)));
        }

        println!("[CACHE] Miss for profile {}. Fetching from API.", uuid);

        match tokio::try_join!(
            crate::outbound::fetch_username(uuid),
            crate::outbound::fetch_user_face(uuid)
        ) {
            Ok((name, avatar_bytes)) => {
                if let Err(e) = self.cache_player(uuid, &name, avatar_bytes.clone()).await {
                    eprintln!("[CACHE] Failed to cache player {}: {}", uuid, e);
                }

                let avatar_base64 = base64::engine::general_purpose::STANDARD.encode(&avatar_bytes);
                Some((name, format!("data:image/png;base64,{}", avatar_base64)))
            }
            Err(e) => {
                eprintln!("[CACHE] Failed to fetch player info for {}: {}", uuid, e);
                None
            }
        }
    }
}

pub fn cache_world_icon(world_path: &StdPath) -> Result<Option<String>> {
    let source_icon_path = world_path.join("icon.png");
    
    if source_icon_path.exists() {
        let dest_dir = PathBuf::from(CACHE_DIR).join("world");
        let dest_icon_path = dest_dir.join("icon.png");

        std_fs::create_dir_all(&dest_dir)?;

        std_fs::copy(&source_icon_path, &dest_icon_path)?;
        println!("[CACHE] Copied world icon to {:?}", dest_icon_path);

        return Ok(Some(format!("/{}/world/icon.png", CACHE_URL)));
    }
    
    Ok(None)
}