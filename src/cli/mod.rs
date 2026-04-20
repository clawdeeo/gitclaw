use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitclaw", about = "Install software from GitHub releases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub token: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Install {
        package: String,
        #[arg(short, long)]
        force: bool,
    },
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    Update {
        package: Option<String>,
    },
    Uninstall {
        package: String,
    },
    Search {
        package: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}