# PRP: AnkiDroid API Rust Package

---

## Goal

**Feature Goal**: Create a standalone Rust package `ankidroid-api-rust` that provides type-safe Rust bindings to the AnkiDroid API, exposing all functionality from the Kotlin/Java API with exact signature compatibility.

**Deliverable**: A reusable Rust crate that can be published to crates.io, providing full access to AnkiDroid's content provider API through idiomatic Rust interfaces, suitable for use in Android projects and the existing Tauri plugin.

**Success Definition**:

- All AnkiDroid API methods from `PRPs/research/AnkiDroidAPI/` are available in Rust with type-safe signatures
- The package compiles as a standalone crate for Android targets
- Integration tests demonstrate successful CRUD operations on AnkiDroid data
- The existing tauri-plugin-ankidroid can use this as a dependency

## Why

- **Separation of Concerns**: Extract AnkiDroid API logic from the Tauri plugin for reusability
- **Type Safety**: Provide compile-time guarantees for AnkiDroid API interactions
- **Broader Ecosystem**: Enable any Rust Android project to interact with AnkiDroid
- **Maintainability**: Single source of truth for AnkiDroid API bindings in Rust

## What

Create a Rust crate that mirrors the AnkiDroid API structure with:

- Direct mappings of all ContentProvider URIs and operations
- Type-safe representations of all AnkiDroid data models (Notes, Cards, Decks, Models)
- Helper functions matching the AddContentApi convenience methods
- Proper error handling for Android/JNI operations
- Comprehensive API documentation with examples

### Success Criteria

- [ ] All FlashCardsContract constants and URIs are accessible
- [ ] AddContentApi methods have Rust equivalents with identical functionality
- [ ] NoteInfo, BasicModel, Basic2Model, and Ease types are represented
- [ ] Utils functions (field/tag joining, checksum) are implemented
- [ ] Package builds for all Android targets (arm64-v8a, armeabi-v7a, x86, x86_64)
- [ ] Integration tests pass on Android emulator with AnkiDroid installed
- [ ] Documentation includes usage examples for common operations

## All Needed Context

### Context Completeness Check

_This PRP contains all necessary information for implementing AnkiDroid API bindings in Rust without prior knowledge of the codebase._

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API
  why: Official AnkiDroid API documentation with usage examples
  critical: Content provider URI structure and permission requirements

- url: https://mozilla.github.io/uniffi-rs/latest/
  why: UniFFI documentation for generating language bindings (alternative approach)
  critical: Not used directly but provides context for FFI best practices

- file: /workspaces/tauri-plugin-ankidroid/packages/tauri-plugin-ankidroid-android/src/android/jni_helpers.rs
  why: Existing JNI helper patterns for safe Android interop
  pattern: SafeJNIEnv wrapper, ContentValuesBuilder, error handling
  gotcha: Must handle JNI local references properly to avoid memory leaks

- file: /workspaces/tauri-plugin-ankidroid/packages/tauri-plugin-ankidroid-android/src/android/add_content_api.rs
  why: Current AddContentApi JNI wrapper implementation
  pattern: How to call Java methods from Rust, handle nullable returns
  gotcha: Check for AnkiDroid availability before API calls

- file: /workspaces/tauri-plugin-ankidroid/PRPs/research/AnkiDroidAPI/src/main/java/com/ichi2/anki/FlashCardsContract.kt
  why: Complete contract definition with all URIs and constants
  pattern: URI structure, column names, MIME types
  critical: Authority and permission constants must match exactly

- file: /workspaces/tauri-plugin-ankidroid/PRPs/research/AnkiDroidAPI/src/main/java/com/ichi2/anki/api/AddContentApi.kt
  why: High-level API methods that need Rust equivalents
  pattern: Method signatures, parameter validation, return types
  critical: API version checking logic, duplicate detection algorithm

- docfile: PRPs/ai_docs/ankidroid-api-specification.md
  why: Comprehensive API specification extracted from Kotlin files
  section: Complete API surface mapping
