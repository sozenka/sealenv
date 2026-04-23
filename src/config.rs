use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_DIR: &str = ".sealenv";
const CONFIG_FILE: &str = ".sealenv/config.toml";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub active_profile: Option<String>,
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
        };
        save(&cfg)?;
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
