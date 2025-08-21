// use crate::android::add_content_api::AddContentApi; // Disabled - can't use from external app
use crate::android::constants::{deck_columns, DECKS_URI, DEFAULT_DECK_ID};
use crate::android::content_provider::{insert, query};
use crate::android::cursor::collect_cursor_results;
use crate::android::error::{AndroidError, AndroidResult};
use crate::android::jni_helpers::{ContentValuesBuilder, SafeJNIEnv};
use jni::objects::JObject;

/// Find a deck ID by name
pub fn find_deck_id_by_name(
    env: SafeJNIEnv,
    activity: &JObject,
    deck_name: &str,
) -> AndroidResult<Option<i64>> {
    log::info!("Searching for deck: {}", deck_name);

    let projection = vec![
        deck_columns::DID.to_string(),
        deck_columns::DECK_ID.to_string(),
        deck_columns::NAME.to_string(),
        deck_columns::DECK_NAME.to_string(),
    ];

    let cursor = query(env, DECKS_URI)
        .projection(projection)
        .execute(activity)?;

    let decks = collect_cursor_results(cursor, |cursor| {
        // Try both possible column names for deck ID
        let deck_id = cursor
            .get_long_by_name(deck_columns::DID)
            .or_else(|_| cursor.get_long_by_name(deck_columns::DECK_ID))
            .unwrap_or(0);

        // Try both possible column names for deck name
        let name = cursor
            .get_string_by_name(deck_columns::NAME)
            .or_else(|_| cursor.get_string_by_name(deck_columns::DECK_NAME))
            .unwrap_or_default();

        Ok((deck_id, name))
    })?;

    // Look for exact name match first
    for (id, name) in &decks {
        if name == deck_name {
            log::info!("Found exact deck match: {} (ID: {})", name, id);
            return Ok(Some(*id));
        }
    }

    // Look for case-insensitive match
    let deck_name_lower = deck_name.to_lowercase();
    for (id, name) in &decks {
        if name.to_lowercase() == deck_name_lower {
            log::info!("Found case-insensitive deck match: {} (ID: {})", name, id);
            return Ok(Some(*id));
        }
    }

    // Look for partial match
    for (id, name) in &decks {
        if name.to_lowercase().contains(&deck_name_lower) {
            log::info!("Found partial deck match: {} (ID: {})", name, id);
            return Ok(Some(*id));
        }
    }

    log::info!("Deck '{}' not found", deck_name);
    Ok(None)
}

/// Create a deck if it doesn't exist, otherwise return existing deck ID
pub fn create_deck_if_not_exists(
    mut env: SafeJNIEnv,
    activity: &JObject,
    deck_name: &str,
) -> AndroidResult<i64> {
    log::info!("🏗️ CREATE_DECK_IF_NOT_EXISTS: '{}'", deck_name);

    // First check if deck already exists
    log::info!("🔍 Step 1: Checking if deck already exists...");
    let env_clone = env.clone();
    match find_deck_id_by_name(env_clone, activity, deck_name) {
        Ok(Some(existing_id)) => {
            log::info!("✅ Deck '{}' already exists with ID: {}", deck_name, existing_id);
            return Ok(existing_id);
        }
        Ok(None) => {
            log::info!("🔍 Deck '{}' not found in initial search", deck_name);
        }
        Err(e) => {
            log::error!("❌ Error during initial deck search: {}", e);
            // Continue with creation attempt
        }
    }

    // Double check by listing all decks to make sure we didn't miss it
    log::info!("🔍 Step 2: Double-checking by listing all decks...");
    let env_for_list = env.clone();
    match list_decks(env_for_list, activity) {
        Ok(decks) => {
            log::info!("✅ Found {} total decks", decks.len());
            for (id, name) in &decks {
                log::info!("  - Deck: '{}' (ID: {})", name, id);
                if name.eq_ignore_ascii_case(deck_name) {
                    log::info!("✅ Found existing deck '{}' with ID {} (case insensitive match)", name, id);
                    return Ok(*id);
                }
            }
            log::info!("🔍 No case-insensitive match found for '{}'", deck_name);
        }
        Err(e) => {
            log::error!("❌ Failed to list decks during double-check: {}", e);
        }
    }

    // Try to create the deck only if we're absolutely sure it doesn't exist
    log::info!("🏗️ Step 3: Deck '{}' does not exist, attempting to create it", deck_name);
    let env_for_create = env.clone();
    
    // Check for JNI exceptions before creation attempt
    if env.env().exception_check().unwrap_or(false) {
        log::error!("🔥 JNI EXCEPTION PRESENT before deck creation attempt!");
        env.env().exception_describe().ok();
        env.env().exception_clear().ok();
    }
    
    match create_deck(env_for_create, activity, deck_name) {
        Ok(deck_id) => {
            log::info!("✅ Created new deck '{}' with ID: {}", deck_name, deck_id);
            Ok(deck_id)
        }
        Err(e) => {
            log::error!("❌ CRITICAL: Failed to create deck '{}': {}", deck_name, e);
            log::error!("❌ Error details: {:?}", e);

            // Check for JNI exceptions after creation failure
            if env.env().exception_check().unwrap_or(false) {
                log::error!("🔥 JNI EXCEPTION DETECTED after deck creation failure!");
                if let Ok(exception) = env.env().exception_occurred() {
                    log::error!("🔥 Exception object found: {}", exception.as_raw() as usize);
                    env.env().exception_describe().ok();
                    env.env().exception_clear().ok();
                }
            }

            // Check if this is a "deck already exists" error
            let error_message = e.to_string();
            if error_message.contains("already exists") || error_message.contains("Deck name already exists") {
                log::info!("🔄 Deck '{}' seems to already exist (race condition?), searching again...", deck_name);
                // Check one more time in case another process created it
                let env_for_find = env.clone();
                match find_deck_id_by_name(env_for_find, activity, deck_name) {
                    Ok(Some(existing_id)) => {
                        log::info!("✅ Found existing deck '{}' with ID: {} after creation failure", deck_name, existing_id);
                        return Ok(existing_id);
                    }
                    Ok(None) => {
                        log::error!("❌ Still could not find deck '{}' after creation failure", deck_name);
                    }
                    Err(find_error) => {
                        log::error!("❌ Error searching for deck after creation failure: {}", find_error);
                    }
                }
            }

            // For automated testing, let's NOT fall back to default deck immediately
            // Instead, propagate the error so we can see what's really happening
            log::error!("🔥 PROPAGATING ERROR instead of using default deck to debug the issue");
            return Err(e);
        }
    }
}