```

### Current Codebase Structure

```bash
/workspaces/tauri-plugin-ankidroid/
├── Cargo.toml                 # Workspace root
├── packages/
│   ├── tauri-plugin-ankidroid-android/  # Existing Tauri plugin
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── android/       # Android-specific code
│   │       │   ├── jni_helpers.rs
│   │       │   ├── add_content_api.rs
│   │       │   └── ...
│   │       └── lib.rs
│   └── tauri-plugin-ankidroid-js/      # JavaScript bindings
└── PRPs/
    └── research/
        └── AnkiDroidAPI/      # Kotlin API reference
```

### Desired Codebase Structure with New Package

```bash
/workspaces/tauri-plugin-ankidroid/
├── Cargo.toml                 # Workspace root (UPDATE: add new member)
├── packages/
│   ├── ankidroid-api-rust/    # NEW PACKAGE
│   │   ├── Cargo.toml         # Package manifest
│   │   ├── README.md          # Package documentation
│   │   ├── LICENSE-MIT        # Dual licensing
│   │   ├── LICENSE-APACHE     # Dual licensing
│   │   ├── API_SPEC.md        # Complete API specification
│   │   ├── build.rs           # Build script for Android targets
│   │   ├── src/
│   │   │   ├── lib.rs         # Main library entry point
│   │   │   ├── contract.rs    # FlashCardsContract constants/URIs
│   │   │   ├── models.rs      # Data models (Note, Card, Deck, Model)
│   │   │   ├── api.rs         # AddContentApi implementation
│   │   │   ├── utils.rs       # Utility functions (field/tag handling)
│   │   │   ├── error.rs       # Error types and handling
│   │   │   ├── jni/           # JNI-specific implementations
│   │   │   │   ├── mod.rs     # JNI module organization
│   │   │   │   ├── helpers.rs # JNI helper functions (from existing)
│   │   │   │   ├── content_resolver.rs # ContentResolver operations
│   │   │   │   └── cursor.rs  # Cursor handling
│   │   │   └── ffi/           # Alternative FFI approach (future)
│   │   │       └── mod.rs     # Placeholder for non-JNI approach
│   │   ├── tests/
│   │   │   ├── integration/   # Integration tests
│   │   │   └── unit/          # Unit tests
│   │   └── examples/
│   │       ├── add_note.rs    # Example: Adding a note
│   │       ├── query_deck.rs  # Example: Querying decks
│   │       └── sync_media.rs  # Example: Media sync
│   ├── tauri-plugin-ankidroid-android/  # MODIFY: Use ankidroid-api-rust
│   │   ├── Cargo.toml         # Add dependency on ankidroid-api-rust
│   │   └── src/
└── PRPs/
    └── ai_docs/
        └── ankidroid-api-specification.md  # NEW: Detailed API spec
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: JNI requires proper thread attachment
// Must use AttachGuard or attach_current_thread() before any JNI calls

// CRITICAL: AnkiDroid may not be installed
// Always check API availability with try_new() pattern

// CRITICAL: Content Provider permissions
// Requires com.ichi2.anki.permission.READ_WRITE_PERMISSION

// CRITICAL: Local reference management
// Use LocalFrame for operations creating many JNI objects

// CRITICAL: Field separator is 0x1F character
// Must use exact separator for field joining/splitting

// CRITICAL: Checksum calculation for duplicates
// Must strip HTML and calculate SHA1 exactly as AnkiDroid does
```

## Implementation Blueprint

### Core Data Models

```rust
// src/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub guid: String,
    pub mid: i64,  // Model ID
    pub mod_: i64,  // Modification time
    pub usn: i32,   // Update sequence number
    pub tags: Vec<String>,
    pub fields: Vec<String>,
    pub sfld: String,  // Sort field
    pub csum: i64,     // Checksum
    pub flags: i32,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub note_id: i64,
    pub ord: i32,
    pub name: String,
    pub deck_id: i64,
    pub question: String,
    pub answer: String,
    pub question_simple: String,
    pub answer_simple: String,
    pub answer_pure: String,
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    pub desc: String,
    pub counts: Vec<i32>,  // [learn, review, new]
    pub options: serde_json::Value,
    pub dyn_: bool,  // Is filtered deck
}

