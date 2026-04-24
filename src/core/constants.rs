pub const APP_NAME: &str = "gitclaw";
pub const APP_NAME_SHORT: &str = "gcw";
pub const REPO_OWNER: &str = "airscripts";
pub const REPO_NAME: &str = "gitclaw";

pub const GITCLAW_DIR: &str = ".gitclaw";
pub const REGISTRY_FILE: &str = "registry.toml";
pub const CONFIG_FILE: &str = "config.toml";
pub const LOCAL_CONFIG_FILE: &str = ".gitclaw.toml";
pub const LEGACY_HOME_CONFIG_FILE: &str = ".gitclaw.toml";
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
pub const GITHUB_URL_PREFIX: &str = "https://github.com/";
pub const GITHUB_URL_PREFIX_SHORT: &str = "github.com/";
pub const GITHUB_API_PATH_RELEASES_TAG: &str = "/repos/{}/{}/releases/tags/{}";
pub const GITHUB_API_PATH_RELEASES_LATEST: &str = "/repos/{}/{}/releases/latest";
pub const GITHUB_API_PATH_RELEASES: &str = "/repos/{}/{}/releases";
pub const GITHUB_API_PATH_RELEASES_PAGED: &str = "/repos/{}/{}/releases?per_page={}";

pub const RELEASE_TAG_LATEST: &str = "latest";

pub const HTTP_HEADER_LINK: &str = "link";
pub const HTTP_LINK_REL_NEXT: &str = "rel=\"next\"";

pub const KV_KEY_WIDTH: usize = 20;
pub const SEARCH_LIMIT_DEFAULT: &str = "10";
pub const SEARCH_LIMIT_MAX: usize = 100;
pub const EXEC_PERMISSION_MODE: u32 = 0o755;
pub const EXEC_PERMISSION_BITS: u32 = 0o111;
pub const DATE_PREFIX_LEN: usize = 10;
pub const WALK_MAX_DEPTH: usize = 3;
pub const BYTES_PER_KIB: u64 = 1024;
pub const UPDATER_BACKUP_EXT: &str = "backup";

pub const SCORE_PLATFORM_MATCH: i32 = 10;
pub const SCORE_LINUX_PARTIAL: i32 = 5;
pub const SCORE_KNOWN_EXTENSION: i32 = 5;
pub const SCORE_SHELL_SCRIPT: i32 = 2;
pub const SCORE_CHECKSUM_PENALTY: i32 = -100;

pub const EXT_TAR_GZ: &str = ".tar.gz";
pub const EXT_TGZ: &str = ".tgz";
pub const EXT_ZIP: &str = ".zip";
pub const EXT_TAR_BZ2: &str = ".tar.bz2";
pub const EXT_TBZ2: &str = ".tbz2";
pub const EXT_TAR_XZ: &str = ".tar.xz";
pub const EXT_TXZ: &str = ".txz";
pub const EXT_TAR_ZST: &str = ".tar.zst";
pub const EXT_TZST: &str = ".tzst";
pub const EXT_TAR: &str = ".tar";
pub const EXT_DEB: &str = ".deb";
pub const EXT_BIN: &str = ".bin";
pub const EXT_APPIMAGE: &str = ".appimage";
pub const EXT_RPM: &str = ".rpm";
pub const EXT_SH: &str = ".sh";

pub const EXT_SHA256: &str = ".sha256";
pub const EXT_SHA512: &str = ".sha512";
pub const EXT_MD5: &str = ".md5";
pub const EXT_SHA: &str = ".sha";
pub const EXT_SIG: &str = ".sig";
pub const EXT_ASC: &str = ".asc";
pub const EXT_CHECKSUM: &str = ".checksum";
pub const STR_CHECKSUM: &str = "checksum";
pub const STR_SHA256SUM: &str = "sha256sum";
pub const STR_SHA512SUM: &str = "sha512sum";

pub const DEB_DATA_TAR_GZ: &str = "data.tar.gz";
pub const DEB_DATA_TAR_XZ: &str = "data.tar.xz";
pub const DEB_DATA_TAR_BZ2: &str = "data.tar.bz2";
pub const DEB_DATA_TAR_ZST: &str = "data.tar.zst";
pub const DEB_DATA_TAR: &str = "data.tar";

pub const COL_SEARCH_TAG: usize = 20;
pub const COL_SEARCH_NAME: usize = 42;
pub const COL_SEARCH_ASSETS: usize = 8;
pub const COL_SEARCH_NAME_MAX: usize = 40;
pub const COL_SEARCH_NAME_TRUNCATE: usize = 37;

pub const COL_LIST_PACKAGE: usize = 25;
pub const COL_LIST_IDENTIFIER: usize = 20;
pub const COL_LIST_VERSION: usize = 15;
pub const COL_LIST_PATH: usize = 30;
pub const COL_LIST_PATH_MAX: usize = 28;
pub const COL_LIST_PATH_TRUNCATE: usize = 25;

pub const ARCH_X86_64: &str = "x86_64";
pub const ARCH_AARCH64: &str = "aarch64";
pub const ARCH_AMD64: &str = "amd64";
pub const ARCH_ARM64: &str = "arm64";
pub const ARCH_X64: &str = "x64";

pub const PLATFORM_LINUX: &str = "linux";
pub const PLATFORM_LINUX_X86_64: &str = "linux-x86_64";
pub const PLATFORM_LINUX_AMD64: &str = "linux-amd64";
pub const PLATFORM_LINUX_X64: &str = "linux-x64";
pub const PLATFORM_LINUX_AARCH64: &str = "linux-aarch64";
pub const PLATFORM_LINUX_ARM64: &str = "linux-arm64";
pub const PLATFORM_X86_64_GNU: &str = "x86_64-unknown-linux-gnu";
pub const PLATFORM_X86_64_MUSL: &str = "x86_64-unknown-linux-musl";
pub const PLATFORM_AARCH64_GNU: &str = "aarch64-unknown-linux-gnu";
pub const PLATFORM_AARCH64_MUSL: &str = "aarch64-unknown-linux-musl";
