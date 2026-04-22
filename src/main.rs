use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod banner;
mod checksum;
mod cli;
mod config;
mod github;
mod install;
mod platform;
mod registry;
mod self_update;
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

    // Check for platform mismatch (e.g., Darwin binary on Linux)
    if let Some(warning) = platform::check_target_mismatch() {
        eprintln!("{}", warning);
    }

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
    // Show banner for certain commands
    match &cli.command {
        Commands::Install { .. } | Commands::List { .. } | Commands::Platform { .. } | Commands::SelfUpdate { .. } => {
            banner::print_banner();
        }
        _ => {}
    }

    match cli.command {
        Commands::Install {
            packages,
            force,
            dry_run,
            verify,
        } => {
            if packages.len() == 1 {
                install::handle_install(&packages[0], force, dry_run, verify, &config).await?
            } else {
                install::handle_install_multiple(&packages, force, dry_run, verify, &config).await?
            }
        }
        Commands::List { verbose } => registry::list_installed(verbose)?,
        Commands::Update { package } => install::handle_update(package.as_deref(), &config).await?,
        Commands::Uninstall { package } => registry::uninstall(&package, &config.install_dir)?,
        Commands::Search { package, limit } => {
            github::search_releases(&package, limit, &config).await?
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Platform {} => {
            let (os, arch) = gitclaw::platform::current_platform()?;
            println!("Detected platform: {} {}", os, arch);
            #[cfg(target_os = "macos")]
            println!("Compiled for: macOS");
            #[cfg(target_os = "linux")]
            println!("Compiled for: Linux");
            #[cfg(target_os = "windows")]
            println!("Compiled for: Windows");
            println!("OS aliases: {:?}", os.aliases());
            println!("Arch aliases: {:?}", arch.aliases());
            if let Some(warning) = gitclaw::platform::check_target_mismatch() {
                println!("\n{}", warning);
            }
        }
        Commands::SelfUpdate { check } => {
            if check {
                self_update::check_for_update(&config).await?
            } else {
                self_update::perform_update(&config).await?
            }
        }
    }

    Ok(())
}
