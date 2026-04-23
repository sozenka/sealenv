use anyhow::Result;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_DIR: &str = ".sealenv";
const CONFIG_FILE: &str = ".sealenv/config.toml";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub active_profile: Option<String>,
    pub project_id: Option<String>,
}

fn generate_project_id() -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn load() -> Result<Config> {
    if !Path::new(CONFIG_FILE).exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(CONFIG_FILE)?;
    Ok(toml::from_str(&text)?)
}

fn save(cfg: &Config) -> Result<()> {
    let text = toml::to_string(cfg)?;
    fs::write(CONFIG_FILE, text)?;
    Ok(())
}

pub fn init() -> Result<()> {
    fs::create_dir_all(CONFIG_DIR)?;
    if !Path::new(CONFIG_FILE).exists() {
        let cfg = Config {
            active_profile: Some("dev".to_string()),
            project_id: Some(generate_project_id()),
        };
        save(&cfg)?;
    } else {
        // Backfill project_id for repos initialized before this field existed.
        let mut cfg = load()?;
        if cfg.project_id.is_none() {
            cfg.project_id = Some(generate_project_id());
            save(&cfg)?;
        }
    }
    Ok(())
}

pub fn get_active_profile() -> Option<String> {
    load().ok()?.active_profile
}

pub fn set_active_profile(name: &str) -> Result<()> {
    let mut cfg = load()?;
    cfg.active_profile = Some(name.to_string());
    save(&cfg)
}

pub fn get_project_id() -> Option<String> {
    load().ok()?.project_id
}
