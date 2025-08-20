# AnkiDroid API Rust Specification

This document provides a comprehensive specification of the Rust implementation of the AnkiDroid API. The Rust API provides type-safe bindings for AnkiDroid's FlashCardsContract content provider.

## Overview

The Rust API consists of:
1. **High-Level API** (`AnkiDroidApi`): Convenient methods for common operations
2. **Data Models**: Strongly-typed structs representing AnkiDroid entities
3. **Error Handling**: Comprehensive error types with categorization
4. **Utilities**: Helper functions for data manipulation
5. **Constants**: All necessary URIs, permissions, and identifiers

## API Structure

### Main API Class

```rust
pub struct AnkiDroidApi {
    // JNI JavaVM for Android operations
}

impl AnkiDroidApi {
    pub fn new(vm: jni::JavaVM) -> Self
    pub fn is_available(&self) -> Result<bool>
}
```

## Note Operations

### add_note

Adds a single note to AnkiDroid.

```rust
pub fn add_note(&self, note: &Note) -> Result<i64>
```

**Parameters:**
- `note: &Note` - The note to add, containing model ID, fields, and tags

**Returns:** 
- `Ok(i64)` - The ID of the created note
- `Err(AnkiDroidError)` - Error if operation fails

**Errors:**
- `NotAvailable` - AnkiDroid not installed
- `Permission` - Missing permissions
- `Validation` - Invalid note data
- `Database` - Database constraint violation

**Example:**
```rust
let note = Note::builder(model_id)
    .field("Front text".to_string())
    .field("Back text".to_string())
    .tag("my-tag".to_string())
    .build();
    
let note_id = api.add_note(&note)?;
```

### add_notes

Bulk adds multiple notes.

```rust
pub fn add_notes(&self, deck_id: i64, notes: &[Note]) -> Result<usize>
```

**Parameters:**
- `deck_id: i64` - Target deck ID (use `DEFAULT_DECK_ID` for default)
- `notes: &[Note]` - Slice of notes to add

**Returns:**
- `Ok(usize)` - Number of successfully added notes
- `Err(AnkiDroidError)` - Error if operation fails

### get_note

Retrieves a note by ID.

```rust
pub fn get_note(&self, note_id: i64) -> Result<NoteInfo>
```

**Parameters:**
- `note_id: i64` - The note ID to retrieve

**Returns:**
- `Ok(NoteInfo)` - Complete note information
- `Err(AnkiDroidError)` - Error if note not found or operation fails

### update_note_fields

Updates the fields of an existing note.

```rust
pub fn update_note_fields(&self, note_id: i64, fields: &[String]) -> Result<()>
```

**Parameters:**
- `note_id: i64` - The note ID to update
- `fields: &[String]` - New field values (must match model field count)

### update_note_tags

Updates the tags of an existing note.

```rust
pub fn update_note_tags(&self, note_id: i64, tags: &[String]) -> Result<()>
```

**Parameters:**
- `note_id: i64` - The note ID to update  
- `tags: &[String]` - New tag list

### find_duplicate_notes

Finds notes with duplicate first fields.

```rust
pub fn find_duplicate_notes(&self, model_id: i64, key: &str) -> Result<Vec<NoteInfo>>
```

**Parameters:**
- `model_id: i64` - Model ID to search within
- `key: &str` - First field value to search for

**Returns:**
- `Ok(Vec<NoteInfo>)` - List of matching notes
- `Err(AnkiDroidError)` - Error if operation fails

## Model Operations

### add_new_basic_model

Creates a new basic (front/back) model.

```rust
pub fn add_new_basic_model(&self, name: &str) -> Result<Option<i64>>
```

**Parameters:**
- `name: &str` - Name for the new model

**Returns:**
- `Ok(Some(i64))` - Model ID if successful
- `Ok(None)` - If model creation failed
- `Err(AnkiDroidError)` - Error if operation fails

### add_new_basic2_model

Creates a new basic model with reverse card.

```rust
pub fn add_new_basic2_model(&self, name: &str) -> Result<Option<i64>>
```

