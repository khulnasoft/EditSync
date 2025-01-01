fn main() {
    if std::env::var("EDITSYNC_UPDATE_EXPLANATION").is_ok() {
        println!(r#"cargo:rustc-cfg=feature="no-bundled-uninstall""#);
    }
}
