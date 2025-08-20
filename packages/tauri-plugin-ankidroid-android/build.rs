fn main() {
    #[cfg(feature = "tauri-plugin")]
    {
        const COMMANDS: &[&str] = &["hello"];
        tauri_plugin::Builder::new(COMMANDS)
            .android_path("android")
            .build();
    }
}
