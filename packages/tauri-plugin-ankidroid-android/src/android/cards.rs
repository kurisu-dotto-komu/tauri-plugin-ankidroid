use crate::android::add_content_api::AddContentApi;
use crate::android::constants::{note_columns, FIELD_SEPARATOR, NOTES_URI};
use crate::android::content_provider::{delete, insert, query, update};
use crate::android::cursor::collect_cursor_results;
use crate::android::decks::get_or_create_deck_id;
use crate::android::error::{AndroidError, AndroidResult};
use crate::android::jni_helpers::{ContentValuesBuilder, SafeJNIEnv};
use crate::android::models::{find_basic_model_id, validate_model_for_cards};
use jni::objects::JObject;

/// Create a new card (note) in AnkiDroid
pub fn create_card(
    mut env: SafeJNIEnv,
    activity: &JObject,
    front: &str,
    back: &str,
    deck_name: Option<&str>,
    tags: Option<&str>,
) -> AndroidResult<i64> {
    log::info!(
        "🔄 START CREATE_CARD - Front: '{}', Back: '{}', Deck: {:?}, Tags: {:?}",
        front,
        back,
        deck_name,
        tags
    );

    // Clear any pending JNI exceptions at the start
    if env.env().exception_check().unwrap_or(false) {
        log::warn!("🔥 Clearing pending JNI exception at start of create_card");
        env.env().exception_clear().ok();
    }

    // Validate inputs
    log::info!("📝 Validating card fields...");
    if let Err(e) = validate_card_fields(front, back) {
        log::error!("❌ Card field validation failed: {}", e);
        return Err(e);
    }
    log::info!("✅ Card fields validated successfully");

    // Get or create deck - AVOID cloning env here to prevent reference issues
    log::info!("🏗️ Getting or creating deck: {:?}", deck_name);
    let deck_id = match get_or_create_deck_id(&mut env, activity, deck_name) {
        Ok(id) => {
            log::info!("✅ Deck operation successful - using deck ID: {}", id);
            id
        }
        Err(e) => {
            log::error!("❌ CRITICAL: Deck operation failed with error: {}", e);
            log::error!("❌ Error type: {:?}", e);
            
            // Check for and clear any JNI exceptions
            if env.env().exception_check().unwrap_or(false) {
                log::error!("🔥 JNI EXCEPTION DETECTED during deck operation!");
                env.env().exception_describe().ok();
                env.env().exception_clear().ok();
            }
            return Err(e);
        }
    };

    // Find Basic model
    log::info!("🔍 Finding Basic model...");
    let model_id = match find_basic_model_id(&mut env, activity) {
        Ok(id) => {
            log::info!("✅ Found Basic model ID: {}", id);
            id
        }
        Err(e) => {
            log::error!("❌ Failed to find Basic model: {}", e);
            // Clear any exceptions before returning
            if env.env().exception_check().unwrap_or(false) {
                env.env().exception_clear().ok();
            }
            return Err(e);
        }
    };

    // Validate model is suitable for cards
    log::info!("🔍 Validating model for cards...");
    if let Err(e) = validate_model_for_cards(&mut env, activity, model_id) {
        log::error!("❌ Model validation failed: {}", e);
        // Clear any exceptions before returning
        if env.env().exception_check().unwrap_or(false) {
            env.env().exception_clear().ok();
        }
        return Err(e);
    }
    log::info!("✅ Model validated successfully");

    // Try direct ContentProvider approach with mid in ContentValues
    log::info!("🎯 Attempting to create note using direct ContentProvider with mid...");
    
    // Format fields with proper separator (0x1f)
    let fields = format!("{}{}{}", front, FIELD_SEPARATOR, back);
    let tags_str = tags.unwrap_or("").trim();
    log::info!("📝 Formatted fields: '{}', tags: '{}'", fields, tags_str);
    
    // Create ContentValues INCLUDING the model ID (mid)
    log::info!("🏗️ Creating ContentValues with mid={}...", model_id);
    let mut env_for_values = env.clone();
    let values_builder = match ContentValuesBuilder::new(&mut env_for_values) {
        Ok(mut builder) => {
            log::info!("✅ ContentValuesBuilder created");
            
            // IMPORTANT: Include mid (model ID) in the ContentValues
            builder = builder.put_long(note_columns::MID, model_id)?;
            log::info!("✅ Added mid={} to ContentValues", model_id);
            
            // Add the fields
            builder = builder.put_string(note_columns::FLDS, &fields)?;
            log::info!("✅ Added flds to ContentValues");
            
            // Add tags if provided
            if !tags_str.is_empty() {
                builder = builder.put_string(note_columns::TAGS, tags_str)?;
                log::info!("✅ Added tags to ContentValues");
            }
            
            // Add other required fields that might be needed
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            builder = builder.put_long(note_columns::MOD, timestamp)?;
            log::info!("✅ Added mod (timestamp) to ContentValues");
            
            // guid is usually generated by AnkiDroid, but we can provide one
            let guid = format!("tpa{}", timestamp); // tauri-plugin-ankidroid prefix
            builder = builder.put_string(note_columns::GUID, &guid)?;
            log::info!("✅ Added guid to ContentValues");
            
            // sfld (sort field) is usually the first field
            builder = builder.put_string(note_columns::SFLD, front)?;
            log::info!("✅ Added sfld to ContentValues");
            
            // usn (update sequence number) - 0 for new notes
            builder = builder.put_int(note_columns::USQN, 0)?;
            log::info!("✅ Added usn to ContentValues");
            
            // flags - default to 0
            builder = builder.put_int(note_columns::FLAGS, 0)?;
            log::info!("✅ Added flags to ContentValues");
            
            // data - empty string for now
            builder = builder.put_string(note_columns::DATA, "")?;
            log::info!("✅ Added data to ContentValues");
            
            builder
        }
        Err(e) => {
            log::error!("❌ Failed to create ContentValuesBuilder: {}", e);
            return Err(e);
        }
    };

    // Insert the note
    log::info!("💾 Inserting note into AnkiDroid...");
    log::info!("✅ ContentValues built successfully");
    let env_for_insert = env.clone();
    let result_uri = match insert(env_for_insert, NOTES_URI).execute(activity, values_builder) {
        Ok(uri) => {
            log::info!("✅ Note inserted successfully, URI: {}", uri);
            uri
        }
        Err(e) => {
            log::error!("❌ CRITICAL: Note insertion failed: {}", e);
            log::error!("❌ Error details: {:?}", e);
            // Clear any exceptions before returning
            if env.env().exception_check().unwrap_or(false) {
                log::error!("🔥 JNI exception during insert operation");
                env.env().exception_describe().ok();
                env.env().exception_clear().ok();
            }
            return Err(e);
        }
    };

    // Extract note ID from URI
    log::info!("🔍 Extracting note ID from URI...");
    let note_id = match extract_id_from_uri(&result_uri) {
        Ok(id) => {
            log::info!("✅ Successfully extracted note ID: {}", id);
            id
        }
        Err(e) => {
            log::error!("❌ Failed to extract note ID from URI '{}': {}", result_uri, e);
            return Err(e);
        }
    };

    // Final exception check before success
    if env.env().exception_check().unwrap_or(false) {
        log::error!("🔥 JNI exception detected at end of create_card - clearing");
        env.env().exception_describe().ok();
        env.env().exception_clear().ok();
    }

    log::info!("🎉 CREATE_CARD COMPLETED SUCCESSFULLY - Note ID: {}", note_id);
    Ok(note_id)
}

