name: "Tauri AnkiDroid Plugin Refactor - JNI Architecture with Modular Rust Implementation"
description: |

---

## Goal

**Feature Goal**: Refactor the Tauri AnkiDroid plugin to embrace pure Rust JNI implementation with modular, maintainable architecture while removing all desktop support and Java/Kotlin files

**Deliverable**: A mobile-only Tauri plugin with clean Rust JNI abstractions, modular file organization, and complete Card CRUD operations via AnkiDroid ContentProvider

**Success Definition**: The plugin compiles for Android only, passes all unit tests, successfully creates/reads/updates/deletes cards in AnkiDroid, and maintains file sizes under 300 lines per module

## Why

- Current implementation has 600+ line monolithic functions with deeply nested cursor operations
- Repetitive exception handling without reusable abstractions causes maintenance burden
- Manual resource management risks memory leaks without RAII patterns
- Desktop mock code adds unnecessary complexity for a mobile-only plugin
- Java/Kotlin files are redundant when Rust can handle JNI directly

## What

Transform the existing Tauri AnkiDroid plugin into a well-structured, maintainable Rust-only implementation that:
- Uses modular file organization with feature-based separation (cards.rs, decks.rs, models.rs)
- Implements RAII patterns for automatic JNI resource cleanup
- Provides reusable JNI helper abstractions for exception handling and cursor iteration
- Removes all desktop support and Java/Kotlin files
- Focuses initially on Card CRUD operations with proper validation

### Success Criteria

- [ ] All desktop code and Java/Kotlin files removed
- [ ] Card CRUD operations working via ContentProvider
- [ ] Each Rust module under 300 lines
- [ ] All JNI resources managed with RAII patterns
- [ ] Unit tests passing for each module
- [ ] Plugin builds successfully for Android target only

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/jni/0.21/jni/
  why: Core JNI crate documentation for Rust - essential for all JNI operations
  critical: Lifetime management with 'local, proper error handling with Result types

- url: https://v2.tauri.app/develop/plugins/develop-mobile/
  why: Official Tauri mobile plugin development guide
  critical: Plugin structure, mobile.rs patterns, Android-specific setup

- url: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/geolocation
  why: Reference implementation of a well-structured Tauri mobile plugin
  critical: Shows channel-based communication, permission handling patterns

- file: packages/tauri-plugin-ankidroid-android/src/mobile.rs
  why: Current implementation to refactor - shows existing JNI patterns and issues
  pattern: Long cursor iteration blocks, manual exception handling
  gotcha: 600+ line functions, deeply nested error handling

- file: PRPs/planning/API.md
  why: Proposed JavaScript API interface for the plugin
  pattern: TypeScript interface definitions, method signatures
  critical: Must match these signatures in final implementation

- file: PRPs/planning/REFERENCE_IMPLEMENTATION.md
  why: Java reference showing AnkiDroid ContentProvider integration
  pattern: suspendCard, changeDeck, addMedia methods
  critical: Shows correct ContentProvider URIs and ContentValues usage

- docfile: PRPs/ai_docs/ankidroid_api_reference.md
  why: Comprehensive AnkiDroid ContentProvider documentation
  section: FlashCardsContract constants, URI patterns, field formats
  critical: Fields must be separated by \u001f, proper URI construction

- docfile: PRPs/ai_docs/tauri_plugin_patterns.md
  why: Best practices for Tauri plugin JNI implementation
  section: RAII patterns, exception handling, resource management
