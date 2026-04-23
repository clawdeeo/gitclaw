use clap::{Parser, Subcommand};
use clap_complete::Shell;

use crate::core::constants::{APP_NAME, ENV_VAR_TOKEN};

#[derive(Parser)]
#[command(
    name = APP_NAME,
    about = "Install software from GitHub releases.",
    version,
    before_help = crate::output::BANNER
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        short,
        long,
        global = true,
        env = ENV_VAR_TOKEN,
        help = "GitHub token for authentication (optional)."
    )]
    pub token: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Install packages from GitHub releases.")]
    Install {
        #[arg(num_args = 1.., help = "Package(s) to install (format: owner/repo or owner/repo@version).")]
        packages: Vec<String>,
        #[arg(short, long, help = "Force reinstall even if already installed.")]
        force: bool,
        #[arg(long, help = "Show what would be installed without downloading.")]
        dry_run: bool,
        #[arg(long, help = "Verify checksums after download.")]
        verify: bool,
    },
    #[command(about = "List installed packages.")]
    List {
        #[arg(short, long, help = "Show detailed information.")]
        verbose: bool,
    },
    #[command(about = "Update installed packages.")]
    Update {
        #[arg(help = "Package to update (omit to update all).")]
        package: Option<String>,
    },
    #[command(about = "Uninstall a package.")]
    Uninstall {
        #[arg(help = "Package to uninstall (format: owner/repo or identifier).")]
        package: String,
    },
    #[command(about = "Search for releases on GitHub.")]
    Search {
        #[arg(help = "Repository to search (format: owner/repo).")]
        package: String,
        #[arg(
            short,
            long,
            default_value = "10",
            help = "Maximum number of releases to show."
        )]
        limit: usize,
    },
    #[command(about = "Generate shell completions.")]
    Completions {
        #[arg(value_enum, help = "Shell to generate completions for.")]
        shell: Shell,
    },
    #[command(about = "Show platform information.")]
    Platform {},
    #[command(name = "self", about = "Update gitclaw to the latest version.")]
    SelfUpdate {
        #[arg(long, help = "Only check for updates, do not install.")]
        check: bool,
    },
    #[command(about = "Run an installed package.")]
    Run {
        #[arg(help = "Package to run (format: owner/repo or identifier).")]
        package: String,
        #[arg(trailing_var_arg = true, help = "Arguments to pass to the package.")]
        args: Vec<String>,
    },
}