#[derive(Debug, Clone)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub field_names: Vec<String>,
    pub num_cards: i32,
    pub css: String,
    pub deck_id: Option<i64>,
    pub sort_field_index: i32,
    pub type_: i32,
    pub latex_post: String,
    pub latex_pre: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Ease {
    Ease1 = 1,
    Ease2 = 2,
    Ease3 = 3,
    Ease4 = 4,
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE packages/ankidroid-api-rust/Cargo.toml
  - Define package metadata and dependencies
  - Add jni, ndk, ndk-context, serde, thiserror dependencies
  - Configure Android-specific build targets
  - Set up feature flags for optional functionality

Task 2: CREATE src/error.rs
  - Define AnkiDroidError enum with all error variants
  - Implement From traits for JNI and Android errors
  - Add Result type alias for API operations
  - Include error context and recovery suggestions

Task 3: COPY AND ADAPT src/jni/helpers.rs from existing plugin
  - Copy SafeJNIEnv, AttachGuard, LocalFrame implementations
  - Extract ContentValuesBuilder and StringHelper
  - Remove Tauri-specific code, keep only core JNI helpers
  - Add comprehensive documentation

Task 4: CREATE src/contract.rs
  - Define all FlashCardsContract constants as pub const
  - Implement URI builder functions for each content type
  - Add AUTHORITY, READ_WRITE_PERMISSION constants
  - Include all column name constants for each table

Task 5: CREATE src/models.rs
  - Implement all data models (Note, Card, Deck, Model, etc.)
  - Add From<Cursor> trait implementations for each model
  - Implement ToContentValues trait for writable models
  - Add builder patterns for model creation

Task 6: CREATE src/utils.rs
  - Implement field joining/splitting (FIELD_SEPARATOR = \u001f)
  - Implement tag joining/splitting with space handling
  - Port fieldChecksum function with SHA1 and HTML stripping
  - Add HTML stripping utilities matching Kotlin implementation

Task 7: CREATE src/api.rs
  - Implement AddContentApi struct with new() constructor
  - Add all methods from Kotlin AddContentApi:
    - add_note, add_notes, update_note_tags, update_note_fields
    - find_duplicate_notes, get_note, get_note_count
    - add_new_basic_model, add_new_basic2_model, add_new_custom_model
    - get_model_list, get_field_list, get_current_model_id
    - add_new_deck, get_deck_list, get_selected_deck_name
    - add_media_from_uri, preview_new_note
    - get_api_host_spec_version
  - Implement proper error handling and validation

Task 8: CREATE src/jni/content_resolver.rs
  - Implement ContentResolver wrapper for Android operations
  - Add query, insert, update, delete methods
  - Handle Cursor iteration and data extraction
  - Implement proper cleanup and error handling

Task 9: CREATE src/jni/cursor.rs
  - Implement Cursor wrapper with iterator support
  - Add column index caching for performance
  - Implement type-safe column value extraction
  - Handle null values appropriately

Task 10: CREATE src/lib.rs
  - Set up module structure and re-exports
  - Add comprehensive crate-level documentation
  - Define public API surface
  - Add feature gates for optional functionality

Task 11: UPDATE workspace Cargo.toml
  - Add ankidroid-api-rust to workspace members
  - Configure shared dependencies

Task 12: CREATE build.rs
  - Add Android target detection
  - Configure linking for Android libraries
  - Set up cross-compilation helpers

Task 13: CREATE tests/integration/basic_operations.rs
  - Test note creation and retrieval
  - Test deck operations
  - Test model/note type operations
  - Verify error handling

Task 14: CREATE examples/add_note.rs
  - Demonstrate basic note addition workflow
  - Show error handling patterns
  - Include permission checking

Task 15: CREATE README.md
  - Add usage examples and quick start guide
  - Document Android setup requirements
  - Include troubleshooting section
  - Add contribution guidelines

Task 16: CREATE API_SPEC.md
  - Document complete API surface with signatures
  - Map Kotlin methods to Rust equivalents
  - Include type conversion tables
  - Add migration guide from direct JNI usage

Task 17: UPDATE tauri-plugin-ankidroid-android/Cargo.toml
  - Add dependency on ankidroid-api-rust
  - Remove redundant JNI code

Task 18: REFACTOR tauri-plugin-ankidroid-android to use new crate
  - Replace direct JNI calls with ankidroid-api-rust API
  - Update error handling to use new error types
  - Simplify command implementations
```

### Critical Implementation Details

```rust
// Example: Safe API initialization pattern
impl AnkiDroidApi {
    pub fn try_new(context: &JObject) -> Result<Self, AnkiDroidError> {
        // Check if AnkiDroid is installed
        if !Self::is_available(context)? {
            return Err(AnkiDroidError::NotInstalled);
        }

        // Check API version compatibility
        let version = Self::get_api_version(context)?;
        if version < MIN_SUPPORTED_VERSION {
            return Err(AnkiDroidError::IncompatibleVersion(version));
        }

        // Initialize with proper error handling
        Ok(Self {
            context: context.clone(),
            // ... other fields
        })
    }
}

// Example: Proper JNI error handling
fn query_notes(&self, model_id: i64) -> Result<Vec<Note>, AnkiDroidError> {
    let (guard, env) = AttachGuard::new()?;
    let frame = LocalFrame::new(&mut env, 512)?;

    let uri = build_note_uri()?;
    let cursor = self.content_resolver.query(
        &uri,
        None,  // projection
        Some(&format!("mid = {}", model_id)),
        None,  // selection args
        None,  // sort order
    )?;

    let mut notes = Vec::new();
    while cursor.move_next()? {
        notes.push(Note::from_cursor(&cursor)?);
    }

    Ok(notes)
}
```

## Validation Gates

### After Each Major Component

```bash
# After Task 6 (Utils implementation)
cargo test --package ankidroid-api-rust --lib utils

# After Task 9 (Core JNI components)
cargo build --package ankidroid-api-rust --target aarch64-linux-android

# After Task 13 (Integration tests)
cargo test --package ankidroid-api-rust --test integration -- --test-threads=1

# After Task 18 (Plugin refactor)
cd packages/tauri-plugin-ankidroid-e2e-test-app && npm run test:e2e
```

## Final Checklist

- [ ] All AnkiDroid API methods have Rust equivalents
- [ ] Package builds for all Android architectures
- [ ] Integration tests pass with real AnkiDroid instance
- [ ] Documentation includes working examples
- [ ] Error messages provide actionable recovery steps
- [ ] No memory leaks in JNI operations (validated with Valgrind)
- [ ] Tauri plugin successfully uses the new crate
- [ ] API_SPEC.md accurately reflects implementation
- [ ] Version detection works for AnkiDroid 2.5+
- [ ] Checksum calculation matches AnkiDroid exactly

## Implementation Order Summary

1. **Foundation** (Tasks 1-3): Package setup and core JNI helpers
2. **Data Layer** (Tasks 4-6): Models, contracts, and utilities
3. **API Layer** (Tasks 7-9): Main API and ContentResolver
4. **Integration** (Tasks 10-12): Library structure and build
5. **Testing** (Tasks 13-14): Validation and examples
6. **Documentation** (Tasks 15-16): User-facing docs
7. **Migration** (Tasks 17-18): Update existing plugin

## Risk Mitigation

- **Risk**: JNI complexity leads to memory leaks
  - **Mitigation**: Use RAII patterns, LocalFrame, extensive testing
- **Risk**: API changes in future AnkiDroid versions
  - **Mitigation**: Version detection, compatibility layer, clear deprecation

- **Risk**: Performance issues with bulk operations
  - **Mitigation**: Batch operations, prepared statements, async where possible

## Notes for Implementer

- Start with read-only operations before implementing writes
- Test each component in isolation before integration
- Use existing jni_helpers.rs as reference but adapt for standalone use
- Consider future non-JNI approach (Kotlin/Native) but don't over-engineer
- Prioritize safety over performance in initial implementation
- Document every public API method with examples

---

**Confidence Score**: 9/10 - This PRP provides comprehensive implementation guidance with concrete code examples, clear task ordering, and extensive context from existing code. The only uncertainty is around potential undocumented AnkiDroid API behaviors that may only surface during testing.
