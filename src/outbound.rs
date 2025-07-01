use std::time::Duration;

use serde::Deserialize;
use anyhow::{anyhow};
use anyhow::Result;
use anyhow::Context;

/*
Attributions
mc-heads.net
Iso heads https://mc-heads.net/head/<texture_id>/32
Faces https://mc-heads.net/avatar/<uuid>

minecraft.wiki
biomes https://minecraft.wiki/images/BiomeSprite_<biome>.png
structures https://minecraft.wiki/images/EnvSprite_<structure>.png
(biome list https://minecraft.wiki/w/Java_Edition_data_values/Biomes)
entity faces https://minecraft.wiki/images/EntitySprite_<variant-entity>.png
https://minecraft.wiki/w/Template:EntitySprite
supports variants! variant 0 is just "axolotl", other variants use their variant name
*/

#[inline]
fn username_api_url(uuid: &str) -> String {
    format!("https://api.minecraftservices.com/minecraft/profile/lookup/{}", uuid)
}

#[inline]
fn face_api_url(uuid: &str) -> String {
    format!("https://mc-heads.net/avatar/{}/8", uuid)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to create HTTP client")
}

pub async fn fetch_username(uuid: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Profile {
        name: String,
    }

    let profile = client().get(&username_api_url(uuid)).send().await
        .context("Failed to send request to mojang")?
        .json::<Profile>().await
        .context("Failed to parse JSON")?;

    Ok(profile.name)
}

pub async fn fetch_user_face(uuid: &str) -> Result<Vec<u8>> {
    let response = client().get(&face_api_url(uuid)).send().await
        .context("Failed to send request to face API")?;

    if !response.status().is_success() {
        return Err(anyhow!("Face API returned error status: {}", response.status()));
    }
    
    let avatar_bytes = response.bytes().await
        .context("Failed to read avatar image bytes")?
        .to_vec();
    
    Ok(avatar_bytes)
}