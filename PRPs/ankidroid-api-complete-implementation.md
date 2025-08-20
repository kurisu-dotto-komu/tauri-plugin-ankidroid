name: "AnkiDroid API Complete Implementation - Fix All JNI Exceptions and Implement Full API"
description: |

---

## Goal

**Feature Goal**: Implement complete AnkiDroid API integration via JNI with proper exception handling, fixing all current errors and providing comprehensive API coverage

**Deliverable**: Fully functional Tauri plugin with all AnkiDroid API methods implemented correctly, no exceptions, proper error handling, and complete documentation

**Success Definition**: All API methods work without exceptions, cards can be created/read/updated/deleted successfully, media handling works, deck/model management is complete, and all edge cases are handled

## Why

- Current implementation has multiple NullPointerException and "Queue unknown" errors that prevent basic functionality
- Incomplete API coverage limits the plugin's usefulness
- Poor exception handling leads to app crashes
- Missing critical operations like media handling, deck management, and card suspension
- Need robust implementation that handles all AnkiDroid states (app not running, permission denied, etc.)

## What

Complete implementation of the AnkiDroid API that:
- Fixes all current exceptions (NullPointerException, Queue unknown, card modified)
- Implements all API methods from the AnkiDroid ContentProvider
- Provides proper error handling and recovery
- Includes media operations
- Handles permissions correctly
- Works when AnkiDroid is not running
- Provides comprehensive TypeScript API matching the Rust implementation

### Success Criteria

- [ ] No JNI exceptions during normal operations
- [ ] All CRUD operations work for cards/notes
- [ ] Media upload and management works
- [ ] Deck and model operations complete
- [ ] Permission handling robust
- [ ] Graceful fallback when AnkiDroid unavailable
- [ ] Comprehensive error messages
- [ ] All tests passing

## All Needed Context

### Context Completeness Check

_This PRP contains everything needed to implement the AnkiDroid API correctly, including exact method signatures, exception fixes, and proper JNI patterns._

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API
  why: Official API documentation
  critical: Content Provider URIs, permissions, field separators

- file: packages/tauri-plugin-ankidroid-android/src/android/cards.rs
  why: Current implementation with issues to fix
  pattern: Exception patterns, incorrect ContentValues usage
  gotcha: Line 61 shows 'did' exclusion issue causing Queue errors

- file: packages/tauri-plugin-ankidroid-android/src/android/error.rs
  why: Error handling infrastructure
  pattern: JniResultExt trait for exception checking
  critical: Exception checking must happen after EVERY JNI call

- file: PRPs/ai_docs/ankidroid_complete_api_reference.md
  why: Complete API reference with all method signatures
  section: All sections - this is the authoritative reference
  critical: Section 6 has all exception fixes

- file: PRPs/planning/REFERENCE_IMPLEMENTATION.md
  why: Working Java implementation examples
  pattern: suspendCard, changeDeck, addMedia methods
  critical: Shows correct ContentProvider usage patterns

- file: PRPs/planning/API.md
  why: TypeScript API interface specification
  pattern: Complete API surface that needs implementation
  critical: Must match these signatures in final implementation
```

### Current Codebase Structure

```bash
packages/tauri-plugin-ankidroid-android/src/
├── android/
│   ├── mod.rs          # Module exports
│   ├── cards.rs        # Card operations (has bugs)
│   ├── decks.rs        # Deck operations
│   ├── models.rs       # Model operations
│   ├── jni_helpers.rs  # JNI utilities
│   ├── error.rs        # Error handling
│   ├── cursor.rs       # Cursor operations
│   ├── content_provider.rs # ContentProvider interface
│   ├── validation.rs   # Input validation
│   └── constants.rs    # API constants
├── commands.rs         # Tauri commands
├── lib.rs             # Plugin entry
├── mobile.rs          # Mobile implementation
└── types.rs           # Data structures
```

### Known Issues and Fixes

```rust
// ISSUE 1: Queue 'X' is unknown error
// WRONG - in cards.rs line 61
values.put("did", deck_id);  // This causes the error

// FIX - Remove 'did' from ContentValues
values.put("mid", model_id);
values.put("flds", fields);
values.put("tags", tags);
// DO NOT include 'did' - deck is set via model's default

// ISSUE 2: NullPointerException on Collection.getConf()
// CAUSE: AnkiDroid not running or collection not initialized
// FIX: Check if AnkiDroid available before operations
if AddContentApi.getAnkiDroidPackageName(context) == null {
    return Err(AndroidError::AnkiDroidNotAvailable);
}