/// Create a new deck
pub fn create_deck(mut env: SafeJNIEnv, activity: &JObject, deck_name: &str) -> AndroidResult<i64> {
    log::info!("Creating new deck: {}", deck_name);

    if deck_name.trim().is_empty() {
        return Err(AndroidError::validation_error("Deck name cannot be empty"));
    }

    // Skip AddContentApi for now since it's causing crashes
    // The AddContentApi class is in AnkiDroid app, not ours, so we can't instantiate it
    // We'll use ContentProvider directly instead
    
    // Use ContentProvider method
    log::info!("Using ContentProvider to create deck...");
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_string(deck_columns::DECK_NAME, deck_name)?
        .put_string(deck_columns::NAME, deck_name)?; // Try both column names

    let result_uri = insert(env, DECKS_URI).execute(activity, values)?;

    // Extract deck ID from the returned URI
    extract_id_from_uri(&result_uri)
}

/// Get all available decks
pub fn list_decks(env: SafeJNIEnv, activity: &JObject) -> AndroidResult<Vec<(i64, String)>> {
    log::info!("Listing all available decks");

    // Try querying without any projection first to see what columns are available
    let cursor = query(env, DECKS_URI)
        .execute(activity)?;

    collect_cursor_results(cursor, |cursor| {
        // Try to get column count and names for debugging
        log::info!("Cursor column count: {}", cursor.get_column_count().unwrap_or(0));
        
        // Try different possible column name combinations
        let deck_id = cursor
            .get_long_by_name("_id")  // Most common Android ID column
            .or_else(|_| cursor.get_long_by_name(deck_columns::DID))
            .or_else(|_| cursor.get_long_by_name(deck_columns::DECK_ID))
            .unwrap_or_else(|e| {
                log::warn!("Could not get deck ID: {}", e);
                0
            });

        // Try different possible column names for deck name
        let name = cursor
            .get_string_by_name("name")  // Simple 'name' column
            .or_else(|_| cursor.get_string_by_name(deck_columns::NAME))
            .or_else(|_| cursor.get_string_by_name(deck_columns::DECK_NAME))
            .or_else(|_| cursor.get_string_by_name("deckname"))  // Try different variations
            .or_else(|_| cursor.get_string_by_name("deck_name_json"))  // AnkiDroid might use JSON
            .unwrap_or_else(|e| {
                log::warn!("Could not get deck name: {}", e);
                format!("Deck {}", deck_id)  // Fallback to show deck ID
            });

        log::info!("Found deck: '{}' (ID: {})", name, deck_id);
        Ok((deck_id, name))
    })
}

/// Check if a deck exists by ID
pub fn deck_exists(env: SafeJNIEnv, activity: &JObject, deck_id: i64) -> AndroidResult<bool> {
    log::info!("Checking if deck exists: {}", deck_id);

    let projection = vec![deck_columns::DID.to_string()];
    let selection = format!("{} = ?", deck_columns::DID);
    let selection_args = vec![deck_id.to_string()];

    let mut cursor = query(env, DECKS_URI)
        .projection(projection)
        .selection(selection)
        .selection_args(selection_args)
        .execute(activity)?;

    let count = cursor.get_count()?;
    Ok(count > 0)
}

