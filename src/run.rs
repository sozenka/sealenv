use anyhow::{Context, Result};
use std::process::Command;

use crate::crypto;

pub fn inject_and_run(args: &[String]) -> Result<()> {
    let vars = crypto::load_vars_from_enc()
        .context("Could not load env vars. Run `sealenv encrypt` first.")?;

    let (program, rest) = args.split_first().context("No command provided")?;

    let status = Command::new(program)
        .args(rest)
        .envs(vars)
        .status()
        .with_context(|| format!("Failed to run command: {}", program))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}