### add_new_custom_model

Creates a fully customized model.

```rust
pub fn add_new_custom_model(
    &self,
    name: &str,
    fields: &[String],
    cards: &[String],
    qfmt: &[String],
    afmt: &[String],
    css: Option<&str>,
    deck_id: Option<i64>,
    sort_field: Option<i32>
) -> Result<Option<i64>>
```

**Parameters:**
- `name: &str` - Model name
- `fields: &[String]` - Field names
- `cards: &[String]` - Card template names
- `qfmt: &[String]` - Question format templates
- `afmt: &[String]` - Answer format templates
- `css: Option<&str>` - Custom CSS (optional)
- `deck_id: Option<i64>` - Default deck (optional)
- `sort_field: Option<i32>` - Sort field index (optional)

### current_model_id

Gets the currently selected model ID.

```rust
pub fn current_model_id(&self) -> Result<i64>
```

### get_field_list

Gets field names for a model.

```rust
pub fn get_field_list(&self, model_id: i64) -> Result<Vec<String>>
```

### get_model_list

Gets all available models.

```rust
pub fn get_model_list(&self, min_fields: Option<usize>) -> Result<HashMap<i64, String>>
```

**Parameters:**
- `min_fields: Option<usize>` - Minimum number of fields (default: 1)

**Returns:**
- `Ok(HashMap<i64, String>)` - Map of model ID to model name

### get_model_name

Gets the name of a model by ID.

```rust
pub fn get_model_name(&self, model_id: i64) -> Result<String>
```

## Deck Operations

### add_new_deck

Creates a new deck.

```rust
pub fn add_new_deck(&self, name: &str) -> Result<Option<i64>>
```

### get_deck_list

Gets all available decks.

```rust
pub fn get_deck_list(&self) -> Result<HashMap<i64, String>>
```

### get_selected_deck_name

Gets the name of the currently selected deck.

```rust
pub fn get_selected_deck_name(&self) -> Result<String>
```

### get_deck_name

Gets deck name by ID.

```rust
pub fn get_deck_name(&self, deck_id: i64) -> Result<String>
```

## Media Operations

### add_media_from_uri

Adds a media file to the collection.

```rust
pub fn add_media_from_uri(
    &self, 
    file_uri: &str, 
    preferred_name: &str, 
    mime_type: &str
) -> Result<String>
```

**Parameters:**
- `file_uri: &str` - URI to the media file
- `preferred_name: &str` - Desired filename (without extension)
- `mime_type: &str` - "audio" or "image"

**Returns:**
- `Ok(String)` - Formatted media reference for card fields
- `Err(AnkiDroidError)` - Error if operation fails

## Data Models

### Note

Represents a note to be added or updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub mid: i64,           // Model ID
    pub fields: Vec<String>, // Field values  
    pub tags: Vec<String>,  // Tags
    pub guid: Option<String>, // GUID (auto-generated if None)
}

impl Note {
    pub fn builder(mid: i64) -> NoteBuilder
    pub fn key(&self) -> &str  // Returns first field (for duplicate detection)
}
```

**Builder Pattern:**
```rust
let note = Note::builder(model_id)
    .field("Front".to_string())
    .field("Back".to_string())
    .tag("tag1".to_string())
    .tag("tag2".to_string())
    .build();
```

### NoteInfo

Complete note information from the database.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInfo {
    pub id: i64,            // Note ID
    pub guid: String,       // Globally unique identifier
    pub mid: i64,           // Model ID
    pub mod_time: i64,      // Modification timestamp
    pub usn: i32,           // Update sequence number
    pub tags: Vec<String>,  // Tags
    pub fields: Vec<String>, // Field values
    pub sort_field: String, // Sort field value
    pub checksum: i64,      // First field checksum
    pub flags: i32,         // Flags
    pub data: String,       // Additional data
}
```

### Card