// ISSUE 3: Field separator incorrect
// WRONG
let fields = format!("{},{}", front, back);

// FIX - Must use unit separator
const FIELD_SEPARATOR: &str = "\u{001f}";
let fields = format!("{}{}{}", front, FIELD_SEPARATOR, back);

// ISSUE 4: Memory leak in attach_current_thread
// WRONG - jni_helpers.rs line 306
let leaked_guard = Box::leak(Box::new(guard));

// FIX - Use proper RAII pattern
struct AttachGuard<'a> {
    vm: &'a JavaVM,
}
impl Drop for AttachGuard<'_> {
    fn drop(&mut self) {
        let _ = self.vm.detach_current_thread();
    }
}
```

## Implementation Blueprint

### Data Models and Structure

```rust
// types.rs - Complete data models
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub field_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
    pub available: bool,
    pub permission_granted: bool,
    pub ankidroid_version: Option<String>,
    pub api_version: Option<i32>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: FIX packages/tauri-plugin-ankidroid-android/src/android/cards.rs
  - REMOVE: Line 61 - values.put("did", deck_id) 
  - FIX: Field separator to use \u{001f}
  - FIX: Exception handling after every JNI call
  - ADD: Proper deck assignment after note creation
  - IMPLEMENT: Move cards to deck as separate operation

Task 2: FIX packages/tauri-plugin-ankidroid-android/src/android/jni_helpers.rs
  - FIX: Memory leak in attach_current_thread (line 306)
  - IMPLEMENT: Proper AttachGuard with Drop trait
  - FIX: Unsafe transmute in clone (line 17)
  - ADD: Exception message extraction helper

Task 3: ENHANCE packages/tauri-plugin-ankidroid-android/src/android/error.rs
  - ADD: AnkiDroidNotAvailable error type
  - ADD: PermissionDenied error type
  - ADD: ModelNotFound error type
  - ADD: DeckNotFound error type
  - IMPROVE: Error messages with actionable information

Task 4: CREATE packages/tauri-plugin-ankidroid-android/src/android/api_check.rs
  - IMPLEMENT: check_ankidroid_available function
  - IMPLEMENT: check_permission function
  - IMPLEMENT: request_permission function
  - IMPLEMENT: get_api_version function
  - PATTERN: Use AddContentApi.getAnkiDroidPackageName pattern

Task 5: CREATE packages/tauri-plugin-ankidroid-android/src/android/media.rs
  - IMPLEMENT: add_media function
  - IMPLEMENT: add_media_from_url function
  - IMPLEMENT: add_media_from_base64 function
  - FOLLOW: REFERENCE_IMPLEMENTATION.md addMedia pattern
  - CRITICAL: Use FileProvider for file URIs

Task 6: ENHANCE packages/tauri-plugin-ankidroid-android/src/android/models.rs
  - FIX: find_basic_model to check field count
  - IMPLEMENT: create_basic_model function
  - IMPLEMENT: create_custom_model function
  - IMPLEMENT: get_model_list function
  - IMPLEMENT: get_field_list function

Task 7: ENHANCE packages/tauri-plugin-ankidroid-android/src/android/decks.rs
  - IMPLEMENT: get_selected_deck_id function
  - IMPLEMENT: get_deck_list function
  - IMPLEMENT: create_deck_hierarchy function (for :: notation)
  - FIX: Proper deck creation with error handling

Task 8: CREATE packages/tauri-plugin-ankidroid-android/src/android/card_management.rs
  - IMPLEMENT: suspend_card function
  - IMPLEMENT: unsuspend_card function
  - IMPLEMENT: change_deck function
  - IMPLEMENT: bury_card function
  - FOLLOW: REFERENCE_IMPLEMENTATION.md patterns

Task 9: CREATE packages/tauri-plugin-ankidroid-android/src/android/sync.rs
  - IMPLEMENT: trigger_sync function
  - IMPLEMENT: check_last_sync function
  - RESPECT: 5-minute cooldown between syncs
  - USE: Intent com.ichi2.anki.DO_SYNC

Task 10: ENHANCE packages/tauri-plugin-ankidroid-android/src/android/content_provider.rs
  - ADD: bulk_insert function for multiple notes
  - ADD: apply_batch for transactional operations
  - IMPROVE: Query builder with limit support
  - ADD: URI builder for special operations

Task 11: UPDATE packages/tauri-plugin-ankidroid-android/src/commands.rs
  - ADD: check_api_status command
  - ADD: add_media command
  - ADD: suspend_card command
  - ADD: change_deck command
  - ADD: trigger_sync command
  - ADD: get_deck_list command
  - ADD: get_model_list command

