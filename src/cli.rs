use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub global: bool,

    #[command(subcommand)]
    pub action: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        short_cmd: String,
        #[arg(required = true, num_args = 1..)]
        long_cmd: Vec<String>,
    },
    Remove {
        short_cmd: String,
    },

    Show {
        short_cmd: String,
    },

    List {},

    #[command(external_subcommand)]
    External(Vec<String>),
}

/// Alias che non possono essere usati perché collidono con i sottocomandi
/// (sarebbero irraggiungibili via `External`).
pub const RESERVED_ALIASES: &[&str] = &["add", "remove", "show", "list", "help"];

pub fn validate_alias(alias: &str) -> Result<(), String> {
    if alias.trim().is_empty() {
        return Err("alias must not be empty".to_string());
    }
    if RESERVED_ALIASES.contains(&alias) {
        return Err(format!(
            "alias '{alias}' is reserved and cannot be used"
        ));
    }
    Ok(())
}
