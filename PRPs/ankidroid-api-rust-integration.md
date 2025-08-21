name: "ankidroid-api-rust Integration into tauri-plugin-ankidroid"
description: |
Replace the manual JNI implementation in tauri-plugin-ankidroid with the type-safe ankidroid-api-rust library,
significantly reducing complexity while maintaining full functionality for the e2e tests.

---

## Goal

**Feature Goal**: Replace the entire JNI layer in tauri-plugin-ankidroid-android with ankidroid-api-rust library, removing ~2000+ lines of complex JNI code while maintaining or improving functionality.

**Deliverable**: A simplified tauri-plugin-ankidroid that uses ankidroid-api-rust for all AnkiDroid operations, with passing e2e tests.

**Success Definition**: All e2e tests pass, especially the create-card-test.e2e.js, with significantly reduced code complexity and improved type safety.

## User Persona (if applicable)

**Target User**: Tauri plugin developers integrating AnkiDroid functionality

**Use Case**: Creating, reading, updating, and deleting Anki cards from a Tauri mobile application

**User Journey**: Developer imports tauri-plugin-ankidroid → Plugin initializes with Android context → Commands execute via ankidroid-api-rust → Results return to frontend

**Pain Points Addressed**:

- Complex JNI code maintenance
- Type safety issues with manual JNI
- Error handling complexity
- Thread attachment management

## Why

- Reduce codebase complexity by ~70% (removing 2000+ lines of JNI code)
- Improve type safety and eliminate entire classes of runtime errors
- Leverage battle-tested ankidroid-api-rust library instead of maintaining custom JNI
- Better error handling with structured error types
- Easier maintenance and future feature additions

## What

Replace the custom JNI implementation in tauri-plugin-ankidroid-android with ankidroid-api-rust while maintaining all current functionality:

