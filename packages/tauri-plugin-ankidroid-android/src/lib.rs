#[cfg(feature = "tauri-plugin")]
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(feature = "tauri-plugin")]
mod commands;
#[cfg(feature = "tauri-plugin")]
mod mobile;
pub mod types;

#[cfg(target_os = "android")]
mod android;

#[cfg(feature = "tauri-plugin")]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ankidroid")
        .invoke_handler(tauri::generate_handler![
            commands::hello,
            // New commands with correct terminology
            commands::list_notes,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            // Legacy commands for backward compatibility
            commands::list_cards,
            commands::create_card,
            commands::update_card,
            commands::delete_card,
            // Deck operations
            commands::get_decks,
        ])
        .setup(|app, api| {
            mobile::init(app, api)?;
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_plugin_name() {
        // Test that the plugin is created with the correct name
        let plugin_name = "ankidroid";
        assert_eq!(plugin_name, "ankidroid");
    }
}