Task 12: UPDATE packages/tauri-plugin-ankidroid-android/src/mobile.rs
  - INTEGRATE: All new modules
  - FIX: Error handling for AnkiDroid not available
  - ADD: Permission checking before operations
  - IMPLEMENT: Fallback strategies

Task 13: UPDATE packages/tauri-plugin-ankidroid-js/src/index.ts
  - ADD: All new TypeScript API methods
  - MATCH: API.md interface specifications
  - ADD: Proper TypeScript types
  - ADD: JSDoc documentation

Task 14: CREATE tests for all new functionality
  - UNIT: Test each module independently
  - INTEGRATION: Test API calls end-to-end
  - ERROR: Test error conditions
  - PERMISSION: Test permission scenarios
```

### Implementation Patterns & Key Details

```rust
// api_check.rs - Critical availability checking
pub fn check_ankidroid_available(env: &mut JNIEnv, context: &JObject) -> AndroidResult<bool> {
    // Call AddContentApi.getAnkiDroidPackageName(context)
    let api_class = env.find_class("com/ichi2/anki/api/AddContentApi")?;
    let package_name = env.call_static_method(
        api_class,
        "getAnkiDroidPackageName",
        "(Landroid/content/Context;)Ljava/lang/String;",
        &[JValue::Object(context)]
    )?.l()?;
    
    Ok(!package_name.is_null())
}

// cards.rs - Fixed card creation
pub fn create_card_fixed(
    env: &mut SafeJNIEnv,
    activity: &JObject,
    front: String,
    back: String,
    deck_name: Option<String>,
    tags: Option<String>,
    model_id: Option<i64>,
) -> AndroidResult<i64> {
    // Check AnkiDroid availability first
    if !check_ankidroid_available(env, activity)? {
        return Err(AndroidError::AnkiDroidNotAvailable);
    }
    
    // Get or find model
    let model_id = match model_id {
        Some(id) => id,
        None => find_basic_model_id(env, activity)?
    };
    
    // Prepare fields with CORRECT separator
    let fields = format!("{}\u{001f}{}", front, back);
    
    // Create ContentValues WITHOUT deck_id
    let values = ContentValues::new()
        .put_long("mid", model_id)?
        .put_string("flds", &fields)?
        .put_string("tags", &tags.unwrap_or_default())?
        .build(env)?;
    
    // Insert note
    let uri = insert(env, NOTES_URI, values, activity)?;
    let note_id = extract_id_from_uri(env, &uri)?;
    
    // Handle deck assignment separately if specified
    if let Some(deck) = deck_name {
        let deck_id = find_or_create_deck(env, activity, &deck)?;
        move_all_cards_to_deck(env, activity, note_id, deck_id)?;
    }
    
    Ok(note_id)
}

// media.rs - Media handling
pub fn add_media(
    env: &mut SafeJNIEnv,
    activity: &JObject,
    filename: String,
    data: MediaData,
) -> AndroidResult<String> {
    let content_resolver = get_content_resolver(env, activity)?;
    
    // Handle different data sources
    let file_bytes = match data {
        MediaData::Url(url) => download_from_url(&url)?,
        MediaData::Base64(base64) => decode_base64(&base64)?,
        MediaData::Bytes(bytes) => bytes,
    };
    
    // Save to cache directory
    let cache_dir = get_cache_dir(env, activity)?;
    let file_path = format!("{}/{}", cache_dir, filename);
    std::fs::write(&file_path, file_bytes)?;
    
    // Get URI from FileProvider
    let file_uri = get_file_provider_uri(env, activity, &file_path)?;
    
    // Grant permission to AnkiDroid
    grant_uri_permission(env, activity, &file_uri, "com.ichi2.anki")?;
    
    // Insert via ContentProvider
    let values = ContentValues::new()
        .put_string("file_uri", &file_uri)?
        .put_string("preferred_name", &filename)?
        .build(env)?;
    
    let result_uri = insert(env, MEDIA_URI, values, activity)?;
    
    // Extract actual filename used
    Ok(extract_filename_from_uri(env, &result_uri)?)
}

