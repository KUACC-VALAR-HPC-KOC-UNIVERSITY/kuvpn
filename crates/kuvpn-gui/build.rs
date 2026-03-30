fn main() {
    // Use CARGO_CFG_TARGET_OS so this runs during cross-compilation too.
    // #[cfg(windows)] only fires when the BUILD HOST is Windows, which breaks
    // cross-compiling from Linux (the icon would never be embedded).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
}
