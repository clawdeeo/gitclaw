use anyhow::Result;
use clap::Parser;

mod cli;
mod extract;
mod github;
mod install;
mod platform;
mod registry;
mod util;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install { package, force } => install::handle_install(&package, force).await?,
        Commands::List { verbose } => registry::list_installed(verbose)?,
        Commands::Update { package } => install::handle_update(package.as_deref()).await?,
        Commands::Uninstall { package } => registry::uninstall(&package)?,
        Commands::Search { package, limit } => github::search_releases(&package, limit).await?,
    }

    Ok(())
}
