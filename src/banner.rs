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

fn color_enabled() -> bool {
    std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}

pub fn print_version_line() {
    let version = env!("CARGO_PKG_VERSION");
    let tagline = TAGLINES.choose(&mut rand::thread_rng()).unwrap();

    if color_enabled() {
        println!(
            "{} {}",
            format!("gitclaw v{}", version).cyan().bold(),
            format!("({})", tagline).dimmed()
        );
    } else {
        println!("gitclaw v{} ({})", version, tagline);
    }

    println!();
}

pub fn print_output_header() {
    if color_enabled() {
        println!("{}", "Output:".underline());
    } else {
        println!("Output:");
    }
}

pub fn print_header(text: &str) {
    if color_enabled() {
        println!("{}", text.bold());
    } else {
        println!("{}", text);
    }
}

pub fn print_success(text: &str) {
    if color_enabled() {
        println!("{} {}", "[EXEC]".green().bold(), text);
    } else {
        println!("[EXEC] {}", text);
    }
}

pub fn print_info(text: &str) {
    if color_enabled() {
        println!("{} {}", "[INFO]".cyan(), text);
    } else {
        println!("[INFO] {}", text);
    }
}

pub fn print_kv(key: &str, value: &str) {
    if color_enabled() {
        println!("  {} {}", format!("{:20}", key).dimmed(), value);
    } else {
        println!("  {:20} {}", key, value);
    }
}

pub fn print_install_complete(name: &str, binary_path: &str) {
    if color_enabled() {
        print_success(&format!("Installed {}", name.green().bold()));
        println!("  {} {}", "binary".dimmed(), binary_path.dimmed());

        println!(
            "  {} {}",
            "run   ".dimmed(),
            format!("~/.gitclaw/bin/{}", name).cyan()
        );
    } else {
        println!("[EXEC] Installed {}", name);
        println!("  binary {}", binary_path);
        println!("  run    ~/.gitclaw/bin/{}", name);
    }
}
