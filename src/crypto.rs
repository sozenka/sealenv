use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ring::aead::{self, BoundKey, Nonce, NonceSequence, SealingKey, OpeningKey, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use std::fs;
use std::path::PathBuf;

use crate::config;

const ENV_FILE: &str = ".env";
const ENC_FILE: &str = ".env.enc";

// ── Key storage ──────────────────────────────────────────────────────────────

fn key_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let keys_dir = home.join(".sealenv").join("keys");
    fs::create_dir_all(&keys_dir)?;

    // Use stable project_id from .sealenv/config.toml when available.
    let project_id = if let Some(id) = config::get_project_id() {
        id
    } else {
        // Legacy fallback for repos initialized before project_id existed.
        let cwd = std::env::current_dir()?;
        cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string()
    };

    Ok(keys_dir.join(format!("{}.key", project_id)))
}

pub fn generate_key() -> Result<()> {
    let path = key_path()?;
    if path.exists() {
        println!("  Key already exists at {}", path.display());
        return Ok(());
    }
    let rng = SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes).map_err(|_| anyhow::anyhow!("Failed to generate random key"))?;
    fs::write(&path, &key_bytes)?;
    // Restrict permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    println!("  Key saved to {}", path.display());
    Ok(())
}

fn load_key() -> Result<[u8; 32]> {
    let path = key_path()?;
    let bytes = fs::read(&path)
        .with_context(|| format!("Key not found at {}. Run `sealenv init` first.", path.display()))?;
    if bytes.len() != 32 {
        anyhow::bail!("Key file is corrupt (expected 32 bytes)");
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

pub fn export_key() -> Result<String> {
    let key = load_key()?;
    Ok(B64.encode(key))
}

pub fn import_key(b64: &str) -> Result<()> {
    let bytes = B64.decode(b64.trim()).context("Invalid base64 key string")?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid key length (expected 32 bytes when decoded)");
    }
    let path = key_path()?;
    fs::write(&path, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

// ── Nonce helpers ─────────────────────────────────────────────────────────────

struct OneNonce(Option<aead::Nonce>);

impl NonceSequence for OneNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        self.0.take().ok_or(Unspecified)
    }
}

// ── Encrypt / Decrypt ────────────────────────────────────────────────────────

pub fn encrypt_env() -> Result<()> {
    let plaintext = fs::read(ENV_FILE)
        .with_context(|| format!("Could not read {}. Does it exist?", ENV_FILE))?;

    let key_bytes = load_key()?;
    let rng = SystemRandom::new();

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes).map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
    let mut sealing_key = SealingKey::new(unbound, OneNonce(Some(nonce)));

    let mut in_out = plaintext;

    sealing_key
        .seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

    // Output: [nonce (12 bytes)] + [ciphertext + tag]
    let mut output = Vec::with_capacity(NONCE_LEN + in_out.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&in_out);

    fs::write(ENC_FILE, &output)?;
    Ok(())
}

pub fn decrypt_env() -> Result<()> {
    let data = fs::read(ENC_FILE)
        .with_context(|| format!("Could not read {}. Run `sealenv encrypt` first.", ENC_FILE))?;

    if data.len() < NONCE_LEN {
        anyhow::bail!("Encrypted file is corrupt (too short)");
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_arr);

    let key_bytes = load_key()?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("Failed to create decryption key"))?;
    let mut opening_key = OpeningKey::new(unbound, OneNonce(Some(nonce)));

    let mut buf = ciphertext.to_vec();
    let plaintext = opening_key
        .open_in_place(aead::Aad::empty(), &mut buf)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or corrupt file"))?;

    fs::write(ENV_FILE, plaintext)?;
    Ok(())
}

// ── Load env vars from encrypted file (for `sealenv run`) ───────────────────────

pub fn load_vars_from_enc() -> Result<Vec<(String, String)>> {
    let data = fs::read(ENC_FILE)
        .with_context(|| "No .env.enc found. Run `sealenv encrypt` first.")?;

    if data.len() < NONCE_LEN {
        anyhow::bail!("Encrypted file is corrupt");
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_arr);

    let key_bytes = load_key()?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("Failed to create decryption key"))?;
    let mut opening_key = OpeningKey::new(unbound, OneNonce(Some(nonce)));

    let mut buf = ciphertext.to_vec();
    let plaintext = opening_key
        .open_in_place(aead::Aad::empty(), &mut buf)
        .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

    parse_env_bytes(plaintext)
}

pub fn parse_env_bytes(data: &[u8]) -> Result<Vec<(String, String)>> {
    let text = std::str::from_utf8(data).context("Env file is not valid UTF-8")?;
    let mut vars = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(idx) = line.find('=') {
            let key = line[..idx].trim().to_string();
            let val = line[idx + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
            vars.push((key, val));
        }
    }
    Ok(vars)
}
