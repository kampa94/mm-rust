use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(short, long, global = false)]
    pub global: bool,

    #[command(subcommand)]
    pub action: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        short_cmd: Option<String>,
        long_cmd: Option<Vec<String>>,
    },
    Remove {
        short_cmd: Option<String>,
    },

    Show {
        short_cmd: Option<String>,
    },

    List {},

    #[command(external_subcommand)]
    External(Vec<String>),

    #[command(hide = true)]
    Complete,
}