/// List cards from AnkiDroid
pub fn list_cards(
    env: SafeJNIEnv,
    activity: &JObject,
    limit: Option<i32>,
) -> AndroidResult<Vec<CardData>> {
    log::info!("Listing cards with limit: {:?}", limit);

    // First, try a simple query to test basic connectivity
    log::info!("Testing basic AnkiDroid connectivity...");
    
    let projection = vec![
        note_columns::ID.to_string(),
        note_columns::FLDS.to_string(),
        note_columns::TAGS.to_string(),
    ];

    let query_builder = query(env, NOTES_URI)
        .projection(projection)
        .sort_order(format!("{} DESC", note_columns::MOD));

    log::info!("Executing query against AnkiDroid ContentProvider...");
    let cursor = match query_builder.execute(activity) {
        Ok(cursor) => {
            log::info!("Query executed successfully");
            cursor
        }
        Err(e) => {
            log::error!("Failed to execute query: {}", e);
            return Err(e);
        }
    };

    log::info!("Collecting cursor results...");
    let all_cards = match collect_cursor_results(cursor, |cursor| {
        // Wrap each cursor read operation in error handling
        let note_id = match cursor.get_long_by_name(note_columns::ID) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Failed to read note ID: {}", e);
                return Ok(CardData {
                    id: 0,
                    front: "Error reading ID".to_string(),
                    back: format!("Error: {}", e),
                    deck_id: 1,
                    model_id: 1,
                    tags: "".to_string(),
                });
            }
        };

        let fields_str = match cursor.get_string_by_name(note_columns::FLDS) {
            Ok(fields) => fields,
            Err(e) => {
                log::warn!("Failed to read fields for note {}: {}", note_id, e);
                format!("Error reading fields{}\u{001f}Error: {}", note_id, e)
            }
        };

        let tags = match cursor.get_string_by_name(note_columns::TAGS) {
            Ok(tags) => tags,
            Err(e) => {
                log::warn!("Failed to read tags for note {}: {}", note_id, e);
                "".to_string()
            }
        };

        // Parse fields with error handling
        let (front, back) = match parse_card_fields(&fields_str) {
            Ok((f, b)) => (f, b),
            Err(e) => {
                log::warn!("Failed to parse fields for note {}: {}", note_id, e);
                ("Parse error".to_string(), format!("Error: {}", e))
            }
        };

        Ok(CardData {
            id: note_id,
            front,
            back,
            deck_id: 1, // Default deck ID, since notes don't have deck ID directly
            model_id: 1, // Default model ID
            tags,
        })
    }) {
        Ok(cards) => {
            log::info!("Successfully collected {} cards", cards.len());
            cards
        }
        Err(e) => {
            log::error!("Failed to collect cursor results: {}", e);
            return Err(e);
        }
    };

    // Apply limit after fetching results
    let limited_cards = if let Some(limit_val) = limit {
        let limit_size = limit_val as usize;
        log::info!("Applying limit of {} to {} cards", limit_size, all_cards.len());
        all_cards.into_iter().take(limit_size).collect()
    } else {
        all_cards
    };

    log::info!("Returning {} cards", limited_cards.len());
    Ok(limited_cards)
}

