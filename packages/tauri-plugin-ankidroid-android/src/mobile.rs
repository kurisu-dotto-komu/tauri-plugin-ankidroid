use crate::android::{
    cards::{
        create_card as android_create_card, delete_card as android_delete_card,
        list_cards as android_list_cards, update_card as android_update_card, CardData,
    },
    decks::list_decks as android_list_decks,
    error::AndroidResult,
    jni_helpers::{attach_current_thread, SafeJNIEnv},
};
use crate::types::{Card, CreateCardResponse, Deck};
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

pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Creating card - Front: {}, Back: {}, Deck: {:?}",
        front,
        back,
        deck
    );

    match create_card_impl(&front, &back, deck.as_deref(), tags.as_deref()).await {
        Ok(note_id) => {
            let response = CreateCardResponse::simple_success(note_id);
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to create card: {}", e);
            let response = CreateCardResponse::error(e.to_string());
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

pub async fn list_cards() -> Result<String, String> {
    log::info!("Listing cards");

    match list_cards_impl().await {
        Ok(cards) => {
            serde_json::to_string(&cards).map_err(|e| format!("Failed to serialize cards: {}", e))
        }
        Err(e) => {
            log::error!("Failed to list cards: {}", e);
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

pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Updating card {} - Front: {}, Back: {}",
        note_id,
        front,
        back
    );

    match update_card_impl(note_id, &front, &back, deck.as_deref(), tags.as_deref()).await {
        Ok(success) => {
            let response = if success {
                CreateCardResponse::success(note_id, Some("Card updated successfully".to_string()))
            } else {
                CreateCardResponse::error("Failed to update card".to_string())
            };
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to update card: {}", e);
            let response = CreateCardResponse::error(e.to_string());
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

pub async fn delete_card(note_id: i64) -> Result<String, String> {
    log::info!("Deleting card with note ID: {}", note_id);

    match delete_card_impl(note_id).await {
        Ok(success) => {
            let response = if success {
                CreateCardResponse::success(note_id, Some("Card deleted successfully".to_string()))
            } else {
                CreateCardResponse::error("Failed to delete card".to_string())
            };
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))
        }
        Err(e) => {
            log::error!("Failed to delete card: {}", e);
            let response = CreateCardResponse::error(e.to_string());
            serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize error response: {}", e))
        }
    }
}

// Internal implementation functions

async fn create_card_impl(
    front: &str,
    back: &str,
    deck: Option<&str>,
    tags: Option<&str>,
) -> AndroidResult<i64> {
    // First attach the thread to get a valid JNI environment
    let env = attach_current_thread()?;
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    android_create_card(safe_env, &activity, front, back, deck, tags)
}

async fn list_cards_impl() -> AndroidResult<Vec<Card>> {
    let env = attach_current_thread()?;
    let safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    let card_data = android_list_cards(safe_env, &activity, Some(20))?; // Limit to 20 cards

    // Convert CardData to Card
    let cards: Vec<Card> = card_data
        .into_iter()
        .map(|data| convert_card_data_to_card(data))
        .collect();

    Ok(cards)
}

async fn get_decks_impl() -> AndroidResult<Vec<Deck>> {
    let env = attach_current_thread()?;
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    let deck_data = android_list_decks(safe_env, &activity)?;

    let decks: Vec<Deck> = deck_data
        .into_iter()
        .map(|(id, name)| Deck::new(id, name))
        .collect();

    Ok(decks)
}

async fn update_card_impl(
    note_id: i64,
    front: &str,
    back: &str,
    deck: Option<&str>,
    tags: Option<&str>,
) -> AndroidResult<bool> {
    let env = attach_current_thread()?;
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    android_update_card(safe_env, &activity, note_id, front, back, deck, tags)
}

async fn delete_card_impl(note_id: i64) -> AndroidResult<bool> {
    let env = attach_current_thread()?;
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    android_delete_card(safe_env, &activity, note_id)
}

async fn check_ankidroid_status() -> AndroidResult<String> {
    use jni::objects::JValue;

    let env = attach_current_thread()?;
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Get a fresh activity reference for this thread
    let ctx = ndk_context::android_context();
    if ctx.context().is_null() {
        return Err(crate::android::error::AndroidError::ValidationError(
            "Android context not initialized. Ensure the app is running on Android.".to_string()
        ));
    }
    
    // Create a local reference to the activity
    let activity = unsafe { 
        let raw_activity = ctx.context() as *mut _;
        jni::objects::JObject::from_raw(raw_activity)
    };

    // Get PackageManager
    let package_manager = safe_env
        .env()
        .call_method(
            &activity,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        )?
        .l()?;

    // Check if AnkiDroid is installed
    let ankidroid_package = safe_env.new_string_checked("com.ichi2.anki")?;
    let package_info_result = safe_env.env().call_method(
        &package_manager,
        "getPackageInfo",
        "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
        &[JValue::Object(&ankidroid_package.into()), JValue::Int(0)],
    );

    match package_info_result {
        Ok(_) => Ok("✅ Connected! AnkiDroid is installed and accessible.".to_string()),
        Err(_) => Err(crate::android::error::AndroidError::AnkiDroidNotInstalled),
    }
}

// Helper function to convert CardData to Card
fn convert_card_data_to_card(data: CardData) -> Card {
    // For now, we'll use a simple deck name mapping
    // In a real implementation, you might want to cache deck names
    let deck_name = format!("Deck {}", data.deck_id);

    Card::with_metadata(
        data.id,
        data.front,
        data.back,
        deck_name,
        data.tags,
        Some(data.deck_id),
        Some(data.model_id),
        Some(data.id), // Use the same ID for note_id
    )
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
    async fn test_list_cards_returns_valid_json() {
        let result = list_cards().await;
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
    async fn test_card_conversion() {
        let card_data = CardData {
            id: 123,
            front: "Test Front".to_string(),
            back: "Test Back".to_string(),
            deck_id: 1,
            model_id: 456,
            tags: "test tag".to_string(),
        };

        let card = convert_card_data_to_card(card_data);
        assert_eq!(card.id, 123);
        assert_eq!(card.front, "Test Front");
        assert_eq!(card.back, "Test Back");
        assert_eq!(card.deck_id, Some(1));
        assert_eq!(card.model_id, Some(456));
        assert_eq!(card.tags, "test tag");
        assert!(card.is_valid());
    }

    #[tokio::test]
    async fn test_create_card_response_serialization() {
        let response = CreateCardResponse::simple_success(123);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"note_id\":123"));
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

        // Test create_card with edge cases
        let result = create_card(long_string.clone(), long_string.clone(), None, None).await;
        assert!(
            result.is_ok(),
            "create_card command should handle long strings"
        );

        // Test list_cards doesn't panic
        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards command should not panic");

        // Test get_decks doesn't panic
        let result = get_decks().await;
        assert!(result.is_ok(), "get_decks command should not panic");
    }
}