- Note creation with deck and tag support (Notes generate Cards)
- Note listing and querying (frontend may call them "cards" but we're working with Notes)
- Deck retrieval with proper names (not "Deck {id}")
- Note updates and deletion
- Error handling with graceful fallbacks

### AnkiDroid Terminology Clarification

**CRITICAL**: Understand the Anki data model to avoid confusion:
- **Note**: The actual data entity containing fields (e.g., front/back text). This is what we create/update/delete.
- **Card**: Generated from a Note based on templates. One Note can generate multiple Cards.
- **Model** (Note Type): Defines the fields and card templates for Notes (e.g., "Basic" has Front/Back fields)
- **Deck**: Container where Cards are placed for study
- **Field**: Individual piece of data in a Note (e.g., "Front", "Back", "Extra")

**IMPORTANT**: Our current implementation incorrectly uses "card" terminology for what are actually "notes":
- `create_card` → Should be `create_note` (creates a Note which generates Cards)
- `list_cards` → Should be `list_notes` (lists Notes, not Cards)
- `update_card` → Should be `update_note` (updates a Note's fields)
- `delete_card` → Should be `delete_note` (deletes a Note and its generated Cards)

We MUST follow the AnkiDroid API standard nomenclature. Update all function names, variable names, and documentation to use correct terminology (note/card/deck/model). The frontend and tests will be updated to match.

### Success Criteria

- [ ] E2E test `test:e2e` passes (card creation workflow)
- [ ] E2E test `test:e2e:webview` passes (deck loading)
- [ ] Error handling test passes (resilience)
- [ ] Code reduction of at least 50% in android module
- [ ] Deck names resolve properly (not showing "Deck {id}")

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API#using-the-api
  why: Official AnkiDroid API documentation for understanding the underlying ContentProvider
  critical: Permission requirements and Android 11+ manifest changes

- file: packages/ankidroid-api-rust/src/api.rs
  why: Main API interface of ankidroid-api-rust - shows all available methods
  pattern: AnkiDroidApi struct methods and initialization pattern
  gotcha: API requires JNIEnv and Android Context, not thread-safe

- file: packages/ankidroid-api-rust/src/models.rs
  why: Data structures for Note, Card, Deck - needed for type conversions
  pattern: Builder pattern for Note creation
  gotcha: Fields are Vec<String>, not individual parameters

- file: packages/tauri-plugin-ankidroid-android/src/mobile.rs
  why: Current integration layer - shows how to get Android context in Tauri
  pattern: Thread attachment, context retrieval, error handling patterns
  gotcha: Returns dummy data on errors for graceful degradation

- file: packages/tauri-plugin-ankidroid-android/src/commands.rs
  why: All Tauri commands that need reimplementation
  pattern: Command signatures and response types that must be preserved
  gotcha: JSON serialization format must remain compatible

- file: packages/tauri-plugin-ankidroid-android/src/android/cards.rs
  why: Current card operations to be replaced
  pattern: Field parsing logic and deck/model handling
  gotcha: Uses field separator "|" for multiple fields

- file: packages/tauri-plugin-ankidroid-e2e-test-app/tests/create-card-test.e2e.js
  why: Main e2e test that must pass - defines expected behavior
  pattern: Test flow and expected responses
  gotcha: Expects specific JSON response format

- docfile: PRPs/ai_docs/ankidroid-api-specification.md
  why: Comprehensive API specification for ankidroid-api-rust
  section: API Methods and Error Handling
```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase

```bash
/workspaces/tauri-plugin-ankidroid
├── packages
│   ├── ankidroid-api-rust           # Library to integrate
│   │   └── src
│   │       ├── api.rs               # Main API interface
│   │       ├── models.rs            # Data structures
│   │       ├── error.rs             # Error types
│   │       └── jni/                 # JNI helpers
│   ├── tauri-plugin-ankidroid-android  # Plugin to modify
│   │   └── src
│   │       ├── lib.rs               # Plugin entry point
│   │       ├── commands.rs          # Tauri commands
│   │       ├── mobile.rs            # Android integration
│   │       └── android/             # TO BE REMOVED/REPLACED
│   │           ├── cards.rs
│   │           ├── decks.rs
│   │           ├── jni_helpers.rs
│   │           ├── content_provider.rs
│   │           └── cursor.rs
│   └── tauri-plugin-ankidroid-e2e-test-app
│       └── tests/                   # Must pass after integration
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
/workspaces/tauri-plugin-ankidroid
├── packages
│   ├── ankidroid-api-rust           # External dependency
│   ├── tauri-plugin-ankidroid-android
│   │   └── src
│   │       ├── lib.rs               # Plugin entry point (minimal changes)
│   │       ├── commands.rs          # Updated to use ankidroid-api-rust
│   │       ├── mobile.rs            # Simplified integration layer
│   │       └── android/             # MOSTLY REMOVED
│   │           ├── api_wrapper.rs  # NEW: Thin wrapper around ankidroid-api-rust
│   │           └── error.rs        # Keep for error conversion
│   └── tauri-plugin-ankidroid-e2e-test-app
│       └── tests/                   # All tests passing
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: Android context retrieval in Tauri
// The ndk_context crate provides the Android context as a raw pointer
let ctx = ndk_context::android_context();
let context = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

// CRITICAL: Thread attachment is required for JNI
// Every thread that uses JNI must be attached
let vm = unsafe { JavaVM::from_raw(ctx.vm() as _) };
let env = vm.attach_current_thread().map_err(|e| e.to_string())?;

// CRITICAL: ankidroid-api-rust is NOT thread-safe
// Each thread needs its own API instance
let mut api = AnkiDroidApi::try_new(env, &context)?;

// CRITICAL: Response format must match current implementation
// Frontend expects specific JSON structure
CreateCardResponse {
    success: bool,
    note_id: Option<i64>,
    message: Option<String>,
    error: Option<String>
}

// CRITICAL: Deck names must be resolved
// Current bug shows "Deck {id}" instead of actual names
// ankidroid-api-rust's get_deck_list() returns HashMap<i64, String> with proper names

// IMPORTANT: ankidroid-api-rust is within our codebase and can be extended
// Strategy for extending ankidroid-api-rust:
// 1. Core API methods (matching AnkiDroid spec) stay in api.rs unchanged
// 2. Extended operations go in a NEW module: ankidroid-api-rust/src/extended.rs
// 3. Document extended methods clearly with /// EXTENDED API: prefix
// 4. Keep tauri plugin minimal - prefer adding to extended.rs over api_wrapper.rs
// Example: list_cards(), update_card(), delete_card() should go in extended.rs
```

## Implementation Blueprint

### Data models and structure

```rust
// Use correct AnkiDroid API terminology throughout
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteResponse {  // Correct name - we create Notes
    pub success: bool,
    pub note_id: Option<i64>,  // ID of the created Note
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNoteResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
}

// NoteInfo for frontend display (simplified Note representation)
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteInfo {
    pub id: i64,        // note_id
    pub front: String,  // First field of the Note
    pub back: String,   // Second field of the Note  
    pub deck: String,   // Deck name where Cards are placed
    pub tags: String,   // Tags associated with the Note
}

// Use ankidroid-api-rust models internally
use ankidroid_api_rust::{Note, Card, Model, AnkiDroidApi, AnkiDroidError};
// These correctly distinguish Note vs Card vs Model vs Deck
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: UPDATE packages/tauri-plugin-ankidroid-android/Cargo.toml
  - ADD: ankidroid-api-rust dependency with path = "../ankidroid-api-rust"
  - REMOVE: Direct jni, ndk, ndk-context dependencies (now via ankidroid-api-rust)
  - KEEP: serde, serde_json, thiserror, log, tauri dependencies
  - VERIFY: Workspace inheritance still works

Task 2: CREATE packages/tauri-plugin-ankidroid-android/src/android/api_wrapper.rs
  - IMPLEMENT: get_api_instance() -> Result<(JNIEnv, JObject, AnkiDroidApi)>
  - FOLLOW pattern: packages/tauri-plugin-ankidroid-android/src/mobile.rs (context retrieval)
  - HANDLE: Thread attachment and context conversion
  - RETURN: Tuple of env, context, and initialized API
  - PLACEMENT: New file in android module

Task 3: MODIFY packages/tauri-plugin-ankidroid-android/src/android/error.rs
  - IMPLEMENT: From<AnkiDroidError> for AndroidError conversion
  - MAP: AnkiDroidError variants to existing AndroidError types
  - PRESERVE: Existing error handling patterns for compatibility
  - ADD: Helper methods for error message formatting

Task 4: SIMPLIFY packages/tauri-plugin-ankidroid-android/src/mobile.rs
  - REPLACE: Direct android module calls with api_wrapper calls
  - RENAME: create_card → create_note (uses ankidroid-api-rust add_note)
  - RENAME: list_cards → list_notes (uses extended API list_notes)
  - KEEP: get_decks using api.get_deck_list()
  - RENAME: update_card → update_note (uses extended API update_note)
  - RENAME: delete_card → delete_note (uses extended API delete_note)
  - PRESERVE: Error handling with dummy data fallback pattern
  - UPDATE: All internal variables to use correct terminology

Task 5: UPDATE packages/tauri-plugin-ankidroid-android/src/commands.rs
  - RENAME: All command functions to use correct terminology
  - UPDATE: create_card → create_note
  - UPDATE: list_cards → list_notes  
  - UPDATE: update_card → update_note
  - UPDATE: delete_card → delete_note
  - UPDATE: Response types to match new names (CreateNoteResponse, etc.)
  - TEST: Each command with manual testing after implementation

Task 6: CREATE packages/ankidroid-api-rust/src/extended.rs for extended operations
  - CREATE: New module file extended.rs with clear separation from core API
  - IMPLEMENT: Extended trait for AnkiDroidApi with additional methods
  - ADD: list_notes() method using ContentResolver (NOT list_cards - we're listing Notes!)
  - ADD: update_note() method using ContentResolver (updates Note fields)
  - ADD: delete_note() method using ContentResolver (deletes a Note and its Cards)
  - DOCUMENT: Each method with /// EXTENDED API: prefix
  - DOCUMENT: Clarify Note vs Card distinction in method docs
  - FOLLOW pattern: ankidroid-api-rust/src/api.rs for structure
  - USE: Internal ContentResolver from jni module
  - EXPORT: Add pub mod extended; to lib.rs
  - TEST: Add tests in ankidroid-api-rust/tests/extended_test.rs

Task 7: UPDATE Frontend and Tests for correct terminology
  - UPDATE: packages/tauri-plugin-ankidroid-js/src/index.ts
  - RENAME: createCard → createNote, listCards → listNotes, etc.
  - UPDATE: packages/tauri-plugin-ankidroid-e2e-test-app/src/App.tsx
  - UPDATE: Button labels and function calls to use "Note" terminology
  - UPDATE: packages/tauri-plugin-ankidroid-e2e-test-app/tests/*.e2e.js
  - UPDATE: Test descriptions and expectations to use correct terminology

Task 8: REMOVE obsolete android module files
  - DELETE: android/cards.rs (replaced by api_wrapper)
  - DELETE: android/decks.rs (replaced by api calls)
  - DELETE: android/models.rs (replaced by api calls)
  - DELETE: android/jni_helpers.rs (using ankidroid-api-rust)
  - DELETE: android/content_provider.rs (using ankidroid-api-rust)
  - DELETE: android/cursor.rs (using ankidroid-api-rust)
  - DELETE: android/constants.rs (using ankidroid-api-rust)
  - KEEP: android/error.rs (for error conversion)
  - KEEP: android/mod.rs (update exports)

Task 9: BUILD and fix compilation errors
  - RUN: cd packages/tauri-plugin-ankidroid-android && cargo build
  - FIX: Any import errors or type mismatches
  - ENSURE: All cfg(target_os = "android") conditionals correct
  - VERIFY: No unused dependencies warnings

Task 10: TEST with e2e test app
  - BUILD: npm run android:build
  - DEPLOY: npm run android:deploy
  - RUN: npm run test:e2e
  - VERIFY: Note creation works
  - VERIFY: Deck names show correctly (not "Deck 1")
  - CHECK: Error handling remains graceful
  - VERIFY: All operations use correct Note/Card terminology
```

### Implementation Patterns & Key Details

```rust
// packages/ankidroid-api-rust/src/extended.rs - NEW FILE for extended operations
/// EXTENDED API: Additional operations not in the core AnkiDroid spec
/// These are logical operations commonly needed by applications
/// 
/// IMPORTANT: Note vs Card Terminology
/// - Note: The data entity with fields (what we create/update/delete)
/// - Card: Generated from Notes via templates (for study)
/// - We work with Notes, even though frontend may call them "cards"
use crate::{AnkiDroidApi, AnkiDroidError, Note};
use crate::jni::content_resolver::ContentResolver;
use crate::contract;

pub trait AnkiDroidApiExtended<'local> {
    /// EXTENDED API: List all notes from AnkiDroid
    /// Returns a vector of Notes with their fields and metadata
    /// Note: Despite what frontend calls this, we're listing NOTES not CARDS
    fn list_notes(&self) -> Result<Vec<Note>, AnkiDroidError>;
    
    /// EXTENDED API: Update an existing note's fields
    /// Updates the fields of a Note by its ID
    /// @param note_id - The ID of the Note to update
    /// @param fields - The new field values for the Note
    fn update_note(&mut self, note_id: i64, fields: &[&str]) -> Result<(), AnkiDroidError>;
    
    /// EXTENDED API: Delete a note by ID
    /// Removes a Note and all Cards generated from it
    /// @param note_id - The ID of the Note to delete
    fn delete_note(&mut self, note_id: i64) -> Result<bool, AnkiDroidError>;
}

impl<'local> AnkiDroidApiExtended<'local> for AnkiDroidApi<'local> {
    fn list_notes(&self) -> Result<Vec<Note>, AnkiDroidError> {
        // Implementation using self.resolver (ContentResolver)
        // Query the NOTES table, not cards!
        let uri = contract::notes::ALL_URI;
        let projection = contract::notes::DEFAULT_PROJECTION;
        // Parse results into Note structs
        // ... implementation details
    }
    
    fn update_note(&mut self, note_id: i64, fields: &[&str]) -> Result<(), AnkiDroidError> {
        // Update a Note's fields (not a Card!)
        let uri = format!("{}/{}", contract::notes::CONTENT_URI, note_id);
        // ... implementation details
    }
    
    fn delete_note(&mut self, note_id: i64) -> Result<bool, AnkiDroidError> {
        // Delete a Note (this will delete all Cards generated from it)
        let uri = format!("{}/{}", contract::notes::CONTENT_URI, note_id);
        // ... implementation details
    }
}

// packages/tauri-plugin-ankidroid-android/src/android/api_wrapper.rs - Minimal wrapper
use ankidroid_api_rust::{AnkiDroidApi, AnkiDroidError};
use ankidroid_api_rust::extended::AnkiDroidApiExtended; // Import extended trait
use jni::{JNIEnv, objects::JObject, JavaVM};

pub fn get_api_instance<'a>() -> Result<(JNIEnv<'a>, JObject<'a>, AnkiDroidApi<'a>), String> {
    // Get Android context from ndk_context
    let ctx = ndk_context::android_context();
    let context = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    // Attach current thread to JVM
    let vm = unsafe { JavaVM::from_raw(ctx.vm() as _) };
    let env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;

    // Initialize API - now with extended methods available via trait
    let api = AnkiDroidApi::try_new(env, &context)
        .map_err(|e| format!("Failed to initialize AnkiDroid API: {}", e))?;

    Ok((env, context, api))
}

// mobile.rs - Create note using correct terminology
pub fn create_note(front: String, back: String, deck: Option<String>, tags: Option<String>) -> Result<String, String> {
    let (env, context, mut api) = api_wrapper::get_api_instance()?;

    // Get or create model (use default Basic model)
    let model_id = api.add_new_basic_model("Basic")
        .map_err(|e| e.to_string())?
        .unwrap_or(/* fallback model id */ 1);

    // Get or create deck
    let deck_id = if let Some(deck_name) = deck {
        api.add_new_deck(&deck_name)
            .map_err(|e| e.to_string())?
            .unwrap_or(1) // Default deck
    } else {
        1 // Default deck ID
    };

    // Prepare tags
    let tag_vec: Option<Vec<&str>> = tags.as_ref().map(|t| vec![t.as_str()]);

    // Add note using ankidroid-api-rust (this creates a Note, which generates Cards)
    let note_id = api.add_note(
        model_id,
        deck_id,
        &[&front, &back],
        tag_vec.as_deref()
    ).map_err(|e| e.to_string())?;

    // Return response with correct terminology
    let response = CreateNoteResponse {
        success: true,
        note_id,
        message: Some(format!("Note created successfully")),
        error: None,
    };

    Ok(serde_json::to_string(&response).unwrap())
}

// mobile.rs - Fixed get_decks implementation
pub fn get_decks() -> Result<String, String> {
    let (env, context, api) = api_wrapper::get_api_instance()?;

    // Get deck list returns HashMap<deck_id, deck_name>
    let deck_map = api.get_deck_list().map_err(|e| e.to_string())?;

    // Convert to expected format
    let decks: Vec<Deck> = deck_map.into_iter()
        .map(|(id, name)| Deck { id, name })
        .collect();

    Ok(serde_json::to_string(&decks).unwrap())
}

// mobile.rs - List notes using extended API with correct terminology
pub fn list_notes() -> Result<String, String> {
    let (env, context, api) = api_wrapper::get_api_instance()?;
    
    // Use the extended API method from ankidroid-api-rust
    use ankidroid_api_rust::extended::AnkiDroidApiExtended;
    let notes = api.list_notes().map_err(|e| e.to_string())?;
    
    // Convert to NoteInfo format for frontend
    let note_infos: Vec<NoteInfo> = notes.into_iter()
        .map(|note| NoteInfo::from_note(note))
        .collect();
    
    Ok(serde_json::to_string(&note_infos).unwrap())
}

// mobile.rs - Update note using extended API with correct terminology
pub fn update_note(note_id: i64, front: String, back: String) -> Result<String, String> {
    let (env, context, mut api) = api_wrapper::get_api_instance()?;
    
    use ankidroid_api_rust::extended::AnkiDroidApiExtended;
    api.update_note(note_id, &[&front, &back]).map_err(|e| e.to_string())?;
    
    let response = UpdateNoteResponse {
        success: true,
        message: Some("Note updated successfully".to_string()),
    };
    
    Ok(serde_json::to_string(&response).unwrap())
}

// mobile.rs - Delete note using extended API with correct terminology
pub fn delete_note(note_id: i64) -> Result<String, String> {
    let (env, context, mut api) = api_wrapper::get_api_instance()?;
    
    use ankidroid_api_rust::extended::AnkiDroidApiExtended;
    let deleted = api.delete_note(note_id).map_err(|e| e.to_string())?;
    
    Ok(json!({"success": deleted, "message": "Note deleted successfully"}).to_string())
}
```

### Integration Points

```yaml
CARGO:
  - add to: packages/tauri-plugin-ankidroid-android/Cargo.toml
  - dependency: ankidroid-api-rust = { path = "../ankidroid-api-rust" }

ANDROID_MODULE:
  - simplify: src/android/mod.rs
  - exports: "pub mod api_wrapper; pub mod error;"

MOBILE_MODULE:
  - update: All function implementations to use api_wrapper
  - preserve: Error handling patterns and response formats

BUILD_SCRIPTS:
  - verify: cargo ndk build still works
  - ensure: All Android architectures supported
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Check Rust compilation
cd packages/tauri-plugin-ankidroid-android
cargo check --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Build the plugin
cd packages/tauri-plugin-ankidroid-android
cargo build --release

# Test ankidroid-api-rust separately
cd ../ankidroid-api-rust
cargo test --all-features

# Expected: All tests pass. Debug any failures before proceeding.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build and deploy the test app
cd packages/tauri-plugin-ankidroid-e2e-test-app
npm run android:build
npm run android:deploy

# Grant permissions
npm run android:grant-permissions

# Run e2e tests
npm run test:e2e
npm run test:e2e:webview

# Manual verification
npm run android:dev
# Test in app: Create card, load decks, verify deck names show correctly

# Expected: All tests pass, deck names display properly
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Performance comparison
# Time the card creation before and after integration
time npm run test:e2e

# Memory usage check
adb shell dumpsys meminfo com.tauri.ankidroid.demo

# Check for memory leaks
adb shell am dumpheap com.tauri.ankidroid.demo /data/local/tmp/heap.prof
adb pull /data/local/tmp/heap.prof
# Analyze heap dump for JNI reference leaks

# Stress test - rapid operations
node -e "
const test = require('./test-manual.cjs');
for(let i = 0; i < 100; i++) {
  test.createCard(\`Test \${i}\`, \`Answer \${i}\`);
}
"

# Expected: No crashes, consistent performance, no memory leaks
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] Rust compilation: `cargo build --release`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`

### Feature Validation

- [ ] E2E test passes: `npm run test:e2e`
- [ ] WebView test passes: `npm run test:e2e:webview`
- [ ] Deck names display correctly (not "Deck 1")
- [ ] Card creation/update/delete works
- [ ] Error handling remains graceful
- [ ] No crashes or ANRs

### Code Quality Validation

- [ ] Removed at least 1500 lines of JNI code
- [ ] All responses maintain JSON compatibility
- [ ] Error messages are helpful and specific
- [ ] No memory leaks detected
- [ ] Performance equal or better than before

### Documentation & Deployment

- [ ] Update README with new architecture
- [ ] Document any new build requirements
- [ ] Update package.json version
- [ ] Ensure CI/CD still passes

---

## Anti-Patterns to Avoid

- ❌ Don't try to make ankidroid-api-rust thread-safe - create new instances per thread
- ❌ Don't change the JSON response format - frontend depends on it
- ❌ Don't remove error fallbacks - graceful degradation is important
- ❌ Don't forget to handle null/empty responses from AnkiDroid
- ❌ Don't skip the e2e tests - they catch integration issues
- ❌ Don't ignore JNI reference leaks - they cause crashes over time