Represents a card instance of a note.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub note_id: i64,           // Parent note ID
    pub ord: i32,               // Card ordinal (0-based)
    pub name: String,           // Card name
    pub deck_id: i64,           // Deck ID
    pub question: String,       // Question HTML
    pub answer: String,         // Answer HTML
    pub question_simple: String, // Question without CSS
    pub answer_simple: String,  // Answer without CSS
    pub answer_pure: String,    // Pure answer text
}
```

### Deck

Represents a deck containing cards.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,                    // Deck ID
    pub name: String,               // Deck name
    pub description: String,        // Deck description
    pub counts: Vec<i32>,          // [learn, review, new] counts
    pub options: serde_json::Value, // Deck options JSON
    pub is_dynamic: bool,          // True if filtered deck
}
```

### Model

Defines the structure and rendering of notes.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: i64,                    // Model ID
    pub name: String,               // Model name
    pub field_names: Vec<String>,   // Field names
    pub num_cards: i32,             // Number of card templates
    pub css: String,                // Shared CSS
    pub deck_id: i64,               // Default deck ID
    pub sort_field_index: i32,      // Sort field index
    pub model_type: i32,            // 0=normal, 1=cloze
    pub latex_pre: String,          // LaTeX preamble
    pub latex_post: String,         // LaTeX postamble
}
```

## Error Handling

### AnkiDroidError

All operations return `Result<T, AnkiDroidError>`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnkiDroidError {
    #[error("AnkiDroid not available: {0}")]
    NotAvailable(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}
```

### Error Categories

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    NotAvailable,   // AnkiDroid not installed/accessible
    Permission,     // Missing permissions
    Validation,     // Invalid input data
    NotFound,       // Resource doesn't exist
    Database,       // Database operation failed
    Operation,      // General operation failure
    System,         // System-level error
}

impl AnkiDroidError {
    pub fn category(&self) -> ErrorCategory
    pub fn is_recoverable(&self) -> bool
}
```

### Error Creation Methods

```rust
impl AnkiDroidError {
    pub fn not_available(msg: &str) -> Self
    pub fn permission_denied(msg: &str) -> Self
    pub fn validation_error(msg: &str) -> Self
    pub fn not_found(msg: &str) -> Self
    pub fn database_error(msg: &str) -> Self
    pub fn operation_failed(msg: &str) -> Self
    pub fn system_error(msg: &str) -> Self
}
```

## Utility Functions

### Field Operations

```rust
/// Join fields with AnkiDroid's field separator (0x1F)
pub fn join_fields(fields: &[String]) -> String

/// Split fields by AnkiDroid's field separator
pub fn split_fields(fields: &str) -> Vec<String>
```

### Tag Operations

```rust
/// Join tags with spaces, replacing spaces in tags with underscores
pub fn join_tags(tags: &[String]) -> String

/// Split tags by whitespace
pub fn split_tags(tags: &str) -> Vec<String>
```

### HTML Processing

```rust
/// Strip HTML tags from text
pub fn strip_html(html: &str) -> String

/// Strip HTML entities from text  
pub fn strip_html_entities(text: &str) -> String
```

### Checksum Operations (Android only)

```rust
#[cfg(target_os = "android")]
/// Calculate field checksum for duplicate detection
pub fn field_checksum(data: &str) -> i64
```

## Constants

### Core Constants

```rust
pub const AUTHORITY: &str = "com.ichi2.anki.flashcards";
pub const READ_WRITE_PERMISSION: &str = "com.ichi2.anki.permission.READ_WRITE_PERMISSION";
pub const DEFAULT_DECK_ID: i64 = 1;
pub const FIELD_SEPARATOR: char = '\u{001f}';
```

### URI Building Functions

```rust
pub fn build_authority_uri() -> String
pub fn build_note_uri() -> String
pub fn build_note_by_id_uri(note_id: i64) -> String
pub fn build_cards_for_note_uri(note_id: i64) -> String
pub fn build_specific_card_uri(note_id: i64, ord: i32) -> String
pub fn build_notes_v2_uri() -> String
pub fn build_models_uri() -> String
pub fn build_model_by_id_uri(model_id: i64) -> String
pub fn build_current_model_uri() -> String
pub fn build_templates_uri(model_id: i64) -> String
pub fn build_decks_uri() -> String
pub fn build_selected_deck_uri() -> String
pub fn build_schedule_uri() -> String
pub fn build_media_uri() -> String
```

### Column Constants

The crate provides modules with column name constants for each entity type:

```rust
pub mod note {
    pub const ID: &str = "_id";
    pub const GUID: &str = "guid";
    pub const MID: &str = "mid";
    pub const MOD: &str = "mod";
    pub const USN: &str = "usn";
    pub const TAGS: &str = "tags";
    pub const FLDS: &str = "flds";
    pub const SFLD: &str = "sfld";
    pub const CSUM: &str = "csum";
    pub const FLAGS: &str = "flags";
    pub const DATA: &str = "data";
}

