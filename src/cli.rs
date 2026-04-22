use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "gitclaw",
    about = "Install software from GitHub releases",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        short,
        long,
        global = true,
        env = "GITHUB_TOKEN",
        help = "GitHub token for authentication (optional)"
    )]
    pub token: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install packages from GitHub releases
    #[command(about = "Install packages from GitHub releases")]
    Install {
        #[arg(num_args = 1.., help = "Package(s) to install (format: owner/repo or owner/repo@version)")]
        packages: Vec<String>,
        #[arg(short, long, help = "Force reinstall even if already installed")]
        force: bool,
        #[arg(long, help = "Show what would be installed without downloading")]
        dry_run: bool,
        #[arg(long, help = "Verify checksums after download")]
        verify: bool,
    },
    /// List installed packages
    #[command(about = "List installed packages")]
    List {
        #[arg(short, long, help = "Show detailed information")]
        verbose: bool,
    },
    /// Update installed packages
    #[command(about = "Update installed packages")]
    Update {
        #[arg(help = "Package to update (omit to update all)")]
        package: Option<String>,
    },
    /// Uninstall a package
    #[command(about = "Uninstall a package")]
    Uninstall {
        #[arg(help = "Package to uninstall (format: owner/repo)")]
        package: String,
    },
    /// Search for releases on GitHub
    #[command(about = "Search for releases on GitHub")]
    Search {
        #[arg(help = "Repository to search (format: owner/repo)")]
        package: String,
        #[arg(
            short,
            long,
            default_value = "10",
            help = "Maximum number of releases to show"
        )]
        limit: usize,
    },
    /// Generate shell completions
    #[command(about = "Generate shell completions")]
    Completions {
        #[arg(value_enum, help = "Shell to generate completions for")]
        shell: Shell,
    },
    /// Show platform information
    #[command(about = "Show platform information")]
    Platform {},
    /// Update gitclaw itself
    #[command(about = "Update gitclaw to the latest version")]
    SelfUpdate {
        /// Only check for updates, don't install
        #[arg(long, help = "Only check for updates, don't install")]
        check: bool,
    },
}
