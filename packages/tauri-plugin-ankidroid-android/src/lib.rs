use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
mod mobile;
pub mod types;

#[cfg(target_os = "android")]
mod android;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ankidroid")
        .invoke_handler(tauri::generate_handler![
            commands::hello,
            commands::list_cards,
            commands::create_card,
            commands::get_decks,
            commands::update_card,
            commands::delete_card,
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