```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase

```bash
.
├── packages/
│   ├── tauri-plugin-ankidroid-android/
│   │   ├── android/
│   │   │   └── src/main/java/  # TO BE DELETED
│   │   ├── src/
│   │   │   ├── lib.rs          # Plugin entry point
│   │   │   ├── commands.rs     # Tauri command handlers
│   │   │   ├── desktop.rs      # TO BE DELETED
│   │   │   └── mobile.rs       # Current monolithic implementation
│   │   └── Cargo.toml
│   ├── tauri-plugin-ankidroid-js/
│   │   └── src/
│   │       └── index.ts        # TypeScript API
│   └── tauri-plugin-ankidroid-e2e-test-app/
│       └── src-tauri/          # Test app
└── PRPs/
    ├── planning/
    │   ├── API.md              # JS API specification
    │   └── REFERENCE_IMPLEMENTATION.md  # Java examples
    └── ai_docs/
        └── ankidroid_api_reference.md  # ContentProvider docs
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
packages/tauri-plugin-ankidroid-android/src/
├── lib.rs              # Main plugin entry point (Android-only)
├── commands.rs         # Tauri command definitions
├── android/            # All Android implementations
│   ├── mod.rs          # Module exports and main entry
│   ├── constants.rs    # ContentProvider URIs and column names
│   ├── error.rs        # Custom error types with thiserror
│   ├── jni_helpers.rs  # RAII wrappers, exception handling, string conversion
│   ├── cursor.rs       # CursorIterator with automatic cleanup
│   ├── content_provider.rs # ContentProvider query builder abstraction
│   ├── models.rs       # Model operations (find Basic model)
│   ├── decks.rs        # Deck operations (get/create deck)
│   └── cards.rs        # Card CRUD operations
└── types.rs            # Shared type definitions (Card, Deck, Model structs)

# DELETED FILES:
- src/desktop.rs        # No desktop support
- android/src/main/java/  # All Java/Kotlin files removed
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: AnkiDroid ContentProvider field separator
// Fields MUST be separated by ASCII unit separator (\u{001f})
let fields = format!("{}\u{001f}{}", front, back);

// CRITICAL: JNI local reference limit
// Android has a default limit of 512 local references
// Use push_local_frame/pop_local_frame for operations creating many objects

// CRITICAL: Exception checking after EVERY JNI call
// Java exceptions don't automatically propagate to Rust
if env.exception_check()? {
    env.exception_describe()?;
    env.exception_clear()?;
    return Err(JniError::JavaException);
}

// CRITICAL: ContentProvider URIs vary by AnkiDroid version
// Use "com.ichi2.anki.flashcards" for newer versions
// Fallback to "com.ichi2.anki.provider" for older versions

// CRITICAL: Cursor must be closed to avoid memory leaks
// Always implement Drop trait for cursor wrappers

// CRITICAL: Model finding by name is unreliable
// Users can rename models - search for "Basic" but also check field count
```

## Implementation Blueprint

### Data models and structure

Create the core data models for type safety and JSON serialization.

```rust
// types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub deck: String,
    pub tags: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deck_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub field_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: DELETE packages/tauri-plugin-ankidroid-android/src/desktop.rs
  - REMOVE: All desktop mock implementations
  - VERIFY: No references remain in lib.rs or Cargo.toml
  - UPDATE: lib.rs to remove #[cfg(desktop)] blocks

Task 2: DELETE packages/tauri-plugin-ankidroid-android/android/src/main/java/
  - REMOVE: Entire java directory and all .java/.kt files
  - UPDATE: build.rs if it references Java compilation
  - VERIFY: Cargo.toml doesn't reference Java build steps

Task 3: CREATE packages/tauri-plugin-ankidroid-android/src/android/mod.rs
  - IMPLEMENT: Module exports for all android submodules
  - FOLLOW pattern: Standard Rust module organization
  - EXPORTS: pub use statements for public APIs from submodules

Task 4: CREATE packages/tauri-plugin-ankidroid-android/src/android/constants.rs
  - IMPLEMENT: All ContentProvider URIs and column name constants
  - FOLLOW pattern: PRPs/ai_docs/ankidroid_api_reference.md FlashCardsContract section
  - NAMING: UPPER_SNAKE_CASE for constants
  - INCLUDE: Both URI authorities for version compatibility

Task 5: CREATE packages/tauri-plugin-ankidroid-android/src/android/error.rs
  - IMPLEMENT: Custom error types using thiserror
  - FOLLOW pattern: Tauri plugin error handling conventions
  - TYPES: JniError, ContentProviderError, ValidationError
  - INCLUDE: From implementations for error conversion