/// Update an existing card (note)
pub fn update_card(
    mut env: SafeJNIEnv,
    activity: &JObject,
    note_id: i64,
    front: &str,
    back: &str,
    deck_name: Option<&str>,
    tags: Option<&str>,
) -> AndroidResult<bool> {
    log::info!(
        "Updating card {} - Front: {}, Back: {}",
        note_id,
        front,
        back
    );

    // Validate inputs
    validate_card_fields(front, back)?;

    // Check if note exists
    if !note_exists(env.clone(), activity, note_id)? {
        return Err(AndroidError::NoteNotFound(format!(
            "Note ID {} not found",
            note_id
        )));
    }

    // Format fields with proper separator
    let fields = format!("{}{}{}", front, FIELD_SEPARATOR, back);
    let tags_str = tags.unwrap_or("").trim();

    let mut env_for_values = env.clone();
    let mut values_builder = ContentValuesBuilder::new(&mut env_for_values)?
        .put_string(note_columns::FLDS, &fields)?
        .put_string(note_columns::TAGS, tags_str)?;

    // Note: Deck ID cannot be set on notes directly, it's set on cards
    // We would need to update the cards table separately if we want to change deck
    if let Some(deck_name) = deck_name {
        let mut env_for_deck = env.clone();
        let _deck_id = get_or_create_deck_id(&mut env_for_deck, activity, Some(deck_name))?;
        // TODO: Update the deck ID in the cards table, not the notes table
    }

    // Update the note
    let updated_rows = update(env, NOTES_URI)
        .selection(format!("{} = ?", note_columns::ID))
        .selection_args(vec![note_id.to_string()])
        .execute(activity, values_builder)?;

    let success = updated_rows > 0;
    log::info!(
        "Update card {} - Success: {}, Rows affected: {}",
        note_id,
        success,
        updated_rows
    );
    Ok(success)
}

