use std::process::Command;

use anyhow::bail;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use gitclaw::banner;
use gitclaw::cli::{AliasAction, CacheAction, Cli, Commands};
use gitclaw::config::Config;
use gitclaw::constants::{APP_NAME, APP_NAME_SHORT, DIR_BIN};
use gitclaw::registry::Registry;
use gitclaw::util::registry_path_from;

#[tokio::main]
async fn main() {
    if let Err(e) = color_eyre::install() {
        banner::print_error(&format!("Failed to initialize color-eyre: {}.", e));
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
            banner::print_error(&format!("Failed to load config: {}.", e));
            std::process::exit(1);
        }
    };

    let config = apply_cli_overrides(config, &cli);

    if let Err(e) = run(cli, config).await {
        banner::print_error(&format!("{}", e));
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
            banner::print_version_line();
        }
    }

    match cli.command {
        Commands::Alias { action } => {
            banner::print_output_header();

            match action {
                AliasAction::Add { alias, target } => {
                    gitclaw::alias::handle_alias_add(&alias, &target, &config)?
                }
                AliasAction::Remove { alias } => {
                    gitclaw::alias::handle_alias_remove(&alias, &config)?
                }
                AliasAction::List {} => gitclaw::alias::handle_alias_list(&config)?,
            }
        }

        Commands::Cache { action } => {
            banner::print_output_header();

            match action {
                CacheAction::Clean {} => gitclaw::cache::handle_cache_clean(&config)?,
                CacheAction::Size {} => gitclaw::cache::handle_cache_size(&config)?,
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
            banner::print_output_header();

            let install_config = if local {
                let mut cfg = config.clone();
                cfg.install_dir = std::env::current_dir()?.join(".gitclaw");
                cfg
            } else {
                config.clone()
            };

            let channel = match channel.as_deref() {
                Some(c) => Some(c.parse::<gitclaw::channel::Channel>()?),
                None => None,
            };

            if locked {
                gitclaw::lockfile::install_locked(&install_config).await?
            } else if packages.len() == 1 {
                gitclaw::install::handle_install(
                    &packages[0],
                    force,
                    dry_run,
                    verify,
                    &install_config,
                    channel,
                )
                .await?
            } else {
                gitclaw::install::handle_install_multiple(
                    &packages,
                    force,
                    dry_run,
                    verify,
                    &install_config,
                    channel,
                )
                .await?
            }
        }

        Commands::Lock { dir } => {
            banner::print_output_header();
            let project_dir = std::path::PathBuf::from(dir);
            gitclaw::lockfile::generate_lockfile(&config.install_dir, &project_dir)?
        }

        Commands::List { verbose, outdated } => {
            banner::print_output_header();

            if outdated {
                gitclaw::registry::list_outdated(
                    &config.install_dir,
                    config.github_token.as_deref(),
                )
                .await?
            } else {
                gitclaw::registry::list_installed(verbose, &config.install_dir)?
            }
        }

        Commands::Update { package } => {
            banner::print_output_header();
            gitclaw::install::handle_update(package.as_deref(), &config).await?
        }

        Commands::Uninstall { package, local } => {
            banner::print_output_header();

            let install_dir = if local {
                std::env::current_dir()?.join(".gitclaw")
            } else {
                config.install_dir.clone()
            };

            gitclaw::registry::uninstall(&package, &install_dir, &config)?
        }

        Commands::Search {
            package,
            limit,
            channel,
        } => {
            banner::print_output_header();

            let channel = match channel.as_deref() {
                Some(c) => Some(c.parse::<gitclaw::channel::Channel>()?),
                None => None,
            };

            gitclaw::github::search_releases(&package, limit, &config, channel).await?
        }

        Commands::Export {
            output: output_path,
        } => {
            banner::print_output_header();
            gitclaw::export::handle_export(&config, output_path.as_deref())?
        }

        Commands::Import { file, force } => {
            banner::print_output_header();
            gitclaw::export::handle_import(&config, &file, force).await?
        }

        Commands::Completions { shell } => {
            banner::print_output_header();

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
            banner::print_output_header();

            let arch = gitclaw::platform::current_platform();
            banner::print_info(&format!("Detected platform: linux {}", arch));
            banner::print_info("Compiled for: Linux");
            banner::print_info(&format!("Architecture aliases: {:?}", arch.aliases()));
        }

        Commands::SelfUpdate { check } => {
            banner::print_output_header();

            if check {
                gitclaw::updater::check_for_update(&config).await?
            } else {
                gitclaw::updater::perform_update(&config).await?
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
    let resolved = if !package.contains('/') {
        if let Some(alias_target) = gitclaw::alias::AliasMap::load(config)?.resolve(package) {
            banner::print_info(&format!("Alias '{}' -> '{}'.", package, alias_target));
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
