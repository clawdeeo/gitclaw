use std::process::Command;

use anyhow::bail;

use crate::config::Config;
use crate::constants::{APP_NAME, DIR_BIN};
use crate::registry::Registry;
use crate::util::registry_path_from;

pub async fn handle_run(package: &str, args: Vec<String>, config: &Config) -> anyhow::Result<()> {
    let resolved = crate::alias::resolve_package_input(package, config)?;

    let (owner, repo) = if resolved.contains('/') {
        let (o, r, _) = crate::github::parse_package(&resolved)?;
        (o, r)
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
