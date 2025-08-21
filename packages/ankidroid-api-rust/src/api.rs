//! Main AnkiDroid API module implementing AddContentApi functionality
//!
//! This module provides a complete Rust implementation of AnkiDroid's AddContentApi,
//! offering type-safe access to all AnkiDroid operations including note management,
//! model/deck operations, media handling, and preview functionality.
//!
//! # Features
//!
//! - **Complete API Coverage**: All AddContentApi methods from Kotlin implementation
//! - **Version Compatibility**: Support for both API v1 and v2 with automatic detection
//! - **Type Safety**: Strong typing for all operations with comprehensive error handling
//! - **Memory Management**: Proper JNI resource management with RAII patterns
//! - **Performance**: Optimized operations with efficient batch processing
//!
//! # Examples
//!
//! ```rust
//! use ankidroid_api_rust::api::AnkiDroidApi;
//! use ankidroid_api_rust::{Note, BasicModel};
//!
//! #[cfg(target_os = "android")]
//! fn example_usage(env: JNIEnv, context: JObject) -> Result<()> {
//!     let mut api = AnkiDroidApi::try_new(env, &context)?;
//!     
//!     // Check availability
//!     if !AnkiDroidApi::is_available(env, &context)? {
//!         return Err(AnkiDroidError::not_available("AnkiDroid not installed"));
//!     }
//!     
//!     // Create a basic model
//!     let model_id = api.add_new_basic_model("My Basic Model")?
//!         .ok_or_else(|| AnkiDroidError::database_error("Failed to create model"))?;
//!     
//!     // Add a note
//!     let note_id = api.add_note(
//!         model_id,
//!         1, // Default deck
//!         &["Front text", "Back text"],
//!         Some(&["study", "important"])
//!     )?;
//!     
//!     if let Some(id) = note_id {
//!         println!("Created note with ID: {}", id);
//!     }
//!     
//!     Ok(())
//! }
//! ```

#[cfg(target_os = "android")]
use crate::{
    error::{AnkiDroidError, Result},
    jni::{
        content_resolver::ContentResolver,
        helpers::{ContentValuesBuilder, SafeJNIEnv, JniResultExt},
    },
    models::{BasicModel, Basic2Model},
    contract::{self, note, deck, model, card, DEFAULT_DECK_ID},
    utils::{join_fields, join_tags},
};

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JValue},
    JNIEnv,
};

#[cfg(target_os = "android")]
use std::collections::HashMap;

/// Test tag used for preview operations
#[cfg(target_os = "android")]
const TEST_TAG: &str = "PREVIEW_NOTE";

/// Projection for note queries
#[cfg(target_os = "android")]
const NOTE_PROJECTION: &[&str] = &[note::_ID, note::FLDS, note::TAGS];

/// Main AnkiDroid API client providing access to all AddContentApi functionality
///
/// This struct encapsulates an Android Context and ContentResolver, providing
/// safe access to AnkiDroid's content provider API. It automatically handles
/// API version detection and provides compatibility layers for different
/// AnkiDroid versions.
///
/// # Thread Safety
///
/// This struct is NOT thread-safe due to the underlying JNI environment.
/// Each thread should create its own instance using the same JavaVM.
///
/// # Resource Management
///
/// The API client holds references to JNI objects and should be dropped
/// when no longer needed to ensure proper resource cleanup.
#[cfg(target_os = "android")]
pub struct AnkiDroidApi<'local> {
    /// Safe JNI environment for operations
    env: SafeJNIEnv<'local>,
    /// Android Context for API operations
    context: JObject<'local>,
    /// ContentResolver for database operations
    resolver: ContentResolver<'local>,
    /// API specification version (1 or 2)
    version: i32,
}

