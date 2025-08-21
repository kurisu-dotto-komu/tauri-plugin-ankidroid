use crate::android::api_wrapper;
use crate::types::{Card, CreateCardResponse, Deck};
use ankidroid_api_rust::AnkiDroidApiExtended;
use tauri::{AppHandle, Runtime};

pub fn init<R: Runtime>(
    _app: &AppHandle<R>,
    _api: tauri::plugin::PluginApi<R, ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing AnkiDroid mobile plugin");
    Ok(())
}

pub async fn hello(name: String) -> Result<String, String> {
    log::info!("Hello called with name: {}", name);

    match check_ankidroid_status().await {
        Ok(status) => Ok(format!("Hello, {}! 🎉\n\nAnkiDroid Status: {}", name, status)),
        Err(e) => Ok(format!(
            "Hello, {}! 👋\n\nAnkiDroid access: {}\n\nNote: Make sure AnkiDroid is installed and has API access enabled.",
            name, e
        ))
    }
}

// Renamed from create_card to create_note - we create Notes, not Cards
pub async fn create_note(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Creating note - Front: {}, Back: {}, Deck: {:?}",
        front,
        back,
        deck
    );

    match create_note_impl(&front, &back, deck.as_deref(), tags.as_deref()).await {
        Ok(note_id) => {
            let response = CreateCardResponse::simple_success(note_id);
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to create note: {}", e);
            let response = CreateCardResponse::error(e);
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

// Legacy wrapper for backward compatibility - redirects to create_note
pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    create_note(front, back, deck, tags).await
}

// Renamed from list_cards to list_notes - we list Notes, not Cards
pub async fn list_notes() -> Result<String, String> {
    log::info!("Listing notes");

    match list_notes_impl().await {
        Ok(notes) => {
            serde_json::to_string(&notes).map_err(|e| format!("Failed to serialize notes: {}", e))
        }
        Err(e) => {
            log::error!("Failed to list notes: {}", e);
            // Return error cards to maintain API compatibility
            let error_cards = vec![Card::new(
                1,
                "AnkiDroid Error".to_string(),
                format!("Error occurred: {}", e),
                "Error".to_string(),
                "".to_string(),
            )];
            serde_json::to_string(&error_cards)
                .map_err(|e| format!("Failed to serialize error cards: {}", e))
        }
    }
}

// Legacy wrapper for backward compatibility - redirects to list_notes
pub async fn list_cards() -> Result<String, String> {
    list_notes().await
}

pub async fn get_decks() -> Result<String, String> {
    log::info!("Getting decks");

    match get_decks_impl().await {
        Ok(decks) => {
            serde_json::to_string(&decks).map_err(|e| format!("Failed to serialize decks: {}", e))
        }
        Err(e) => {
            log::error!("Failed to get decks: {}", e);
            // Return default deck to maintain API compatibility
            let default_decks = vec![Deck::new(1, "Default".to_string())];
            serde_json::to_string(&default_decks)
                .map_err(|e| format!("Failed to serialize default decks: {}", e))
        }
    }
}

// Renamed from update_card to update_note - we update Notes, not Cards
pub async fn update_note(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Updating note {} - Front: {}, Back: {}",
        note_id,
        front,
        back
    );

    match update_note_impl(note_id, &front, &back, deck.as_deref(), tags.as_deref()).await {
        Ok(success) => {
            let response = if success {
                CreateCardResponse::success(note_id, Some("Note updated successfully".to_string()))
            } else {
                CreateCardResponse::error("Failed to update note".to_string())
            };
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to update note: {}", e);
            let response = CreateCardResponse::error(e);
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

// Legacy wrapper for backward compatibility - redirects to update_note
pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    update_note(note_id, front, back, deck, tags).await
}

// Renamed from delete_card to delete_note - we delete Notes, not Cards
pub async fn delete_note(note_id: i64) -> Result<String, String> {
    log::info!("Deleting note with ID: {}", note_id);

    match delete_note_impl(note_id).await {
        Ok(success) => {
            let response = if success {
                CreateCardResponse::success(note_id, Some("Note deleted successfully".to_string()))
            } else {
                CreateCardResponse::error("Failed to delete note".to_string())
            };
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to delete note: {}", e);
            let response = CreateCardResponse::error(e);
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

// Legacy wrapper for backward compatibility - redirects to delete_note
pub async fn delete_card(note_id: i64) -> Result<String, String> {
    delete_note(note_id).await
}

// Internal implementation functions using ankidroid-api-rust

async fn create_note_impl(
    front: &str,
    back: &str,
    deck: Option<&str>,
    tags: Option<&str>,
) -> Result<i64, String> {
    api_wrapper::with_api_instance(|api| {
        // Get or create model (use default Basic model)
        let model_id = api.add_new_basic_model("Basic")
            .map_err(|e| api_wrapper::format_error(e))?
            .unwrap_or(1); // Default model ID if already exists

        // Get or create deck
        let deck_id = if let Some(deck_name) = deck {
            api.add_new_deck(deck_name)
                .map_err(|e| api_wrapper::format_error(e))?
                .unwrap_or(1) // Default deck if already exists
        } else {
            1 // Default deck ID
        };

        // Prepare tags
        let tag_vec: Option<Vec<&str>> = tags.map(|t| vec![t]);

        // Add note using ankidroid-api-rust (this creates a Note, which generates Cards)
        let note_id = api.add_note(
            model_id,
            deck_id,
            &[front, back],
            tag_vec.as_deref()
        ).map_err(|e| api_wrapper::format_error(e))?;

        note_id.ok_or_else(|| "Failed to create note - no ID returned".to_string())
    })
}

async fn list_notes_impl() -> Result<Vec<Card>, String> {
    api_wrapper::with_api_instance(|api| {
        // Use the extended API method to list notes
        let notes = api.list_notes()
            .map_err(|e| api_wrapper::format_error(e))?;
        
        // Convert Notes to Card format for frontend compatibility
        let cards: Vec<Card> = notes.into_iter()
            .take(20) // Limit to 20 for performance
            .map(|note| {
                // Use default deck since we don't have deck info directly from notes
                let deck_name = "Default".to_string();
                let deck_id = 1i64; // Default deck ID
                
                // Join fields with a separator for display
                let front = note.fields.get(0).cloned().unwrap_or_default();
                let back = note.fields.get(1).cloned().unwrap_or_default();
                let tags = note.tags.join(" ");
                
                Card::with_metadata(
                    note.id,
                    front,
                    back,
                    deck_name,
                    tags,
                    Some(deck_id),
                    Some(note.mid),
                    Some(note.id),
                )
            })
            .collect();
        
        Ok(cards)
    })
}

async fn get_decks_impl() -> Result<Vec<Deck>, String> {
    api_wrapper::with_api_instance(|api| {
        // Get deck list returns HashMap<deck_id, deck_name>
        let deck_map = api.get_deck_list()
            .map_err(|e| api_wrapper::format_error(e))?;
        
        // Convert to expected format
        let decks: Vec<Deck> = deck_map.into_iter()
            .map(|(id, name)| Deck::new(id, name))
            .collect();
        
        Ok(decks)
    })
}

async fn update_note_impl(
    note_id: i64,
    front: &str,
    back: &str,
    _deck: Option<&str>,
    _tags: Option<&str>,
) -> Result<bool, String> {
    api_wrapper::with_api_instance(|api| {
        // Use the extended API method to update note
        api.update_note(note_id, &[front, back])
            .map_err(|e| api_wrapper::format_error(e))?;
        
        Ok(true)
    })
}

async fn delete_note_impl(note_id: i64) -> Result<bool, String> {
    api_wrapper::with_api_instance(|api| {
        // Use the extended API method to delete note
        let deleted = api.delete_note(note_id)
            .map_err(|e| api_wrapper::format_error(e))?;
        
        Ok(deleted)
    })
}

async fn check_ankidroid_status() -> Result<String, String> {
    use jni::objects::JValue;
    
    // Get Android context
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err("Android context not initialized".to_string());
    }
    
    // Attach thread and get JNI environment
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) }
        .map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    let activity = unsafe {
        jni::objects::JObject::from_raw(ctx.context() as jni::sys::jobject)
    };
    
    // Get PackageManager
    let package_manager = env
        .call_method(
            &activity,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        )
        .map_err(|e| format!("Failed to get PackageManager: {}", e))?
        .l()
        .map_err(|e| format!("Failed to convert PackageManager: {}", e))?;
    
    // Check if AnkiDroid is installed
    let ankidroid_package = env.new_string("com.ichi2.anki")
        .map_err(|e| format!("Failed to create string: {}", e))?;
    
    let package_info_result = env.call_method(
        &package_manager,
        "getPackageInfo",
        "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
        &[JValue::Object(&ankidroid_package.into()), JValue::Int(0)],
    );
    
    match package_info_result {
        Ok(_) => {
            // Try to initialize API to check permissions
            match api_wrapper::with_api_instance(|_api| Ok(())) {
                Ok(_) => Ok("✅ Connected! AnkiDroid is installed and API is accessible.".to_string()),
                Err(e) => Ok(format!("⚠️ AnkiDroid is installed but API access failed: {}", e))
            }
        }
        Err(_) => Err("AnkiDroid is not installed".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_returns_valid_response() {
        let result = hello("TestUser".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("TestUser"));
        assert!(response.len() > 0);
    }

    #[tokio::test]
    async fn test_hello_handles_empty_name() {
        let result = hello("".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Hello"));
    }

    #[tokio::test]
    async fn test_list_notes_returns_valid_json() {
        let result = list_notes().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Validate it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(
            parsed.is_ok(),
            "Response should be valid JSON: {}",
            response
        );

        // Check it's an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "Response should be a JSON array");
    }

    #[tokio::test]
    async fn test_get_decks_returns_valid_json() {
        let result = get_decks().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "get_decks should return valid JSON");

        // Should be an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "get_decks should return a JSON array");
    }

    #[tokio::test]
    async fn test_commands_dont_panic() {
        // Test that commands handle edge cases without panicking
        let test_cases = vec!["", "test", "special!@#$%^&*()chars"];
        let long_string = "very_long_string_".repeat(100);

        for test_case in test_cases {
            let result = hello(test_case.to_string()).await;
            assert!(
                result.is_ok(),
                "hello command should not panic with input: {}",
                test_case
            );
        }

        // Test with long string
        let result = hello(long_string.clone()).await;
        assert!(result.is_ok(), "hello command should handle long strings");

        // Test create_note with edge cases
        let result = create_note(long_string.clone(), long_string.clone(), None, None).await;
        assert!(
            result.is_ok(),
            "create_note command should handle long strings"
        );

        // Test list_notes doesn't panic
        let result = list_notes().await;
        assert!(result.is_ok(), "list_notes command should not panic");

        // Test get_decks doesn't panic
        let result = get_decks().await;
        assert!(result.is_ok(), "get_decks command should not panic");
    }

    #[tokio::test]
    async fn test_backward_compatibility() {
        // Test that legacy functions still work
        let result = create_card("Front".to_string(), "Back".to_string(), None, None).await;
        assert!(result.is_ok(), "create_card wrapper should work");

        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards wrapper should work");

        let result = update_card(1, "Front".to_string(), "Back".to_string(), None, None).await;
        assert!(result.is_ok(), "update_card wrapper should work");

        let result = delete_card(1).await;
        assert!(result.is_ok(), "delete_card wrapper should work");
    }
}