Task 6: CREATE packages/tauri-plugin-ankidroid-android/src/android/jni_helpers.rs
  - IMPLEMENT: SafeJNIEnv wrapper with automatic exception checking
  - IMPLEMENT: StringHelper for Rust<->Java string conversion
  - IMPLEMENT: LocalFrame RAII wrapper for local reference management
  - FOLLOW pattern: Research from Rust JNI helper patterns
  - CRITICAL: All wrappers must implement Drop trait

Task 7: CREATE packages/tauri-plugin-ankidroid-android/src/android/cursor.rs
  - IMPLEMENT: CursorIterator with automatic close on Drop
  - FOLLOW pattern: Iterator trait implementation
  - METHODS: get_string, get_long, advance
  - CRITICAL: Must call cursor.close() in Drop implementation

Task 8: CREATE packages/tauri-plugin-ankidroid-android/src/android/content_provider.rs
  - IMPLEMENT: ContentProviderQuery builder pattern
  - IMPLEMENT: get_content_resolver helper function
  - FOLLOW pattern: Builder pattern with execute() method
  - DEPENDENCIES: Uses cursor.rs CursorIterator

Task 9: CREATE packages/tauri-plugin-ankidroid-android/src/android/models.rs
  - IMPLEMENT: find_basic_model_id function
  - FOLLOW pattern: PRPs/planning/REFERENCE_IMPLEMENTATION.md findModelIdByName
  - VALIDATION: Check both name and field count
  - DEPENDENCIES: Uses content_provider.rs and cursor.rs

Task 10: CREATE packages/tauri-plugin-ankidroid-android/src/android/decks.rs
  - IMPLEMENT: find_deck_id_by_name function
  - IMPLEMENT: create_deck_if_not_exists function
  - FOLLOW pattern: PRPs/planning/REFERENCE_IMPLEMENTATION.md findDeckIdByName
  - DEPENDENCIES: Uses content_provider.rs

Task 11: CREATE packages/tauri-plugin-ankidroid-android/src/android/cards.rs
  - IMPLEMENT: create_card, list_cards, update_card, delete_card functions
  - FOLLOW pattern: Current mobile.rs but using new abstractions
  - DEPENDENCIES: Uses all helper modules
  - CRITICAL: Fields must use \u{001f} separator