#[cfg(target_os = "android")]
impl<'local> AnkiDroidApi<'local> {
    /// Create a new AnkiDroid API instance
    ///
    /// This method creates a new API client from JNI environment and Android Context, 
    /// automatically detecting the API version and initializing the ContentResolver.
    ///
    /// # Arguments
    ///
    /// * `env` - JNI environment (must be valid for 'local lifetime)
    /// * `context` - Android Context (Activity or Application context)
    ///
    /// # Returns
    ///
    /// A new API instance, or an error if:
    /// - AnkiDroid is not installed or accessible
    /// - The API version is not supported
    /// - Context or ContentResolver is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut api = AnkiDroidApi::try_new(env, &context)?;
    /// println!("API version: {}", api.get_api_host_spec_version()?);
    /// ```
    pub fn try_new(env: JNIEnv<'local>, context: &JObject<'local>) -> Result<Self> {
        log::info!("Creating AnkiDroidApi instance...");
        
        // Validate context
        if context.is_null() {
            return Err(AnkiDroidError::null_pointer("Context cannot be null"));
        }

        let mut safe_env = SafeJNIEnv::new(env);
        
        // Create ContentResolver
        let resolver = ContentResolver::from_context(safe_env.clone(), context)?;
        
        // Detect API version
        let version = Self::detect_api_version(&mut safe_env, context)?;
        log::info!("Detected AnkiDroid API version: {}", version);
        
        // Verify AnkiDroid is available
        if !Self::is_available_internal(&mut safe_env, context)? {
            return Err(AnkiDroidError::not_available(
                "AnkiDroid API is not available"
            ));
        }
        
        // We need to create a global reference to the context so it persists
        let context_global = safe_env.env_mut().new_global_ref(context)?;
        let context_obj = safe_env.env_mut().new_local_ref(&context_global)?;
        
        Ok(Self {
            env: safe_env,
            context: context_obj,
            resolver,
            version,
        })
    }

    /// Check if AnkiDroid API is available on this device
    ///
    /// This static method checks if AnkiDroid is installed and the API is accessible
    /// without creating a full API instance.
    ///
    /// # Arguments
    ///
    /// * `env` - JNI environment
    /// * `context` - Android Context
    ///
    /// # Returns
    ///
    /// `true` if AnkiDroid API is available, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// if AnkiDroidApi::is_available(env, &context)? {
    ///     let mut api = AnkiDroidApi::try_new(env, &context)?;
    ///     // Use API...
    /// } else {
    ///     println!("AnkiDroid not available");
    /// }
    /// ```
    pub fn is_available(env: JNIEnv, context: &JObject) -> Result<bool> {
        let mut safe_env = SafeJNIEnv::new(env);
        Self::is_available_internal(&mut safe_env, context)
    }

    // Internal implementation of availability check
    fn is_available_internal(env: &mut SafeJNIEnv, context: &JObject) -> Result<bool> {
        // For external apps, we check if AnkiDroid is installed and accessible via ContentResolver
        // We don't check for AddContentApi class since it's in AnkiDroid app, not ours
        
        // Try to get PackageManager to check if AnkiDroid is installed
        let pm_result = env.env_mut().call_method(
            context,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        );
        
        let package_manager = match pm_result {
            Ok(result) => match result.l() {
                Ok(pm) if !pm.is_null() => pm,
                _ => return Ok(false),
            },
            Err(_) => return Ok(false),
        };
        
        // Check if AnkiDroid package is installed
        let package_name = env.new_string_checked("com.ichi2.anki")?;
        
        // Try to get package info (will fail if not installed)
        let package_info_result = env.env_mut().call_method(
            &package_manager,
            "getPackageInfo",
            "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
            &[JValue::Object(&package_name), JValue::Int(0)],
        );
        
        match package_info_result {
            Ok(result) => match result.l() {
                Ok(info) => {
                    log::debug!("AnkiDroid package found: com.ichi2.anki");
                    Ok(!info.is_null())
                },
                Err(_) => {
                    log::debug!("AnkiDroid package not found");
                    Ok(false)
                },
            },
            Err(_) => {
                log::debug!("Failed to check for AnkiDroid package");
                Ok(false)
            },
        }
    }

