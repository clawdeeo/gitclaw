//! Banner and styling for gitclaw CLI

use colored::Colorize;

/// Simple ASCII art banner for gitclaw
pub const BANNER: &str = r#"
   ___ _  _ ___ ___ __ _ __
  / __| || | __| _ \/  \ /
 | (_ | __ | _||  _/ () \
  \___|_||_|___|_|  \__/|_|
"#;

/// Check if colors should be enabled
fn color_enabled() -> bool {
    std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}

/// Print the banner
pub fn print_banner() {
    if color_enabled() {
        println!("{}", BANNER.cyan().bold());
    } else {
        println!("{}", BANNER);
    }
}

/// Print a styled header
#[allow(dead_code)]
pub fn print_header(text: &str) {
    if color_enabled() {
        println!();
        println!("{}", text.bold().underline());
        println!();
    } else {
        println!();
        println!("=== {} ===", text);
        println!();
    }
}

/// Print success message
#[allow(dead_code)]
pub fn print_success(text: &str) {
    if color_enabled() {
        println!("{} {}", "[OK]".green().bold(), text);
    } else {
        println!("[OK] {}", text);
    }
}

/// Print error message
#[allow(dead_code)]
pub fn print_error(text: &str) {
    if color_enabled() {
        eprintln!("{} {}", "[ERR]".red().bold(), text);
    } else {
        eprintln!("[ERROR] {}", text);
    }
}

/// Print info message
#[allow(dead_code)]
pub fn print_info(text: &str) {
    if color_enabled() {
        println!("{} {}", "[INFO]".blue(), text);
    } else {
        println!("[INFO] {}", text);
    }
}

/// Print warning message
#[allow(dead_code)]
pub fn print_warning(text: &str) {
    if color_enabled() {
        println!("{} {}", "[WARN]".yellow(), text);
    } else {
        println!("[WARN] {}", text);
    }
}

/// Print a key-value pair with aligned keys
#[allow(dead_code)]
pub fn print_kv(key: &str, value: &str) {
    if color_enabled() {
        println!("  {} {}", format!("{:20}", key).dimmed(), value);
    } else {
        println!("  {:20} {}", key, value);
    }
}

/// Print a separator line
#[allow(dead_code)]
pub fn print_separator() {
    if color_enabled() {
        println!("{}", "─".repeat(60).dimmed());
    } else {
        println!("{}", "-".repeat(60));
    }
}

/// Print install complete message
#[allow(dead_code)]
pub fn print_install_complete(name: &str, binary_path: &str) {
    if color_enabled() {
        print_success(&format!("Installed {}", name.green().bold()));
        println!("  Binary: {}", binary_path.dimmed());
        println!("  Run: {}", format!("~/.gitclaw/bin/{}", name).cyan());
    } else {
        println!("[OK] Installed {}", name);
        println!("  Binary: {}", binary_path);
        println!("  Run: ~/.gitclaw/bin/{}", name);
    }
}
