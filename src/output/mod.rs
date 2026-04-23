use colored::Colorize;
use rand::seq::SliceRandom;

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

pub fn print_version_line() {
    let version = env!("CARGO_PKG_VERSION");
    let tagline = TAGLINES.choose(&mut rand::thread_rng()).unwrap();

    println!(
        "{} {}",
        format!("gitclaw v{}", version).cyan().bold(),
        format!("({})", tagline).dimmed()
    );

    println!();
}

pub fn print_output_header() {
    println!("{}", "Output:".underline());
}

pub fn print_header(text: &str) {
    println!("{}", text.bold());
}

pub fn print_success(text: &str) {
    println!("{} {}", "[EXEC]".green().bold(), text);
}

pub fn print_info(text: &str) {
    println!("{} {}", "[INFO]".cyan(), text);
}

pub fn print_warn(text: &str) {
    println!("{} {}", "[WARN]".yellow().bold(), text);
}

pub fn print_error(text: &str) {
    eprintln!("{} {}", "[ERR]".red().bold(), text);
}

pub fn print_kv(key: &str, value: &str) {
    println!("  {} {}", format!("{:20}", key).dimmed(), value);
}

pub fn print_install_complete(name: &str, binary_path: &str) {
    println!();
    print_success(&format!("Installed {}", name.green().bold()));
    println!("  {} {}", "binary".dimmed(), binary_path.dimmed());
    println!("  {} {}", "run   ".dimmed(), binary_path.to_string().cyan());
}