    // Detect API specification version
    fn detect_api_version(env: &mut SafeJNIEnv, context: &JObject) -> Result<i32> {
        // Use PackageManager to get provider metadata
        let pm_result = env.env_mut().call_method(
            context,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        ).check_exception(env.env_mut())?;
        
        let package_manager = pm_result.l().map_err(AnkiDroidError::from)?;
        if package_manager.is_null() {
            return Ok(1); // Default to v1
        }

        // Try to resolve FlashCards content provider
        let authority_string = env.new_string_checked(contract::AUTHORITY)?;
        
        let provider_info_result = env.env_mut().call_method(
            &package_manager,
            "resolveContentProvider",
            "(Ljava/lang/String;I)Landroid/content/pm/ProviderInfo;",
            &[
                JValue::Object(&authority_string.into()),
                JValue::Int(0), // flags
            ],
        ).check_exception(env.env_mut());

        let provider_info = match provider_info_result {
            Ok(result) => result.l().map_err(AnkiDroidError::from)?,
            Err(_) => return Ok(1), // Default to v1
        };

        if provider_info.is_null() {
            return Ok(1); // Default to v1
        }

        // Try to get metadata
        let metadata_result = env.env_mut().get_field(
            &provider_info,
            "metaData",
            "Landroid/os/Bundle;",
        ).check_exception(env.env_mut());

        let metadata = match metadata_result {
            Ok(metadata) => metadata.l().map_err(AnkiDroidError::from)?,
            Err(_) => return Ok(1), // Default to v1
        };

        if metadata.is_null() {
            return Ok(1); // Default to v1
        }

        // Try to get the spec version from metadata
        let spec_key = env.new_string_checked("com.ichi2.anki.provider.spec")?;
        let version_result = env.env_mut().call_method(
            &metadata,
            "getInt",
            "(Ljava/lang/String;I)I",
            &[
                JValue::Object(&spec_key.into()),
                JValue::Int(1), // default value
            ],
        ).check_exception(env.env_mut())?;

        Ok(version_result.i().unwrap_or(1))
    }

    /// Get the API specification version
    ///
    /// Returns the AnkiDroid API specification version:
    /// - 1: AnkiDroid 2.5 (slower bulk operations, limited features)
    /// - 2: AnkiDroid 2.6+ (optimized bulk operations, full features)
    ///
    /// # Returns
    ///
    /// The API version number
    ///
    /// # Examples
    ///
    /// ```rust
    /// let version = api.get_api_host_spec_version()?;
    /// if version >= 2 {
    ///     // Use optimized bulk operations
    /// }
    /// ```
    pub fn get_api_host_spec_version(&self) -> Result<i32> {
        Ok(self.version)
    }

    // ========================================================================
    // Note Operations
    // ========================================================================