Task 12: MODIFY packages/tauri-plugin-ankidroid-android/src/mobile.rs
  - REFACTOR: Replace monolithic functions with calls to android modules
  - PRESERVE: Tauri plugin API signatures
  - USE: New modular implementations from android/*
  - REMOVE: All inline JNI code

Task 13: MODIFY packages/tauri-plugin-ankidroid-android/src/lib.rs
  - REMOVE: All #[cfg(desktop)] blocks
  - REMOVE: desktop module import
  - ADD: android module declaration
  - ENSURE: Mobile-only configuration

Task 14: MODIFY packages/tauri-plugin-ankidroid-android/Cargo.toml
  - REMOVE: Any desktop-specific dependencies
  - ENSURE: target.'cfg(target_os = "android")' configuration
  - VERIFY: jni = "0.21" is present

Task 15: CREATE packages/tauri-plugin-ankidroid-android/src/android/tests/
  - IMPLEMENT: Unit tests for each module
  - MOCK: JNI environment for testing
  - COVERAGE: Error cases, edge cases, happy path
  - FOLLOW pattern: Standard Rust testing with #[cfg(test)]
```

### Implementation Patterns & Key Details

```rust
// jni_helpers.rs - Critical RAII pattern for exception safety
pub struct SafeJNIEnv<'local> {
    env: JNIEnv<'local>,
}

impl<'local> SafeJNIEnv<'local> {
    pub fn call_method_checked<T>(
        &mut self,
        obj: &JObject,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> Result<T, AndroidError> {
        let result = self.env.call_method(obj, name, sig, args)?;
        
        // CRITICAL: Check for Java exceptions immediately
        if self.env.exception_check()? {
            let ex = self.env.exception_occurred()?;
            self.env.exception_clear()?;
            return Err(AndroidError::JavaException(self.get_exception_message(&ex)?));
        }
        
        Ok(result)
    }
}

// cursor.rs - RAII cursor management
impl<'local> Drop for CursorIterator<'local> {
    fn drop(&mut self) {
        // CRITICAL: Always close cursor to prevent memory leaks
        let _ = self.env.call_method(&self.cursor, "close", "()V", &[]);
    }
}

// cards.rs - Proper field formatting
pub fn create_card(front: &str, back: &str, deck_id: i64, model_id: i64) -> Result<i64> {
    // CRITICAL: Use \u{001f} separator for fields
    let fields = format!("{}\u{001f}{}", front, back);
    
    let values = ContentValues::new()
        .put("mid", model_id)
        .put("did", deck_id)
        .put("flds", fields)
        .build();
    
    // Insert and extract note ID from URI
}
```

### Integration Points

```yaml
TAURI_COMMANDS:
  - location: src/commands.rs
  - pattern: "#[tauri::command] pub async fn create_card(...)"
  - delegates to: android::cards::create_card

JNI_ENTRY:
  - location: src/mobile.rs
  - pattern: "pub fn init() initializes JNI context"
  - uses: ndk_context::android_context()

TYPESCRIPT_API:
  - location: packages/tauri-plugin-ankidroid-js/src/index.ts
  - pattern: "export async function createCard(...)"
  - must match: Rust command signatures
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cd packages/tauri-plugin-ankidroid-android
cargo fmt                              # Format code
cargo clippy -- -D warnings           # Lint with all warnings as errors
cargo check --target aarch64-linux-android  # Type check for Android

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test individual modules
cd packages/tauri-plugin-ankidroid-android
cargo test --lib android::jni_helpers
cargo test --lib android::cursor
cargo test --lib android::cards

# Run all tests
cargo test --all

# Expected: All tests pass. Debug failures before proceeding.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build for Android
cd packages/tauri-plugin-ankidroid-android
cargo build --target aarch64-linux-android --release

# Verify no desktop code remains
! grep -r "cfg(desktop)" src/
! find . -name "*.java" -o -name "*.kt"

# Build the test app
cd ../tauri-plugin-ankidroid-e2e-test-app
npm run tauri android build

# Expected: APK builds successfully without errors
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Install and test on Android emulator
cd packages/tauri-plugin-ankidroid-e2e-test-app

# Start Android emulator
$ANDROID_HOME/emulator/emulator -avd Pixel_6_API_33 &

# Install AnkiDroid first
adb install ../../third-party-apks/AnkiDroid-*.apk

# Install test app
adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk

# Grant permissions
adb shell pm grant com.tauri.ankidroid.test com.ichi2.anki.permission.READ_WRITE_DATABASE

# Run basic smoke test
adb shell am start -n com.tauri.ankidroid.test/.MainActivity

# Check logs for successful card creation
adb logcat | grep -E "ANKIDROID_PLUGIN|create_card"

# Expected: Card creation succeeds, no JNI errors in logs
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] No desktop code remains: `! grep -r "cfg(desktop)" src/`
- [ ] No Java/Kotlin files: `! find . -path "*/android/src/main/java/*"`
- [ ] Builds for Android: `cargo build --target aarch64-linux-android`
- [ ] All clippy warnings resolved: `cargo clippy -- -D warnings`

### Feature Validation

- [ ] Card creation works with proper field separation (\u{001f})
- [ ] Card listing returns valid JSON
- [ ] Card update modifies existing notes
- [ ] Card deletion removes notes from AnkiDroid
- [ ] Error messages are descriptive and actionable

### Code Quality Validation

- [ ] No file exceeds 300 lines
- [ ] All JNI resources use RAII patterns
- [ ] Exception checking after every JNI call
- [ ] Cursor cleanup via Drop trait
- [ ] Proper error propagation with Result types

### Documentation & Deployment

- [ ] Each module has module-level documentation
- [ ] Public functions have doc comments
- [ ] README updated to reflect mobile-only nature
- [ ] No hardcoded values - all use constants.rs

---

## Anti-Patterns to Avoid

- ❌ Don't use unwrap() on JNI operations - always propagate errors
- ❌ Don't forget exception checking after JNI calls
- ❌ Don't manually manage JNI resources without RAII
- ❌ Don't exceed 300 lines per file - split into modules
- ❌ Don't hardcode ContentProvider URIs - use constants
- ❌ Don't forget the \u{001f} field separator