use std::process::Command;

const EDITSYNC_MANIFEST: &str = include_str!("../editsync/Cargo.toml");

fn main() {
    let editsync_cargo_toml: cargo_toml::Manifest =
        toml::from_str(EDITSYNC_MANIFEST).expect("failed to parse editsync Cargo.toml");
    println!(
        "cargo:rustc-env=EDITSYNC_PKG_VERSION={}",
        editsync_cargo_toml.package.unwrap().version.unwrap()
    );
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );

    // If we're building this for nightly, we want to set the EDITSYNC_COMMIT_SHA
    if let Some(release_channel) = std::env::var("EDITSYNC_RELEASE_CHANNEL").ok() {
        if release_channel.as_str() == "nightly" {
            // Populate git sha environment variable if git is available
            println!("cargo:rerun-if-changed=../../.git/logs/HEAD");
            if let Some(output) = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
            {
                let git_sha = String::from_utf8_lossy(&output.stdout);
                let git_sha = git_sha.trim();

                println!("cargo:rustc-env=EDITSYNC_COMMIT_SHA={git_sha}");
            }
        }
    }
}