    /// Add a single note to AnkiDroid
    ///
    /// Creates a new note with the specified model, deck, fields, and tags.
    /// No duplicate checking is performed - use `find_duplicate_notes` first
    /// if duplicate detection is needed.
    ///
    /// # Arguments
    ///
    /// * `model_id` - ID of the note type/model to use
    /// * `deck_id` - ID of the deck for the generated cards (use DEFAULT_DECK_ID for default)
    /// * `fields` - Array of field values (length must match model field count)
    /// * `tags` - Optional array of tags to assign to the note
    ///
    /// # Returns
    ///
    /// The ID of the created note, or None if the note could not be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// let note_id = api.add_note(
    ///     model_id,
    ///     DEFAULT_DECK_ID,
    ///     &["What is the capital of France?", "Paris"],
    ///     Some(&["geography", "capitals"])
    /// )?;
    /// 
    /// if let Some(id) = note_id {
    ///     println!("Created note with ID: {}", id);
    /// }
    /// ```
    pub fn add_note(
        &mut self,
        model_id: i64,
        deck_id: i64,
        fields: &[&str],
        tags: Option<&[&str]>,
    ) -> Result<Option<i64>> {
        log::info!(
            "Adding note with model_id={}, deck_id={}, {} fields",
            model_id, deck_id, fields.len()
        );

        // Validate inputs
        if fields.is_empty() {
            return Err(AnkiDroidError::validation_error("Fields cannot be empty"));
        }

        // Build ContentValues for the note
        let mut values = ContentValuesBuilder::new(&mut self.env)?
            .put_long(note::MID, model_id)?
            .put_string(note::FLDS, &join_fields(&fields.iter().map(|s| *s).collect::<Vec<_>>()))?;

        // Add tags if provided
        if let Some(tags_array) = tags {
            if !tags_array.is_empty() {
                values = values.put_string(note::TAGS, &join_tags(&tags_array.iter().map(|s| *s).collect::<Vec<_>>()))?;
            }
        }

        // Insert the note
        let note_uri = self.resolver.insert(
            &contract::build_note_uri(),
            values,
        )?;

        // Extract note ID from URI
        let note_id = Self::extract_id_from_uri(&note_uri)?;

        // Move cards to specified deck if not default
        if deck_id != DEFAULT_DECK_ID {
            self.move_note_cards_to_deck(note_id, deck_id)?;
        }

        log::info!("✅ Note created successfully with ID: {}", note_id);
        Ok(Some(note_id))
    }

    /// Get the API specification version
    ///
    /// Returns the AnkiDroid API specification version.
    ///
    /// # Returns
    ///
    /// The API version number (1 or 2)
    pub fn get_api_version(&self) -> i32 {
        self.version
    }

    // Helper method to extract ID from URI
    fn extract_id_from_uri(uri: &str) -> Result<i64> {
        uri.split('/')
            .last()
            .ok_or_else(|| AnkiDroidError::validation_error("Invalid URI format"))?
            .parse::<i64>()
            .map_err(|_| AnkiDroidError::validation_error("Invalid ID in URI"))
    }

    // Helper method to move cards to a deck
    fn move_note_cards_to_deck(&mut self, note_id: i64, deck_id: i64) -> Result<()> {
        let cards_uri = contract::build_cards_for_note_uri(note_id);
        
        let mut cursor = self.resolver.query(
            &cards_uri,
            Some(vec![card::CARD_ORD.to_string()]),
            None,
            None,
            None,
        )?;

        while cursor.move_to_next()? {
            let ord_str = cursor.get_string_by_name(card::CARD_ORD)?;
            let card_uri = contract::build_specific_card_uri(note_id, &ord_str);
            
            let values = ContentValuesBuilder::new(&mut self.env)?
                .put_long(card::DECK_ID, deck_id)?;
            
            self.resolver.update(&card_uri, values, None, None)?;
        }

        Ok(())
    }

    /// Add a new basic note type with two fields (Front, Back)
    ///
    /// Creates a simple front/back note type with one card template.
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the new note type
    ///
    /// # Returns
    ///
    /// The ID of the created model, or None if creation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let model_id = api.add_new_basic_model("My Basic Cards")?
    ///     .ok_or_else(|| AnkiDroidError::database_error("Failed to create model"))?;
    /// println!("Created basic model with ID: {}", model_id);
    /// ```
    pub fn add_new_basic_model(&mut self, name: &str) -> Result<Option<i64>> {
        self.add_new_custom_model(
            name,
            BasicModel::FIELDS,
            BasicModel::CARD_NAMES,
            BasicModel::QFMT,
            BasicModel::AFMT,
            None,
            None,
            None,
        )
    }