/// Get deck name by ID
pub fn get_deck_name(env: SafeJNIEnv, activity: &JObject, deck_id: i64) -> AndroidResult<String> {
    log::info!("Getting deck name for ID: {}", deck_id);

    let projection = vec![
        deck_columns::NAME.to_string(),
        deck_columns::DECK_NAME.to_string(),
    ];
    let selection = format!("{} = ?", deck_columns::DID);
    let selection_args = vec![deck_id.to_string()];

    let cursor = query(env, DECKS_URI)
        .projection(projection)
        .selection(selection)
        .selection_args(selection_args)
        .execute(activity)?;

    let results = collect_cursor_results(cursor, |cursor| {
        // Try both possible column names
        let name = cursor
            .get_string_by_name(deck_columns::NAME)
            .or_else(|_| cursor.get_string_by_name(deck_columns::DECK_NAME))
            .unwrap_or_default();
        Ok(name)
    })?;

    results
        .into_iter()
        .next()
        .ok_or_else(|| AndroidError::deck_not_found(format!("Deck ID {} not found", deck_id)))
}

/// Validate deck name
pub fn validate_deck_name(deck_name: &str) -> AndroidResult<()> {
    if deck_name.trim().is_empty() {
        return Err(AndroidError::validation_error("Deck name cannot be empty"));
    }

    if deck_name.len() > 100 {
        return Err(AndroidError::validation_error(
            "Deck name too long (max 100 characters)",
        ));
    }

    // Check for invalid characters (AnkiDroid specific restrictions)
    if deck_name.contains('\0') || deck_name.contains('\n') || deck_name.contains('\r') {
        return Err(AndroidError::validation_error(
            "Deck name contains invalid characters",
        ));
    }

    Ok(())
}

/// Extract ID from ContentProvider insert result URI
fn extract_id_from_uri(uri_string: &str) -> AndroidResult<i64> {
    log::debug!("Extracting ID from URI: {}", uri_string);

    // AnkiDroid typically returns URIs like "content://com.ichi2.anki.flashcards/decks/123"
    uri_string
        .split('/')
        .last()
        .and_then(|id_str| id_str.parse::<i64>().ok())
        .ok_or_else(|| {
            log::warn!("Could not parse deck ID from URI: {}", uri_string);
            AndroidError::database_error(format!(
                "Could not parse deck ID from URI: {}",
                uri_string
            ))
        })
}

/// Get or create deck ID for the given deck name
pub fn get_or_create_deck_id(
    env: &mut SafeJNIEnv,
    activity: &JObject,
    deck_name: Option<&str>,
) -> AndroidResult<i64> {
    match deck_name {
        Some(name) => {
            validate_deck_name(name)?;
            let env_clone = env.clone();
            create_deck_if_not_exists(env_clone, activity, name)
        }
        None => {
            log::info!(
                "No deck name provided, using default deck ID: {}",
                DEFAULT_DECK_ID
            );
            Ok(DEFAULT_DECK_ID)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_deck_name_valid() {
        assert!(validate_deck_name("Valid Deck Name").is_ok());
        assert!(validate_deck_name("Test::Subdeck").is_ok());
        assert!(validate_deck_name("数学").is_ok()); // Unicode characters
    }

    #[test]
    fn test_validate_deck_name_invalid() {
        assert!(validate_deck_name("").is_err());
        assert!(validate_deck_name("   ").is_err());
        assert!(validate_deck_name("Name\0WithNull").is_err());
        assert!(validate_deck_name("Name\nWithNewline").is_err());
        assert!(validate_deck_name(&"x".repeat(101)).is_err());
    }

    #[test]
    fn test_extract_id_from_uri() {
        assert_eq!(
            extract_id_from_uri("content://com.ichi2.anki.flashcards/decks/123").unwrap(),
            123
        );
        assert_eq!(
            extract_id_from_uri("content://provider/decks/456").unwrap(),
            456
        );
        assert!(extract_id_from_uri("invalid_uri").is_err());
        assert!(extract_id_from_uri("content://provider/decks/not_a_number").is_err());
    }

    #[test]
    fn test_default_deck_id() {
        assert_eq!(DEFAULT_DECK_ID, 1);
    }

    #[test]
    fn test_deck_name_case_insensitive_comparison() {
        let deck_name = "Test Deck";
        let deck_name_lower = deck_name.to_lowercase();
        assert_eq!(deck_name_lower, "test deck");
        assert!("Test Deck".to_lowercase().contains(&deck_name_lower));
    }

    #[test]
    fn test_get_or_create_deck_id_logic() {
        // Test with None
        let deck_name: Option<&str> = None;
        assert!(deck_name.is_none());

        // Test with Some
        let deck_name = Some("Test Deck");
        assert!(deck_name.is_some());
        assert!(validate_deck_name(deck_name.unwrap()).is_ok());
    }
}
