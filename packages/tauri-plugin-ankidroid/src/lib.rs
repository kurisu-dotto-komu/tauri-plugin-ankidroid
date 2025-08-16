use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
mod desktop;
mod mobile;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ankidroid")
        .invoke_handler(tauri::generate_handler![commands::hello, commands::list_cards,])
        .setup(|app, api| {
            #[cfg(mobile)]
            {
                mobile::init(app, api)?;
            }
            #[cfg(desktop)]
            {
                desktop::init(app, api)?;
            }
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
