use anyhow::Result;
use std::fs;
use std::path::Path;

const ENTRIES: &[&str] = &[
    ".env",
    "*.env.local",
    ".sealenv/keys/",
];

pub fn setup() -> Result<()> {
    let path = Path::new(".gitignore");
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    let mut added = Vec::new();

    for entry in ENTRIES {
        if !content.lines().any(|l| l.trim() == *entry) {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            added.push(*entry);
        }
    }

    if !added.is_empty() {
        fs::write(path, &content)?;
        for e in added {
            println!("  Added {} to .gitignore", e);
        }
    }

    Ok(())
}
