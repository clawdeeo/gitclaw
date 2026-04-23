use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod banner;
mod checksum;
mod cli;
mod config;
mod extract;
mod github;
mod install;
mod platform;
mod registry;
mod self_update;
mod util;

use anyhow::bail;
use cli::{Cli, Commands};
use config::Config;
use registry::Registry;
use util::registry_path_from;

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

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let config = apply_cli_overrides(config, &cli);

    if let Err(e) = run(cli, config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Config {
    if cli.token.is_some() {
        config.github_token = cli.token.clone();
    }
    config
}

async fn run(cli: Cli, config: Config) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Completions { .. } => {}
        Commands::Install { .. }
        | Commands::List { .. }
        | Commands::Update { .. }
        | Commands::Uninstall { .. }
        | Commands::Search { .. }
        | Commands::Platform { .. }
        | Commands::SelfUpdate { .. }
        | Commands::Run { .. } => {
            banner::print_version_line();
        }
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

        Commands::List { verbose } => {
            banner::print_output_header();
            registry::list_installed(verbose, &config.install_dir)?
        }

        Commands::Update { package } => {
            banner::print_output_header();
            install::handle_update(package.as_deref(), &config).await?
        }

        Commands::Uninstall { package } => {
            banner::print_output_header();
            registry::uninstall(&package, &config.install_dir)?
        }

        Commands::Search { package, limit } => {
            banner::print_output_header();
            github::search_releases(&package, limit, &config).await?
        }

        Commands::Completions { shell } => {
            banner::print_output_header();
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }

        Commands::Platform {} => {
            banner::print_output_header();
            let arch = platform::current_platform();
            banner::print_info(&format!("Detected platform: linux {}", arch));
            banner::print_info(&format!("Compiled for: Linux"));
            banner::print_info(&format!("Architecture aliases: {:?}", arch.aliases()));
        }

        Commands::SelfUpdate { check } => {
            banner::print_output_header();

            if check {
                self_update::check_for_update(&config).await?
            } else {
                self_update::perform_update(&config).await?
            }
        }

        Commands::Run { package, args } => {
            banner::print_output_header();
            run_package(&package, args, &config).await?
        }
    }

    Ok(())
}

async fn run_package(package: &str, args: Vec<String>, config: &Config) -> anyhow::Result<()> {
    let (owner, repo) = if package.contains('/') {
        let parts: Vec<&str> = package.split('/').collect();
        if parts.len() != 2 {
            bail!("Invalid package format. Use 'owner/repo' or just 'repo'.");
        }
        (parts[0].to_string(), parts[1].to_string())
    } else {
        let registry_path = registry_path_from(&config.install_dir);
        let reg = Registry::load_from(&registry_path)?;

        let matches: Vec<_> = reg
            .packages
            .values()
            .filter(|p| p.repo == package)
            .collect();

        match matches.len() {
            0 => bail!(
                "Package '{}' not installed. Use 'gitclaw install owner/{}' first.",
                package,
                package
            ),
            1 => (matches[0].owner.clone(), matches[0].repo.clone()),
            _ => bail!(
                "Multiple packages named '{}'. Use full name (owner/repo).",
                package
            ),
        }
    };

    let binary_path = config.install_dir.join("bin").join(&repo);

    if !binary_path.exists() {
        bail!(
            "Binary for '{}/{}' not found at {}",
            owner,
            repo,
            binary_path.display()
        );
    }

    use std::process::Command;
    let status = Command::new(&binary_path).args(args).status()?;

    if !status.success() {
        bail!("Process exited with code: {:?}", status.code());
    }

    Ok(())
}