pub mod card {
    pub const NOTE_ID: &str = "note_id";
    pub const ORD: &str = "ord";
    pub const CARD_NAME: &str = "card_name";
    pub const DECK_ID: &str = "deck_id";
    pub const QUESTION: &str = "question";
    pub const ANSWER: &str = "answer";
    // ... more constants
}

// Similar modules for deck, model, card_template, review_info, anki_media
```

## Built-in Model Templates

### BasicModel

```rust
pub struct BasicModel;

impl BasicModel {
    pub const FIELDS: &'static [&'static str] = &["Front", "Back"];
    pub const CARDS: &'static [&'static str] = &["Card 1"];
    pub const QUESTION_FORMAT: &'static str = "{{Front}}";
    pub const ANSWER_FORMAT: &'static str = "{{FrontSide}}<hr id=\"answer\">{{Back}}";
}
```

### Basic2Model

```rust
pub struct Basic2Model;

impl Basic2Model {
    pub const FIELDS: &'static [&'static str] = &["Front", "Back"];
    pub const CARDS: &'static [&'static str] = &["Card 1", "Card 2"];
    pub const QUESTION_FORMATS: &'static [&'static str] = &["{{Front}}", "{{Back}}"];
    pub const ANSWER_FORMATS: &'static [&'static str] = &[
        "{{FrontSide}}<hr id=\"answer\">{{Back}}",
        "{{FrontSide}}<hr id=\"answer\">{{Front}}"
    ];
}
```

## Version Information

```rust
/// API specification version this implementation supports
pub const API_HOST_SPEC_VERSION: i32 = 2;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Feature flags for conditional compilation
pub mod features {
    pub const HAS_JNI: bool = cfg!(target_os = "android");
    pub const HAS_API: bool = cfg!(target_os = "android");
    pub const HAS_SERDE: bool = true;
}
```

## Type Aliases

```rust
/// Result type for all API operations
pub type Result<T> = std::result::Result<T, AnkiDroidError>;
```

## Usage Examples

### Complete Note Addition Workflow

```rust
use ankidroid_api_rust::{AnkiDroidApi, Note, DEFAULT_DECK_ID, Result};

#[cfg(target_os = "android")]
fn add_geography_cards(vm: jni::JavaVM) -> Result<()> {
    let api = AnkiDroidApi::new(vm);
    
    // Ensure AnkiDroid is available
    if !api.is_available()? {
        return Err(AnkiDroidError::not_available("Please install AnkiDroid"));
    }
    
    // Get or create model
    let model_id = api.current_model_id()?;
    
    // Create notes
    let notes = vec![
        Note::builder(model_id)
            .field("Capital of France?".to_string())
            .field("Paris".to_string())
            .tag("geography".to_string())
            .build(),
        Note::builder(model_id)
            .field("Capital of Spain?".to_string())
            .field("Madrid".to_string())
            .tag("geography".to_string())
            .build(),
    ];
    
    // Add notes
    let count = api.add_notes(DEFAULT_DECK_ID, &notes)?;
    println!("Added {} notes", count);
    
    Ok(())
}
```

This specification covers the complete Rust API surface for interacting with AnkiDroid's content provider in a type-safe manner.