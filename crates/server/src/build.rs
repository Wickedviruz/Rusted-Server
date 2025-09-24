use std::env;
use std::process::Command;

fn main() {
    // Git commit hash
    if let Ok(output) = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
        if let Ok(hash) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
        }
    }

    // Kompileringsdatum & tid
    println!("cargo:rustc-env=BUILD_DATE={}", chrono::Utc::now().to_rfc3339());

    // Kompileringsplattform
    println!("cargo:rustc-env=BUILD_TARGET={}", env::var("TARGET").unwrap_or_default());
}
