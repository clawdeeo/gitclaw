use clap::Parser;

mod cli;
mod config;
mod github;
mod install;
mod platform;
mod registry;
mod util;

// Extract module as a directory
mod extract;

use cli::{Cli, Commands};
use config::Config;

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

    // Load configuration and merge with CLI args
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Apply CLI overrides to config
    let config = apply_cli_overrides(config, &cli);

    if let Err(e) = run(cli, config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Apply CLI argument overrides to the loaded config
fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Config {
    // CLI token takes precedence over config file
    if cli.token.is_some() {
        config.github_token = cli.token.clone();
    }
    config
}

async fn run(cli: Cli, config: Config) -> anyhow::Result<()> {
    match cli.command {
        Commands::Install { package, force } => {
            install::handle_install(&package, force, &config).await?
        }
        Commands::List { verbose } => registry::list_installed(verbose)?,
        Commands::Update { package } => install::handle_update(package.as_deref(), &config).await?,
        Commands::Uninstall { package } => registry::uninstall(&package)?,
        Commands::Search { package, limit } => {
            github::search_releases(&package, limit, &config).await?
        }
    }

    Ok(())
}
