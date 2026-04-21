use clap::Parser;

mod cli;
mod github;
mod install;
mod platform;
mod registry;
mod util;

// Extract module as a directory
mod extract;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to initialize color-eyre: {}", e);
        std::process::exit(1);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Install { package, force } => install::handle_install(&package, force).await?,
        Commands::List { verbose } => registry::list_installed(verbose)?,
        Commands::Update { package } => install::handle_update(package.as_deref()).await?,
        Commands::Uninstall { package } => registry::uninstall(&package)?,
        Commands::Search { package, limit } => github::search_releases(&package, limit).await?,
    }

    Ok(())
}
