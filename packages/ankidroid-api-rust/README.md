# AnkiDroid API Rust

[![Crates.io](https://img.shields.io/crates/v/ankidroid-api-rust.svg)](https://crates.io/crates/ankidroid-api-rust)
[![Documentation](https://docs.rs/ankidroid-api-rust/badge.svg)](https://docs.rs/ankidroid-api-rust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Type-safe Rust bindings for the AnkiDroid API FlashCardsContract. This crate provides comprehensive access to AnkiDroid's content provider API, allowing Android applications to interact with AnkiDroid's database and functionality in a type-safe manner.

## Features

- **Type Safety**: All API operations use strongly-typed Rust structs and enums
- **Error Handling**: Comprehensive error types with detailed error information  
- **Android Integration**: Full JNI support for Android applications
- **Content Provider Access**: Direct access to AnkiDroid's FlashCardsContract
- **Serialization Support**: Built-in serde support for all data types
- **High-Level API**: Convenient methods for common operations
- **Low-Level Access**: Direct ContentResolver access for advanced use cases

## Platform Requirements

- **Android**: API level 14+ (Android 4.0+)
- **AnkiDroid**: Version 2.5+ (API v1), Version 2.6+ recommended (API v2)
- **Permissions**: `com.ichi2.anki.permission.READ_WRITE_PERMISSION`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ankidroid-api-rust = "0.1.0"
```

For Android applications, ensure your `AndroidManifest.xml` includes the required permission:

```xml
<uses-permission android:name="com.ichi2.anki.permission.READ_WRITE_PERMISSION" />
```

## Quick Start

### Basic Note Addition

```rust
use ankidroid_api_rust::{AnkiDroidApi, Note, Result};

#[cfg(target_os = "android")]
fn add_flashcard(vm: jni::JavaVM) -> Result<()> {
    let api = AnkiDroidApi::new(vm);
    
    // Check if AnkiDroid is available
    if !api.is_available()? {
        return Err(AnkiDroidError::not_available("AnkiDroid not installed"));
    }
    
    // Get the current model ID
    let model_id = api.current_model_id()?;
    
    // Create a new note
    let note = Note::builder(model_id)
        .field("What is the capital of France?".to_string())
        .field("Paris".to_string())
        .tag("geography".to_string())
        .tag("capitals".to_string())
        .build();
    
    // Add the note to AnkiDroid
    let note_id = api.add_note(&note)?;
    println!("Added note with ID: {}", note_id);
    
    Ok(())
}
```

### Creating a Custom Model

```rust
use ankidroid_api_rust::{AnkiDroidApi, BasicModel, Result};

#[cfg(target_os = "android")]
fn create_custom_model(api: &AnkiDroidApi) -> Result<i64> {
    // Create a new basic model (front/back)
    let model_id = api.add_new_basic_model("My Custom Cards")?
        .ok_or_else(|| AnkiDroidError::operation_failed("Failed to create model"))?;
    
    println!("Created model with ID: {}", model_id);
    Ok(model_id)
}
```

### Bulk Adding Notes

```rust
use ankidroid_api_rust::{AnkiDroidApi, Note, Result};

#[cfg(target_os = "android")]
fn bulk_add_notes(api: &AnkiDroidApi, model_id: i64) -> Result<usize> {
    let notes = vec![
        Note::builder(model_id)
            .field("Question 1".to_string())
            .field("Answer 1".to_string())
            .build(),
        Note::builder(model_id)
            .field("Question 2".to_string())
            .field("Answer 2".to_string())
            .build(),
        Note::builder(model_id)
            .field("Question 3".to_string())
            .field("Answer 3".to_string())
            .build(),
    ];
    
    let added_count = api.add_notes(1, &notes)?; // 1 = default deck
    println!("Added {} notes successfully", added_count);
    
    Ok(added_count)
}
```

## API Overview

### High-Level API (`AnkiDroidApi`)

The high-level API provides convenient methods for common operations:

#### Note Operations
- `add_note(note: &Note) -> Result<i64>` - Add a single note
- `add_notes(deck_id: i64, notes: &[Note]) -> Result<usize>` - Bulk add notes
- `get_note(note_id: i64) -> Result<NoteInfo>` - Retrieve a note
- `update_note_fields(note_id: i64, fields: &[String]) -> Result<()>` - Update note fields
- `update_note_tags(note_id: i64, tags: &[String]) -> Result<()>` - Update note tags

#### Model Operations
- `add_new_basic_model(name: &str) -> Result<Option<i64>>` - Create basic model
- `add_new_basic2_model(name: &str) -> Result<Option<i64>>` - Create basic model with reverse
- `current_model_id() -> Result<i64>` - Get current model ID
- `get_model_list() -> Result<HashMap<i64, String>>` - List all models
- `get_field_list(model_id: i64) -> Result<Vec<String>>` - Get model fields

#### Deck Operations
- `add_new_deck(name: &str) -> Result<Option<i64>>` - Create a new deck
- `get_deck_list() -> Result<HashMap<i64, String>>` - List all decks
- `get_selected_deck_name() -> Result<String>` - Get current deck name

#### Media Operations
- `add_media_from_uri(file_uri: &str, preferred_name: &str, mime_type: &str) -> Result<String>` - Add media file

### Data Models

All data structures implement `Serialize` and `Deserialize` for easy JSON handling:

#### Note
```rust
pub struct Note {
    pub mid: i64,           // Model ID
    pub fields: Vec<String>, // Field values
    pub tags: Vec<String>,  // Tags
    pub guid: Option<String>, // GUID (auto-generated)
}
```

#### NoteInfo
```rust
pub struct NoteInfo {
    pub id: i64,
    pub guid: String,
    pub mid: i64,
    pub mod_time: i64,
    pub usn: i32,
    pub tags: Vec<String>,
    pub fields: Vec<String>,
    pub sort_field: String,
    pub checksum: i64,
    pub flags: i32,
    pub data: String,
}
```

#### Model
```rust
pub struct Model {
    pub id: i64,
    pub name: String,
    pub field_names: Vec<String>,
    pub num_cards: i32,
    pub css: String,
    pub deck_id: i64,
    pub sort_field_index: i32,
    pub model_type: i32,
    pub latex_pre: String,
    pub latex_post: String,
}
```

### Error Handling

All operations return `Result<T, AnkiDroidError>`. The error type provides detailed categorization:

```rust
use ankidroid_api_rust::{AnkiDroidError, ErrorCategory};

fn handle_error(result: Result<()>) {
    match result {
        Ok(()) => println!("Success!"),
        Err(err) => {
            println!("Error: {}", err);
            
            match err.category() {
                ErrorCategory::NotAvailable => println!("AnkiDroid not installed"),
                ErrorCategory::Permission => println!("Missing permissions"),
                ErrorCategory::Validation => println!("Invalid input data"),
                ErrorCategory::NotFound => println!("Resource not found"),
                ErrorCategory::Database => println!("Database error"),
                ErrorCategory::Operation => println!("Operation failed"),
                ErrorCategory::System => println!("System error"),
            }
            
            if err.is_recoverable() {
                println!("This error can be retried");
            }
        }
    }
}
```

### Utility Functions

The crate provides utility functions for field and tag manipulation:

```rust
use ankidroid_api_rust::{join_fields, split_fields, join_tags, split_tags};

// Field operations
let fields = vec!["Front".to_string(), "Back".to_string()];
let joined = join_fields(&fields); // "Front\x1FBack"
let split_back = split_fields(&joined); // ["Front", "Back"]

// Tag operations  
let tags = vec!["tag1".to_string(), "tag with space".to_string()];
let joined_tags = join_tags(&tags); // "tag1 tag_with_space"
let split_back_tags = split_tags(&joined_tags); // ["tag1", "tag_with_space"]
```

## Examples

See the `examples/` directory for complete working examples:

- `add_note.rs` - Complete note addition workflow
- `content_resolver_example.rs` - Low-level ContentResolver usage
- `utils_example.rs` - Utility function demonstrations

## Building for Android

This crate is designed for Android applications. When building for Android targets:

1. Set up the Android NDK and necessary build tools
2. Configure your project for cross-compilation
3. Ensure the JNI dependencies are properly linked

Example build command:
```bash
cargo build --target aarch64-linux-android
```

## Logging

The crate uses the `log` crate for logging. On Android, initialize the Android logger:

```rust
#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
    );
}
```

## Version Compatibility

- **API v1** (AnkiDroid 2.5): Basic functionality with slower bulk operations
- **API v2** (AnkiDroid 2.6+): Optimized bulk operations and improved model handling

The crate automatically detects the API version and uses appropriate methods.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is dual-licensed under either:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

This crate is based on the AnkiDroid API specification and provides Rust bindings for the FlashCardsContract. Special thanks to the AnkiDroid team for maintaining this powerful API.