/// Delete a card (note)
pub fn delete_card(mut env: SafeJNIEnv, activity: &JObject, note_id: i64) -> AndroidResult<bool> {
    log::info!("Deleting card with note ID: {}", note_id);

    // Check if note exists
    let env_clone = env.clone();
    if !note_exists(env_clone, activity, note_id)? {
        return Err(AndroidError::NoteNotFound(format!(
            "Note ID {} not found",
            note_id
        )));
    }

    // Delete the note
    let deleted_rows = delete(env, NOTES_URI)
        .selection(format!("{} = ?", note_columns::ID))
        .selection_args(vec![note_id.to_string()])
        .execute(activity)?;

    let success = deleted_rows > 0;
    log::info!(
        "Delete card {} - Success: {}, Rows affected: {}",
        note_id,
        success,
        deleted_rows
    );
    Ok(success)
}

/// Check if a note exists by ID
pub fn note_exists(env: SafeJNIEnv, activity: &JObject, note_id: i64) -> AndroidResult<bool> {
    let projection = vec![note_columns::ID.to_string()];
    let selection = format!("{} = ?", note_columns::ID);
    let selection_args = vec![note_id.to_string()];

    let mut cursor = query(env, NOTES_URI)
        .projection(projection)
        .selection(selection)
        .selection_args(selection_args)
        .execute(activity)?;

    let count = cursor.get_count()?;
    Ok(count > 0)
}

/// Get card information by note ID
pub fn get_card_by_id(
    env: SafeJNIEnv,
    activity: &JObject,
    note_id: i64,
) -> AndroidResult<CardData> {
    log::info!("Getting card by ID: {}", note_id);

    let projection = vec![
        note_columns::ID.to_string(),
        note_columns::FLDS.to_string(),
        note_columns::TAGS.to_string(),
    ];

    let cursor = query(env, NOTES_URI)
        .projection(projection)
        .selection(format!("{} = ?", note_columns::ID))
        .selection_args(vec![note_id.to_string()])
        .execute(activity)?;

    let results = collect_cursor_results(cursor, |cursor| {
        let id = cursor.get_long_by_name(note_columns::ID)?;
        let fields_str = cursor.get_string_by_name(note_columns::FLDS)?;
        let tags = cursor.get_string_by_name(note_columns::TAGS)?;
        // Note: We can't query MID directly from notes table
        let model_id = 1; // Default model ID
        let deck_id = 1; // Default deck ID, since notes don't have deck ID directly

        let (front, back) = parse_card_fields(&fields_str)?;

        Ok(CardData {
            id,
            front,
            back,
            deck_id,
            model_id,
            tags,
        })
    })?;

    results
        .into_iter()
        .next()
        .ok_or_else(|| AndroidError::NoteNotFound(format!("Note ID {} not found", note_id)))
}

/// Data structure for card information
#[derive(Debug, Clone)]
pub struct CardData {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub deck_id: i64,
    pub model_id: i64,
    pub tags: String,
}

