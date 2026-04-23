use std::process::Command;

use anyhow::bail;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod cli;
mod core;
mod network;
mod output;

use cli::{AliasAction, CacheAction, Cli, Commands};
use core::config::Config;
use core::constants::{APP_NAME, APP_NAME_SHORT, DIR_BIN};
use core::registry::Registry;
use core::util::registry_path_from;

#[tokio::main]
async fn main() {
    if let Err(e) = color_eyre::install() {
        output::print_error(&format!("Failed to initialize color-eyre: {}.", e));
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
            output::print_error(&format!("Failed to load config: {}.", e));
            std::process::exit(1);
        }
    };

    let config = apply_cli_overrides(config, &cli);

    if let Err(e) = run(cli, config).await {
        output::print_error(&format!("{}", e));
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
        Commands::Cache { .. }
        | Commands::Export { .. }
        | Commands::Import { .. }
        | Commands::Install { .. }
        | Commands::Lock { .. }
        | Commands::List { .. }
        | Commands::Update { .. }
        | Commands::Uninstall { .. }
        | Commands::Search { .. }
        | Commands::Completions { .. }
        | Commands::Platform { .. }
        | Commands::SelfUpdate { .. }
        | Commands::Run { .. }
        | Commands::Alias { .. } => {
            output::print_version_line();
        }
    }

    match cli.command {
        Commands::Alias { action } => {
            output::print_output_header();
            match action {
                AliasAction::Add { alias, target } => {
                    core::alias::handle_alias_add(&alias, &target, &config)?
                }
                AliasAction::Remove { alias } => core::alias::handle_alias_remove(&alias, &config)?,
                AliasAction::List {} => core::alias::handle_alias_list(&config)?,
            }
        }
        Commands::Cache { action } => {
            output::print_output_header();
            match action {
                CacheAction::Clean {} => core::cache::handle_cache_clean(&config)?,
                CacheAction::Size {} => core::cache::handle_cache_size(&config)?,
            }
        }
        Commands::Install {
            packages,
            force,
            dry_run,
            verify,
            locked,
            local,
            channel,
        } => {
            output::print_output_header();

            let install_config = if local {
                let mut cfg = config.clone();
                cfg.install_dir = std::env::current_dir()?.join(".gitclaw");
                cfg
            } else {
                config.clone()
            };

            let ch = match channel.as_deref() {
                Some(c) => Some(c.parse::<core::channel::Channel>()?),
                None => None,
            };

            if locked {
                core::lockfile::install_locked(&install_config).await?
            } else if packages.len() == 1 {
                core::install::handle_install(
                    &packages[0],
                    force,
                    dry_run,
                    verify,
                    &install_config,
                    ch,
                )
                .await?
            } else {
                core::install::handle_install_multiple(
                    &packages,
                    force,
                    dry_run,
                    verify,
                    &install_config,
                    ch,
                )
                .await?
            }
        }

        Commands::Lock { dir } => {
            output::print_output_header();
            let project_dir = std::path::PathBuf::from(dir);
            core::lockfile::generate_lockfile(&config.install_dir, &project_dir)?
        }
        Commands::List { verbose, outdated } => {
            output::print_output_header();
            if outdated {
                core::registry::list_outdated(&config.install_dir, config.github_token.as_deref())
                    .await?
            } else {
                core::registry::list_installed(verbose, &config.install_dir)?
            }
        }

        Commands::Update { package } => {
            output::print_output_header();
            core::install::handle_update(package.as_deref(), &config).await?
        }

        Commands::Uninstall { package, local } => {
            output::print_output_header();
            let install_dir = if local {
                std::env::current_dir()?.join(".gitclaw")
            } else {
                config.install_dir.clone()
            };
            core::registry::uninstall(&package, &install_dir, &config)?
        }

        Commands::Search {
            package,
            limit,
            channel,
        } => {
            output::print_output_header();
            let ch = match channel.as_deref() {
                Some(c) => Some(c.parse::<core::channel::Channel>()?),
                None => None,
            };
            network::github::search_releases(&package, limit, &config, ch).await?
        }

        Commands::Export {
            output: output_path,
        } => {
            output::print_output_header();
            core::export::handle_export(&config, output_path.as_deref())?
        }

        Commands::Import { file, force } => {
            output::print_output_header();
            core::export::handle_import(&config, &file, force).await?
        }

        Commands::Completions { shell } => {
            output::print_output_header();
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name.clone(), &mut std::io::stdout());
            generate(
                shell,
                &mut Cli::command(),
                APP_NAME_SHORT,
                &mut std::io::stdout(),
            );
        }

        Commands::Platform {} => {
            output::print_output_header();
            let arch = network::platform::current_platform();
            output::print_info(&format!("Detected platform: linux {}", arch));
            output::print_info("Compiled for: Linux");
            output::print_info(&format!("Architecture aliases: {:?}", arch.aliases()));
        }

        Commands::SelfUpdate { check } => {
            output::print_output_header();

            if check {
                core::updater::check_for_update(&config).await?
            } else {
                core::updater::perform_update(&config).await?
            }
        }

        Commands::Run { package, args } => {
            output::print_output_header();
            run_package(&package, args, &config).await?
        }
    }

    Ok(())
}

async fn run_package(package: &str, args: Vec<String>, config: &Config) -> anyhow::Result<()> {
    let resolved = if !package.contains('/') {
        if let Some(alias_target) = crate::core::alias::AliasMap::load(config)?.resolve(package) {
            output::print_info(&format!("Alias '{}' -> '{}'.", package, alias_target));
            alias_target.to_string()
        } else {
            package.to_string()
        }
    } else {
        package.to_string()
    };

    let (owner, repo) = if resolved.contains('/') {
        let parts: Vec<&str> = resolved.split('/').collect();
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
            .filter(|p| p.repo == resolved)
            .collect();

        match matches.len() {
            0 => bail!(
                "Package '{}' not installed. Use '{} install owner/{}' first.",
                resolved,
                APP_NAME,
                resolved
            ),
            1 => (matches[0].owner.clone(), matches[0].repo.clone()),
            _ => bail!(
                "Multiple packages named '{}'. Use full name (owner/repo).",
                resolved
            ),
        }
    };

    let binary_path = config.install_dir.join(DIR_BIN).join(&repo);

    if !binary_path.exists() {
        bail!(
            "Binary for '{}/{}' not found at {}.",
            owner,
            repo,
            binary_path.display()
        );
    }

    let status = Command::new(&binary_path).args(args).status()?;

    if !status.success() {
        bail!("Process exited with code: {:?}.", status.code());
    }

    Ok(())
}