// card_management.rs - Card operations
pub fn suspend_card(
    env: &mut SafeJNIEnv,
    activity: &JObject,
    note_id: i64,
    card_ord: i32,
) -> AndroidResult<bool> {
    let values = ContentValues::new()
        .put_long("note_id", note_id)?
        .put_int("card_ord", card_ord)?
        .put_int("suspend", 1)?
        .build(env)?;
    
    let rows_updated = update(env, REVIEW_INFO_URI, values, None, None, activity)?;
    Ok(rows_updated > 0)
}
```

### Integration Points

```yaml
PERMISSIONS:
  - manifest: com.ichi2.anki.permission.READ_WRITE_DATABASE
  - runtime: Check and request for Android M+
  - fallback: Use intent-based API if denied

CONTENT_PROVIDER:
  - authority: com.ichi2.anki.flashcards
  - fallback: com.ichi2.anki.provider for older versions
  - uris: All URIs from constants.rs

ERROR_HANDLING:
  - pattern: Check exception after EVERY JNI call
  - recovery: Graceful fallback when AnkiDroid unavailable
  - messages: User-friendly error descriptions

TYPESCRIPT_API:
  - location: packages/tauri-plugin-ankidroid-js/src/index.ts
  - pattern: Match API.md specifications exactly
  - types: Full TypeScript type definitions
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
cd packages/tauri-plugin-ankidroid-android
cargo fmt
cargo clippy -- -D warnings
cargo check --target aarch64-linux-android

# Expected: Zero errors
```

### Level 2: Unit Tests (Component Validation)

```bash
cd packages/tauri-plugin-ankidroid-android
cargo test --lib android::cards
cargo test --lib android::media
cargo test --lib android::api_check
cargo test --all

# Expected: All tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Build for Android
cargo build --target aarch64-linux-android --release

# Build test app
cd ../tauri-plugin-ankidroid-e2e-test-app
npm run tauri android build

# Install on emulator
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk

# Grant permissions
adb shell pm grant com.tauri.ankidroid.test com.ichi2.anki.permission.READ_WRITE_DATABASE

# Run E2E tests
npm run test:e2e

# Expected: All E2E tests pass
```

### Level 4: AnkiDroid Integration Testing

```bash
# Test with AnkiDroid not installed
adb uninstall com.ichi2.anki
npm run test:e2e:no-anki
# Expected: Graceful fallback, no crashes

# Test with AnkiDroid installed but not running
adb install AnkiDroid.apk
adb shell am force-stop com.ichi2.anki
npm run test:e2e:anki-stopped
# Expected: Appropriate error messages

# Test with AnkiDroid running
adb shell am start -n com.ichi2.anki/.DeckPicker
npm run test:e2e:full
# Expected: All operations succeed

# Test permission denial
adb shell pm revoke com.tauri.ankidroid.test com.ichi2.anki.permission.READ_WRITE_DATABASE
npm run test:e2e:no-permission
# Expected: Falls back to intent API

# Verify no exceptions in logs
adb logcat | grep -E "Exception|Error" | grep -v "No errors"
# Expected: No JNI exceptions
```

## Final Validation Checklist

### Technical Validation

- [ ] No JNI exceptions in any scenario
- [ ] All 4 validation levels pass
- [ ] Memory leaks fixed (verified with profiler)
- [ ] Thread safety verified
- [ ] RAII patterns used for all resources

### Feature Validation

- [ ] Card CRUD operations work
- [ ] Media upload works
- [ ] Deck management works
- [ ] Model management works
- [ ] Sync triggering works
- [ ] Card suspension/burial works
- [ ] Bulk operations work
- [ ] Permission handling robust

### Code Quality Validation

- [ ] All exceptions handled properly
- [ ] Error messages user-friendly
- [ ] No hardcoded values
- [ ] All unsafe blocks justified
- [ ] Documentation complete

### API Completeness

- [ ] All methods from API.md implemented
- [ ] TypeScript types match Rust types
- [ ] All ContentProvider operations covered
- [ ] Intent fallback implemented
- [ ] API status checking works

---

## Anti-Patterns to Avoid

- ❌ Don't include 'did' in note ContentValues - causes Queue errors
- ❌ Don't use comma or newline as field separator - must use \u{001f}
- ❌ Don't skip exception checking after JNI calls
- ❌ Don't leak memory with Box::leak
- ❌ Don't assume AnkiDroid is running
- ❌ Don't hardcode model or deck IDs
- ❌ Don't ignore cursor cleanup
- ❌ Don't transmute JNIEnv unsafely
- ❌ Don't ignore permission requirements
- ❌ Don't use blocking I/O in async context

## Success Metrics

**Confidence Score**: 10/10 - This PRP provides complete information to fix all issues and implement the full API

**Validation**: Following this PRP will result in a fully functional AnkiDroid API integration with zero exceptions and complete feature coverage