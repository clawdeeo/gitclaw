//! Banner and styling for gitclaw CLI

use colored::Colorize;

/// ASCII art banner for gitclaw
#[allow(dead_code)]
pub const BANNER: &str = r#"
   ______   __    __  ________  __       __   ______   __    __ 
  /      \ /  |  /  |/        |/  \     /  | /      \ /  |  /  |
 /$$$$$$  |$$ |  $$ |$$$$$$$$/ $$  \   /$$ |/$$$$$$  |$$ |  $$ |
 $$ |__$$ |$$ |  $$ |   $$ |   $$$  \ /$$$ |$$ |  $$/ $$ |  $$ |
 $$    $$ |$$ |  $$ |   $$ |   $$$$  /$$$$ |$$ |      $$ |  $$ |
 $$$$$$$$ |$$ |  $$ |   $$ |   $$ $$ $$/$$ |$$ |   __ $$ |  $$ |
 $$ |  $$ |$$ \__$$ |   $$ |   $$ |$$$/ $$ |$$ \__/  |$$ \__$$ |
 $$ |  $$ |$$    $$/    $$ |   $$ | $/  $$ |$$    $$/ $$    $$ |
 $$/   $$/  $$$$$$/     $$/    $$/      $$/  $$$$$$/   $$$$$$$ |
                                                        /  \__$$ |
                                                        $$    $$/
                                                         $$$$$$/ 
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

/// Print success message with checkmark
#[allow(dead_code)]
pub fn print_success(text: &str) {
    if color_enabled() {
        println!("{} {}", "✓".green().bold(), text);
    } else {
        println!("[OK] {}", text);
    }
}

/// Print error message with X
#[allow(dead_code)]
pub fn print_error(text: &str) {
    if color_enabled() {
        eprintln!("{} {}", "✗".red().bold(), text);
    } else {
        eprintln!("[ERROR] {}", text);
    }
}

/// Print info message with info symbol
#[allow(dead_code)]
pub fn print_info(text: &str) {
    if color_enabled() {
        println!("{} {}", "ℹ".blue(), text);
    } else {
        println!("[INFO] {}", text);
    }
}

/// Print warning message with warning symbol
#[allow(dead_code)]
pub fn print_warning(text: &str) {
    if color_enabled() {
        println!("{} {}", "⚠".yellow(), text);
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
