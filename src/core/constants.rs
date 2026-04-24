pub const APP_NAME: &str = "gitclaw";
pub const APP_NAME_SHORT: &str = "gcw";
pub const REPO_OWNER: &str = "clawdeeo";
pub const REPO_NAME: &str = "gitclaw";

pub const GITCLAW_DIR: &str = ".gitclaw";
pub const REGISTRY_FILE: &str = "registry.toml";
pub const CONFIG_FILE: &str = "config.toml";
pub const LOCAL_CONFIG_FILE: &str = ".gitclaw.toml";
pub const XDG_CONFIG_SUBDIR: &str = "gitclaw";
pub const ENV_VAR_TOKEN: &str = "GITHUB_TOKEN";
pub const ENV_VAR_CONFIG: &str = "GITCLAW_CONFIG";
pub const ALIASES_FILE: &str = "aliases.toml";
pub const LOCKFILE_NAME: &str = "gitclaw.lock";

pub const DIR_BIN: &str = "bin";
pub const DIR_PACKAGES: &str = "packages";
pub const DIR_CACHE: &str = "cache";
pub const DIR_DOWNLOADS: &str = "downloads";
pub const DIR_EXTRACTED: &str = "extracted";

pub const TEMP_DIR_PREFIX: &str = "gitclaw-";
pub const TEMP_DIR_SELF_UPDATE: &str = "gitclaw-self-update";

pub const GITHUB_API_BASE: &str = "https://api.github.com";
pub const RELEASE_TAG_LATEST: &str = "latest";

pub const KV_KEY_WIDTH: usize = 20;
pub const SEARCH_LIMIT_DEFAULT: &str = "10";
pub const SEARCH_LIMIT_MAX: usize = 100;
pub const EXEC_PERMISSION_MODE: u32 = 0o755;
pub const EXEC_PERMISSION_BITS: u32 = 0o111;
pub const DATE_PREFIX_LEN: usize = 10;

pub const SCORE_PLATFORM_MATCH: i32 = 10;
pub const SCORE_LINUX_PARTIAL: i32 = 5;
pub const SCORE_KNOWN_EXTENSION: i32 = 5;
pub const SCORE_SHELL_SCRIPT: i32 = 2;
pub const SCORE_CHECKSUM_PENALTY: i32 = -100;
