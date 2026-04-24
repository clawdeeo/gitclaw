use clap::{CommandFactory, Parser};
use clap_complete::generate;

use crate::banner;
use crate::cli::{AliasAction, CacheAction, Cli, Commands};
use crate::config::Config;
use crate::constants::{APP_NAME_SHORT, GITCLAW_DIR};

pub async fn start() {
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

    if let Err(e) = dispatch(cli, config).await {
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

fn local_install_dir(config: &Config) -> anyhow::Result<Config> {
    let mut cfg = config.clone();
    cfg.install_dir = std::env::current_dir()?.join(GITCLAW_DIR);
    Ok(cfg)
}

async fn dispatch(cli: Cli, config: Config) -> anyhow::Result<()> {
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
                    crate::alias::handle_alias_add(&alias, &target, &config)?
                }

                AliasAction::Remove { alias } => {
                    crate::alias::handle_alias_remove(&alias, &config)?
                }

                AliasAction::List {} => crate::alias::handle_alias_list(&config)?,
            }
        }

        Commands::Cache { action } => {
            banner::print_output_header();

            match action {
                CacheAction::Clean {} => crate::cache::handle_cache_clean(&config)?,
                CacheAction::Size {} => crate::cache::handle_cache_size(&config)?,
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
                local_install_dir(&config)?
            } else {
                config.clone()
            };

            if locked {
                crate::lockfile::install_locked(&install_config).await?
            } else if packages.len() == 1 {
                crate::install::handle_install(
                    &packages[0],
                    force,
                    dry_run,
                    verify,
                    &install_config,
                    channel,
                )
                .await?
            } else {
                crate::install::handle_install_multiple(
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
            crate::lockfile::generate_lockfile(&config.install_dir, &dir)?
        }

        Commands::List { verbose, outdated } => {
            banner::print_output_header();

            if outdated {
                crate::registry::list_outdated(&config.install_dir, config.github_token.as_deref())
                    .await?
            } else {
                crate::registry::list_installed(verbose, &config.install_dir)?
            }
        }

        Commands::Update { package } => {
            banner::print_output_header();
            crate::install::handle_update(package.as_deref(), &config).await?
        }

        Commands::Uninstall { package, local } => {
            banner::print_output_header();

            let install_dir = if local {
                local_install_dir(&config)?.install_dir
            } else {
                config.install_dir.clone()
            };

            crate::registry::uninstall(&package, &install_dir, &config)?
        }

        Commands::Search {
            package,
            limit,
            channel,
        } => {
            banner::print_output_header();

            crate::github::search_releases(&package, limit, &config, channel).await?
        }

        Commands::Export {
            output: output_path,
        } => {
            banner::print_output_header();
            crate::export::handle_export(&config, output_path.as_deref())?
        }

        Commands::Import { file, force } => {
            banner::print_output_header();
            crate::export::handle_import(&config, &file, force).await?
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

            let arch = crate::platform::current_platform()?;
            banner::print_info(&format!("Detected platform: linux {}", arch));
            banner::print_info("Compiled for: Linux");
            banner::print_info(&format!("Architecture aliases: {:?}", arch.aliases()));
        }

        Commands::SelfUpdate { check } => {
            banner::print_output_header();

            if check {
                crate::updater::check_for_update(&config).await?
            } else {
                crate::updater::perform_update(&config).await?
            }
        }

        Commands::Run { package, args } => {
            banner::print_output_header();
            crate::run::handle_run(&package, args, &config).await?
        }
    }

    Ok(())
}
