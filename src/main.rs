mod crypto;
mod profile;
mod config;
mod gitignore;
mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(
    name = "sealenv",
    about = "Encrypt .env files and switch profiles. One binary, zero setup.",
    version = "0.1.0",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize sealenv in this project
    Init,

    /// Encrypt .env → .env.enc (safe to commit)
    Encrypt,

    /// Decrypt .env.enc → .env (local only)
    Decrypt,

    /// Add a key=value to the current profile
    Add {
        /// KEY=VALUE pair to add
        entry: String,
    },

    /// Manage profiles (create, use, list)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Run a command with env vars injected (no .env file written)
    Run {
        /// Command and arguments to run
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Show diff between two profiles
    Diff {
        /// First profile name
        profile_a: String,
        /// Second profile name
        profile_b: String,
    },

    /// Manage encryption keys
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Create a new profile
    Create { name: String },
    /// Switch to a profile
    Use { name: String },
    /// List all profiles
    List,
}

#[derive(Subcommand)]
enum KeyAction {
    /// Export the key for this project (share with teammates)
    Export,
    /// Import a key from another machine
    Import {
        /// The base64-encoded key string
        key: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            config::init()?;
            gitignore::setup()?;
            crypto::generate_key()?;
            println!("{}", "✓ sealenv initialized".green().bold());
            println!("  .env and key added to .gitignore");
            println!("  Key stored in ~/.sealenv/keys/");
            println!("  Run {} to create your first encrypted file", "sealenv encrypt".cyan());
        }

        Commands::Encrypt => {
            crypto::encrypt_env()?;
            println!("{}", "✓ .env encrypted → .env.enc".green().bold());
            println!("  Safe to commit .env.enc to git");
        }

        Commands::Decrypt => {
            crypto::decrypt_env()?;
            println!("{}", "✓ .env.enc decrypted → .env".green().bold());
        }

        Commands::Add { entry } => {
            if !entry.contains('=') {
                anyhow::bail!("Entry must be in KEY=VALUE format, got: {}", entry);
            }
            profile::add_entry(&entry)?;
            let key = entry.split('=').next().unwrap_or(&entry);
            println!("{} {} added to current profile", "✓".green().bold(), key.cyan());
        }

        Commands::Profile { action } => match action {
            ProfileAction::Create { name } => {
                profile::create(&name)?;
                println!("{} profile {} created", "✓".green().bold(), name.cyan());
            }
            ProfileAction::Use { name } => {
                profile::switch_to(&name)?;
                println!("{} switched to profile {}", "✓".green().bold(), name.cyan());
            }
            ProfileAction::List => {
                profile::list()?;
            }
        },

        Commands::Run { command } => {
            if command.is_empty() {
                anyhow::bail!("No command provided. Usage: sealenv run -- npm start");
            }
            run::inject_and_run(&command)?;
        }

        Commands::Diff { profile_a, profile_b } => {
            profile::diff(&profile_a, &profile_b)?;
        }

        Commands::Key { action } => match action {
            KeyAction::Export => {
                let key = crypto::export_key()?;
                println!("{}", "✓ Your project key (share securely with teammates):".green().bold());
                println!("\n  {}\n", key.yellow());
                println!("  {}", "⚠  Never share this in chat, email, or git.".red());
            }
            KeyAction::Import { key } => {
                crypto::import_key(&key)?;
                println!("{}", "✓ Key imported successfully".green().bold());
            }
        },
    }

    Ok(())
}