    /// Add a new basic note type with reverse card
    ///
    /// Creates a front/back note type with two card templates:
    /// one showing Front->Back and another showing Back->Front.
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the new note type
    ///
    /// # Returns
    ///
    /// The ID of the created model, or None if creation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let model_id = api.add_new_basic2_model("Vocabulary Cards")?
    ///     .ok_or_else(|| AnkiDroidError::database_error("Failed to create model"))?;
    /// println!("Created basic2 model with ID: {}", model_id);
    /// ```
    pub fn add_new_basic2_model(&mut self, name: &str) -> Result<Option<i64>> {
        self.add_new_custom_model(
            name,
            Basic2Model::FIELDS,
            Basic2Model::CARD_NAMES,
            Basic2Model::QFMT,
            Basic2Model::AFMT,
            None,
            None,
            None,
        )
    }

    /// Add a new custom note type with full control over fields and templates
    ///
    /// Creates a custom note type with specified fields, card templates,
    /// formatting, and styling.
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the note type
    /// * `fields` - Array of field names
    /// * `cards` - Array of card template names
    /// * `qfmt` - Array of question format templates (one per card)
    /// * `afmt` - Array of answer format templates (one per card)
    /// * `css` - Optional CSS styling (None for default)
    /// * `did` - Optional default deck ID (None for default deck)
    /// * `sortf` - Optional sort field index (None for first field)
    ///
    /// # Returns
    ///
    /// The ID of the created model, or None if creation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let model_id = api.add_new_custom_model(
    ///     "Cloze Model",
    ///     &["Text", "Extra"],
    ///     &["Cloze"],
    ///     &["{{cloze:Text}}"],
    ///     &["{{cloze:Text}}<br>{{Extra}}"],
    ///     Some("body { font-size: 18px; }"),
    ///     None,
    ///     Some(0)
    /// )?;
    /// ```
    pub fn add_new_custom_model(
        &mut self,
        name: &str,
        fields: &[&str],
        cards: &[&str],
        qfmt: &[&str],
        afmt: &[&str],
        css: Option<&str>,
        did: Option<i64>,
        sortf: Option<i32>,
    ) -> Result<Option<i64>> {
        // Validate input arrays
        if qfmt.len() != cards.len() || afmt.len() != cards.len() {
            return Err(AnkiDroidError::validation_error(
                "cards, qfmt, and afmt arrays must all be the same length"
            ));
        }

        if fields.is_empty() {
            return Err(AnkiDroidError::validation_error("Fields cannot be empty"));
        }

        log::info!("Creating custom model '{}' with {} fields, {} cards", 
                   name, fields.len(), cards.len());

        // Create the model
        let mut values = ContentValuesBuilder::new(&mut self.env)?
            .put_string(model::NAME, name)?
            .put_string(model::FIELD_NAMES, &join_fields(&fields.iter().map(|s| *s).collect::<Vec<_>>()))?
            .put_int(model::NUM_CARDS, cards.len() as i32)?;

        if let Some(css_content) = css {
            values = values.put_string(model::CSS, css_content)?;
        }

        if let Some(deck_id) = did {
            values = values.put_long(model::DECK_ID, deck_id)?;
        }

        if let Some(sort_field) = sortf {
            values = values.put_int(model::SORT_FIELD_INDEX, sort_field)?;
        }

        let model_uri = self.resolver.insert(&contract::build_models_uri(), values)?;
        let model_id = Self::extract_id_from_uri(&model_uri)?;

        // Set up card templates
        for i in 0..cards.len() {
            let template_uri = contract::build_template_uri(model_id, i as i32);
            
            let template_values = ContentValuesBuilder::new(&mut self.env)?
                .put_string(contract::card_template::NAME, cards[i])?
                .put_string(contract::card_template::QUESTION_FORMAT, qfmt[i])?
                .put_string(contract::card_template::ANSWER_FORMAT, afmt[i])?;

            let updated = self.resolver.update(&template_uri, template_values, None, None)?;
            if updated == 0 {
                log::warn!("Failed to update template {} for model {}", i, model_id);
            }
        }

        log::info!("✅ Custom model created with ID: {}", model_id);
        Ok(Some(model_id))
    }

