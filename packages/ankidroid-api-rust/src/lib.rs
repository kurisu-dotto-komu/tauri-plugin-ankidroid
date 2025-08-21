//! # AnkiDroid API Rust
//! 
//! Type-safe Rust bindings for the AnkiDroid API FlashCardsContract.
//! 
//! This crate provides comprehensive access to AnkiDroid's content provider API,
//! allowing Android applications to interact with AnkiDroid's database and functionality
//! in a type-safe manner. The API includes support for managing notes, cards, decks,
//! models (note types), and performing review operations.
//! 
//! ## Features
//! 
//! - **Type Safety**: All API operations use strongly-typed Rust structs and enums
//! - **Error Handling**: Comprehensive error types with detailed error information
//! - **Android Integration**: Full JNI support for Android applications
//! - **Content Provider Access**: Direct access to AnkiDroid's FlashCardsContract
//! - **Serialization Support**: Built-in serde support for all data types
//! 
//! ## Platform Support
//! 
//! This crate is designed specifically for Android applications. The JNI and API
//! modules are only available when compiling for Android targets.
//! 
//! ## Basic Usage
//! 
//! ```rust,ignore
//! // This example only works on Android targets
//! use ankidroid_api_rust::{AnkiDroidApi, Note, Result};
//! 
//! #[cfg(target_os = "android")]
//! fn example_usage(vm: jni::JavaVM) -> Result<()> {
//!     let api = AnkiDroidApi::new(vm);
//!     
//!     // Check if AnkiDroid is available
//!     if !api.is_available()? {
//!         return Err(AnkiDroidError::not_available("AnkiDroid not installed"));
//!     }
//!     
//!     // Create a new note
//!     let note = Note::builder(123) // model ID
//!         .fields(vec!["Front text".to_string(), "Back text".to_string()])
//!         .tag("my-tag".to_string())
//!         .build();
//!     
//!     // Add the note to AnkiDroid
//!     let note_id = api.add_note(&note)?;
//!     println!("Added note with ID: {}", note_id);
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## URI Constants
//! 
//! The crate provides all necessary constants for interacting with AnkiDroid's
//! content provider:
//! 
//! ```rust
//! use ankidroid_api_rust::{AUTHORITY, READ_WRITE_PERMISSION, DEFAULT_DECK_ID};
//! 
//! assert_eq!(AUTHORITY, "com.ichi2.anki.flashcards");
//! assert_eq!(DEFAULT_DECK_ID, 1);
//! ```
//! 
//! ## Error Handling
//! 
//! All fallible operations return a `Result<T, AnkiDroidError>`. The error type
//! provides detailed information about what went wrong and includes helper methods
//! for categorizing errors:
//! 
//! ```rust
//! use ankidroid_api_rust::{AnkiDroidError, Result};
//! 
//! fn handle_error(result: Result<()>) {
//!     match result {
//!         Ok(()) => println!("Success!"),
//!         Err(err) => {
//!             println!("Error: {}", err);
//!             println!("Category: {}", err.category());
//!             
//!             if err.is_recoverable() {
//!                 println!("This error can be retried");
//!             }
//!         }
//!     }
//! }
//! ```

// Core module declarations
pub mod contract;
pub mod error;
pub mod models;
pub mod utils;

// Android-specific modules (only available on Android targets)
#[cfg(target_os = "android")]
pub mod jni;

#[cfg(target_os = "android")]
pub mod api;

#[cfg(target_os = "android")]
pub mod extended;

// Re-exports for convenience and public API
pub use error::{AnkiDroidError, Result};
pub use models::{
    Note, Card, Deck, Model, NoteInfo, Ease,
    NoteBuilder, BasicModel, Basic2Model
};
pub use utils::{
    join_fields, split_fields, join_tags, split_tags,
    strip_html, strip_html_entities
};

#[cfg(target_os = "android")]
pub use utils::field_checksum;
pub use contract::{
    AUTHORITY, READ_WRITE_PERMISSION, DEFAULT_DECK_ID, FIELD_SEPARATOR,
    // URI builder functions
    build_authority_uri, build_note_uri, build_note_by_id_uri, build_cards_for_note_uri,
    build_specific_card_uri, build_notes_v2_uri, build_models_uri, build_model_by_id_uri,
    build_current_model_uri, build_templates_uri, build_decks_uri, build_selected_deck_uri,
    build_schedule_uri, build_media_uri,
    // Column constant modules
    note, card, deck, model, card_template, review_info, anki_media
};

// Android-specific re-exports
#[cfg(target_os = "android")]
pub use api::AnkiDroidApi;

#[cfg(target_os = "android")]
pub use extended::AnkiDroidApiExtended;

/// Version information for this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name for logging and identification
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// Feature flags for optional functionality
pub mod features {
    //! Feature flags for conditional compilation
    
    /// Whether JNI support is available (Android only)
    pub const HAS_JNI: bool = cfg!(target_os = "android");
    
    /// Whether API support is available (Android only) 
    pub const HAS_API: bool = cfg!(target_os = "android");
    
    /// Whether serde support is enabled (always true in this version)
    pub const HAS_SERDE: bool = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(CRATE_NAME, "ankidroid-api-rust");
    }
    
    #[test]
    fn test_feature_flags() {
        // JNI should only be available on Android
        #[cfg(target_os = "android")]
        assert!(features::HAS_JNI);
        
        #[cfg(not(target_os = "android"))]
        assert!(!features::HAS_JNI);
        
        // API should only be available on Android
        #[cfg(target_os = "android")]
        assert!(features::HAS_API);
        
        #[cfg(not(target_os = "android"))]
        assert!(!features::HAS_API);
        
        // Serde should always be available
        assert!(features::HAS_SERDE);
    }
    
    #[test]
    fn test_public_api_availability() {
        // These should always be available
        use crate::{AUTHORITY, DEFAULT_DECK_ID, VERSION, AnkiDroidError};
        
        assert_eq!(AUTHORITY, "com.ichi2.anki.flashcards");
        assert_eq!(DEFAULT_DECK_ID, 1);
        assert!(!VERSION.is_empty());
        
        // Test error creation
        let err = AnkiDroidError::validation_error("test");
        assert!(matches!(err, AnkiDroidError::ValidationError(_)));
    }
    
    #[test]
    fn test_model_builders() {
        let note = Note::builder(123)
            .field("Front".to_string())
            .field("Back".to_string())
            .tag("test".to_string())
            .build();
            
        assert_eq!(note.mid, 123);
        assert_eq!(note.fields.len(), 2);
        assert_eq!(note.tags.len(), 1);
        assert_eq!(note.key(), "Front");
    }
    
    #[cfg(target_os = "android")]
    #[test]
    fn test_android_specific_imports() {
        // These should only be available on Android
        use crate::AnkiDroidApi;
        // If this compiles, the import is available
    }
}