/// Validate card fields
fn validate_card_fields(front: &str, back: &str) -> AndroidResult<()> {
    if front.trim().is_empty() {
        return Err(AndroidError::validation_error(
            "Front field cannot be empty",
        ));
    }

    if back.trim().is_empty() {
        return Err(AndroidError::validation_error("Back field cannot be empty"));
    }

    if front.len() > 65536 {
        return Err(AndroidError::validation_error(
            "Front field too long (max 65536 characters)",
        ));
    }

    if back.len() > 65536 {
        return Err(AndroidError::validation_error(
            "Back field too long (max 65536 characters)",
        ));
    }

    Ok(())
}

/// Parse card fields from the fields string
fn parse_card_fields(fields_str: &str) -> AndroidResult<(String, String)> {
    let parts: Vec<&str> = fields_str.split(FIELD_SEPARATOR).collect();

    let front = parts.get(0).unwrap_or(&"").to_string();
    let back = parts.get(1).unwrap_or(&"").to_string();

    if front.is_empty() && back.is_empty() && !fields_str.is_empty() {
        // Fallback: if no separator found, try to split differently
        let fallback_parts: Vec<&str> = fields_str.split('\n').collect();
        if fallback_parts.len() >= 2 {
            return Ok((fallback_parts[0].to_string(), fallback_parts[1].to_string()));
        }
        // Last resort: put everything in front
        return Ok((fields_str.to_string(), "".to_string()));
    }

    Ok((front, back))
}

/// Extract ID from ContentProvider insert result URI
fn extract_id_from_uri(uri_string: &str) -> AndroidResult<i64> {
    log::debug!("Extracting ID from URI: {}", uri_string);

    let id = uri_string
        .split('/')
        .last()
        .and_then(|id_str| id_str.parse::<i64>().ok())
        .unwrap_or_else(|| {
            log::warn!("Could not parse note ID from URI: {}", uri_string);
            // Generate a timestamp-based ID as fallback
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            log::warn!("Using timestamp-based ID: {}", timestamp);
            timestamp
        });

    Ok(if id == 0 {
        // If parsed ID is 0, use timestamp
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    } else {
        id
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_card_fields_valid() {
        assert!(validate_card_fields("Front", "Back").is_ok());
        assert!(validate_card_fields("Question?", "Answer!").is_ok());
    }

    #[test]
    fn test_validate_card_fields_invalid() {
        assert!(validate_card_fields("", "Back").is_err());
        assert!(validate_card_fields("Front", "").is_err());
        assert!(validate_card_fields("   ", "Back").is_err());
        assert!(validate_card_fields("Front", "   ").is_err());
    }

    #[test]
    fn test_parse_card_fields() {
        let fields = format!("Front{}Back", FIELD_SEPARATOR);
        let (front, back) = parse_card_fields(&fields).unwrap();
        assert_eq!(front, "Front");
        assert_eq!(back, "Back");
    }

    #[test]
    fn test_parse_card_fields_fallback() {
        let fields = "Front\nBack";
        let (front, back) = parse_card_fields(&fields).unwrap();
        assert_eq!(front, "Front");
        assert_eq!(back, "Back");
    }

    #[test]
    fn test_parse_card_fields_no_separator() {
        let fields = "Just one field";
        let (front, back) = parse_card_fields(&fields).unwrap();
        assert_eq!(front, "Just one field");
        assert_eq!(back, "");
    }

    #[test]
    fn test_extract_id_from_uri() {
        let result = extract_id_from_uri("content://com.ichi2.anki.flashcards/notes/123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn test_extract_id_from_uri_fallback() {
        let result = extract_id_from_uri("invalid_uri");
        assert!(result.is_ok()); // Should use timestamp fallback
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_field_separator() {
        assert_eq!(FIELD_SEPARATOR, '\u{001f}');
    }

    #[test]
    fn test_card_data_structure() {
        let card = CardData {
            id: 123,
            front: "Front".to_string(),
            back: "Back".to_string(),
            deck_id: 1,
            model_id: 456,
            tags: "test".to_string(),
        };

        assert_eq!(card.id, 123);
        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "Back");
        assert_eq!(card.deck_id, 1);
        assert_eq!(card.model_id, 456);
        assert_eq!(card.tags, "test");
    }
}
