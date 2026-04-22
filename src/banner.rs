//! Banner and styling for gitclaw CLI

use colored::Colorize;
use rand::seq::SliceRandom;

/// ASCII art banner for gitclaw (white/colorless)
pub const BANNER: &str = r#"
       _ _       _
      (_) |     | |
  __ _ _| |_ ___| | __ ___      __
 / _` | | __/ __| |/ _` \ \ /\ / /
| (_| | | || (__| | (_| |\ V  V /
 \__, |_|\__\___|_|\__,_| \_/\_/
  __/ |
 |___/
"#;

/// Collection of funny taglines
const TAGLINES: [&str; 20] = [
    "Spared no expense.",
    "The claw is the law.",
    "Downloading the internet, one repo at a time.",
    "Because manually downloading releases is so 2024.",
    "GitHub releases? I hardly know her!",
    "Your CPU called. It wants a break.",
    "Making /usr/local/bin great again.",
    "Rustaceans installing at the speed of cargo.",
    "Turbofish not included.",
    "Works on my machine™",
    "If it compiles, it ships.",
    "Here be binaries.",
    "Fearlessly installing since 2024.",
    "Not tested on animals, only on Airscript.",
    "Your daily dose of dependencies.",
    "curl | sh, but make it safe(r).",
    "Homebrew? Never heard of her.",
    "sudo make me a sandwich.",
    "It's not a bug, it's a feature.",
    "Ship it and see.",
];

/// Check if colors should be enabled
fn color_enabled() -> bool {
    std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}

/// Print the banner (white/colorless)
pub fn print_banner() {
    println!("{}", BANNER);
}

/// Print a random tagline
pub fn print_tagline() {
    let tagline = TAGLINES.choose(&mut rand::thread_rng()).unwrap();
    if color_enabled() {
        println!("{}", tagline.dimmed());
    } else {
        println!("{}", tagline);
    }
    println!();
}

/// Print an underlined "Output:" section header
pub fn print_output_header() {
    if color_enabled() {
        println!("{}", "Output:".underline());
        println!();
    } else {
        println!("Output:");
        println!();
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