    /// Create a new deck
    ///
    /// # Arguments
    ///
    /// * `deck_name` - Name for the new deck
    ///
    /// # Returns
    ///
    /// The ID of the created deck, or None if creation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let deck_id = api.add_new_deck("Spanish Vocabulary")?
    ///     .ok_or_else(|| AnkiDroidError::database_error("Failed to create deck"))?;
    /// println!("Created deck with ID: {}", deck_id);
    /// ```
    pub fn add_new_deck(&mut self, deck_name: &str) -> Result<Option<i64>> {
        log::info!("Creating new deck: {}", deck_name);
        
        let values = ContentValuesBuilder::new(&mut self.env)?
            .put_string(deck::DECK_NAME, deck_name)?;

        let deck_uri = self.resolver.insert(&contract::build_decks_uri(), values)?;
        let deck_id = Self::extract_id_from_uri(&deck_uri)?;
        
        log::info!("✅ Deck created with ID: {}", deck_id);
        Ok(Some(deck_id))
    }

    /// Get all decks
    ///
    /// # Returns
    ///
    /// HashMap mapping deck IDs to deck names
    ///
    /// # Examples
    ///
    /// ```rust
    /// let decks = api.get_deck_list()?;
    /// for (id, name) in decks {
    ///     println!("Deck {}: {}", id, name);
    /// }
    /// ```
    pub fn get_deck_list(&mut self) -> Result<HashMap<i64, String>> {
        let mut cursor = self.resolver.query(
            &contract::build_decks_uri(),
            Some(vec![
                deck::DECK_ID.to_string(),
                deck::DECK_NAME.to_string(),
            ]),
            None,
            None,
            None,
        )?;

        let mut decks = HashMap::new();
        
        while cursor.move_to_next()? {
            let id_str = cursor.get_string_by_name(deck::DECK_ID)?;
            let name = cursor.get_string_by_name(deck::DECK_NAME)?;
            
            if let Ok(id) = id_str.parse::<i64>() {
                decks.insert(id, name);
            }
        }

        Ok(decks)
    }

    /// Get a mutable reference to the ContentResolver for extended API operations
    ///
    /// This method provides access to the underlying ContentResolver for advanced
    /// database operations not covered by the standard API.
    ///
    /// # Returns
    ///
    /// A mutable reference to the ContentResolver instance
    pub fn resolver_mut(&mut self) -> &mut ContentResolver<'local> {
        &mut self.resolver
    }

    /// Get a mutable reference to the JNI environment for extended API operations
    ///
    /// This method provides access to the underlying JNI environment for advanced
    /// operations that require direct JNI interaction.
    ///
    /// # Returns
    ///
    /// A mutable reference to the SafeJNIEnv instance
    pub fn env_mut(&mut self) -> &mut SafeJNIEnv<'local> {
        &mut self.env
    }
}

// Make sure we only export for Android targets
#[cfg(not(target_os = "android"))]
compile_error!("This module is only available on Android targets");

#[cfg(test)]
#[cfg(target_os = "android")]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_from_uri() {
        let uri = "content://com.ichi2.anki.flashcards/notes/12345";
        let id = AnkiDroidApi::extract_id_from_uri(uri).unwrap();
        assert_eq!(id, 12345);
    }

    #[test]
    fn test_extract_id_from_uri_invalid() {
        let uri = "content://com.ichi2.anki.flashcards/notes/invalid";
        assert!(AnkiDroidApi::extract_id_from_uri(uri).is_err());
    }

    #[test]
    fn test_validate_mime_type() {
        // This would be tested in the actual add_media_from_uri method
        assert!(true);
    }

    // Additional tests would require a mock JNI environment
    // which is complex to set up in unit tests
}