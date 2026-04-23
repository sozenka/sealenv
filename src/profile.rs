use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;
use crate::crypto;

fn profiles_dir() -> Result<PathBuf> {
    let dir = Path::new(".sealenv").join("profiles");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn profile_path(name: &str) -> Result<PathBuf> {
    Ok(profiles_dir()?.join(format!("{}.env", name)))
}

pub fn create(name: &str) -> Result<()> {
    let path = profile_path(name)?;
    if path.exists() {
        anyhow::bail!("Profile '{}' already exists", name);
    }
    // Copy current .env if it exists, otherwise create empty
    if Path::new(".env").exists() {
        fs::copy(".env", &path)?;
    } else {
        fs::write(&path, b"")?;
    }
    Ok(())
}

pub fn switch_to(name: &str) -> Result<()> {
    let path = profile_path(name)?;
    if !path.exists() {
        anyhow::bail!(
            "Profile '{}' not found. Run `sealenv profile list` to see available profiles.",
            name
        );
    }
    fs::copy(&path, ".env")?;
    config::set_active_profile(name)?;
    Ok(())
}

pub fn list() -> Result<()> {
    let dir = profiles_dir()?;
    let active = config::get_active_profile().unwrap_or_default();

    let mut profiles: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if name.ends_with(".env") {
                Some(name.trim_end_matches(".env").to_string())
            } else {
                None
            }
        })
        .collect();

    profiles.sort();

    if profiles.is_empty() {
        println!("{}", "No profiles yet. Run `sealenv profile create <name>`".yellow());
        return Ok(());
    }

    println!("{}", "Profiles:".bold());
    for p in &profiles {
        if *p == active {
            println!("  {} {}", "▶".green().bold(), p.green().bold());
        } else {
            println!("  {} {}", " ", p);
        }
    }
    Ok(())
}

pub fn add_entry(entry: &str) -> Result<()> {
    let active = config::get_active_profile().unwrap_or_else(|| "default".to_string());
    let path = profile_path(&active)?;

    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let key = entry.split('=').next().unwrap_or("");
    // Remove existing line with same key
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.starts_with(&format!("{}=", key)))
        .collect();
    content = lines.join("\n");
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(entry);
    content.push('\n');

    fs::write(&path, &content)?;

    // Also update .env
    let env_content = fs::read_to_string(".env").unwrap_or_default();
    let env_lines: Vec<&str> = env_content
        .lines()
        .filter(|l| !l.starts_with(&format!("{}=", key)))
        .collect();
    let mut new_env = env_lines.join("\n");
    if !new_env.is_empty() && !new_env.ends_with('\n') {
        new_env.push('\n');
    }
    new_env.push_str(entry);
    new_env.push('\n');
    fs::write(".env", &new_env)?;

    Ok(())
}

pub fn diff(a: &str, b: &str) -> Result<()> {
    let path_a = profile_path(a)?;
    let path_b = profile_path(b)?;

    let content_a = fs::read_to_string(&path_a)
        .with_context(|| format!("Profile '{}' not found", a))?;
    let content_b = fs::read_to_string(&path_b)
        .with_context(|| format!("Profile '{}' not found", b))?;

    let vars_a = crypto::parse_env_bytes(content_a.as_bytes())?;
    let vars_b = crypto::parse_env_bytes(content_b.as_bytes())?;

    use std::collections::HashMap;
    let map_a: HashMap<_, _> = vars_a.iter().cloned().collect();
    let map_b: HashMap<_, _> = vars_b.iter().cloned().collect();

    println!(
        "{} {} {} {}",
        "diff".bold(),
        a.red().bold(),
        "→".dimmed(),
        b.green().bold()
    );
    println!("{}", "─".repeat(40).dimmed());

    let mut any = false;

    // Keys in A but not B
    for (k, _v) in &map_a {
        if !map_b.contains_key(k.as_str()) {
            println!("  {} {}", "−".red().bold(), k.red());
            any = true;
        }
    }

    // Keys in B but not A
    for (k, _v) in &map_b {
        if !map_a.contains_key(k.as_str()) {
            println!("  {} {}", "+".green().bold(), k.green());
            any = true;
        }
    }

    // Keys in both but different values
    for (k, v_a) in &map_a {
        if let Some(v_b) = map_b.get(k.as_str()) {
            if v_a != v_b {
                println!("  {} {} {}", "~".yellow().bold(), k.yellow(), "(value changed)".dimmed());
                any = true;
            }
        }
    }

    if !any {
        println!("  {}", "Profiles are identical".dimmed());
    }

    Ok(())
}
