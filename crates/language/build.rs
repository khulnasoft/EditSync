fn main() {
    if let Ok(bundled) = std::env::var("EDITSYNC_BUNDLE") {
        println!("cargo:rustc-env=EDITSYNC_BUNDLE={}", bundled);
    }
}
