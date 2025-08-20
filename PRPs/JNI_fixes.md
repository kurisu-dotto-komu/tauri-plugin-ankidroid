name: "JNI Integration Fixes PRP - Comprehensive Implementation Guide"
description: |

---

## Goal

**Feature Goal**: Fix all JNI compilation errors (42+ errors) and establish robust, maintainable JNI integration patterns that enable seamless AnkiDroid ContentProvider access from the Tauri plugin on Android.

**Deliverable**: Zero-compilation-error Rust codebase with working AnkiDroid integration, including:
- Standardized SafeJNIEnv ownership patterns 
- Updated JNI API compatibility (v0.21+)
- Robust error handling and memory management
- Comprehensive test coverage for Android operations
- **Visual documentation of working integration via Android screenshots**

**Success Definition**: 
- `cargo build --target aarch64-linux-android` completes without errors
- All AnkiDroid CRUD operations (create/read/update/delete cards) work on real device
- E2E tests pass with actual AnkiDroid ContentProvider integration
- Memory safety validated (no leaks, crashes, or ANR errors)
- **Screenshot evidence of successful card creation and AnkiDroid integration**

## User Persona

**Target User**: Mobile application developers integrating AnkiDroid functionality

**Use Case**: Building cross-platform apps that need to interact with AnkiDroid's spaced repetition system via Tauri mobile plugins

**User Journey**: 
1. Install plugin in Tauri mobile app
2. Call plugin commands to create/read/update/delete AnkiDroid cards
3. Plugin seamlessly communicates with AnkiDroid ContentProvider
4. Users manage flashcards through both AnkiDroid and the integrated app

**Pain Points Addressed**: 
- Current plugin cannot compile for Android due to JNI integration issues
- No working AnkiDroid integration exists for Tauri mobile applications
- Existing JNI patterns conflict with modern Rust ownership semantics

## Why

- **Business Value**: Enables developers to build educational apps leveraging AnkiDroid's proven spaced repetition system
- **Integration Need**: Fills gap between Tauri mobile apps and Android's most popular flashcard application (>10M downloads)
- **Technical Foundation**: Establishes reusable patterns for Tauri plugins requiring complex Android native integration
- **User Impact**: Allows seamless flashcard management across multiple applications without data silos

## What

A completely functional JNI integration layer that:

### User-Visible Behavior:
- Plugin commands execute without compilation errors
- Card CRUD operations complete within 1-2 seconds on real devices
- Proper error messages for AnkiDroid permission/availability issues
- Graceful handling of AnkiDroid app states (installed/not installed, permissions granted/denied)

### Technical Requirements:
- All 42+ compilation errors resolved
- JNI lifetime management follows Rust best practices
- Memory safety guaranteed through RAII patterns
- Thread-safe operations with proper Android context management
- Comprehensive error propagation from JNI through Tauri commands

### Success Criteria

- [ ] Zero compilation errors: `cargo build --target aarch64-linux-android` succeeds
- [ ] Zero linting errors: `cargo clippy --target aarch64-linux-android` passes
- [ ] All unit tests pass: `cargo test` (for non-JNI components)
- [ ] Integration tests pass: Real device testing with AnkiDroid installed
- [ ] Memory safety validated: No leaks during extended use (>100 operations)
- [ ] Performance acceptable: Card operations complete <2 seconds
- [ ] Error handling complete: Graceful failures for all error conditions

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for implementing JNI fixes without prior codebase knowledge, including specific error patterns, modern JNI best practices, Android integration requirements, and exact file locations for all necessary changes._

### Documentation & References

```yaml
# CRITICAL JNI API MIGRATION DOCS - Read these first
- url: https://github.com/jni-rs/jni-rs/blob/master/docs/0.21-MIGRATION.md
  why: Essential migration guide for JNI 0.21+ breaking changes affecting our errors
  critical: JNIEnv now requires 'mut', JValue needs two lifetimes, explicit unsafe blocks required

- url: https://docs.rs/jni/latest/jni/
  why: Official JNI crate documentation for current patterns and API usage
  critical: Auto-local references, thread attachment patterns, global reference management

# TAURI MOBILE PLUGIN PATTERNS
- url: https://v2.tauri.app/develop/plugins/develop-mobile/
  why: Official Tauri mobile plugin development guide and architecture patterns
  critical: Plugin lifecycle, Android configuration, build target setup

- url: https://github.com/tauri-apps/plugins-workspace
  why: Reference implementations for Tauri mobile plugins with native Android integration
  pattern: Examine biometric, NFC, and geolocation plugins for JNI integration patterns

# ANDROID CONTENTPROVIDER INTEGRATION
- url: https://developer.android.com/training/articles/perf-jni
  why: Official Android JNI performance and best practices guide
  critical: Cannot access ContentProvider directly from JNI - must use Java bridge pattern

- url: https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API
  why: AnkiDroid ContentProvider API documentation and integration examples
  critical: Two-tier API structure, permission requirements, URI patterns for cards/decks/models

# CODEBASE PATTERN FILES - Follow these exactly
- file: packages/tauri-plugin-ankidroid-android/src/android/jni_helpers.rs
  why: Current SafeJNIEnv implementation with RAII patterns and exception checking
  pattern: Exception handling, local frame management, string conversions
  gotcha: Mixed ownership patterns causing compilation errors

- file: packages/tauri-plugin-ankidroid-android/src/android/error.rs  
  why: Comprehensive error handling patterns and JNI exception integration
  pattern: AndroidError enum, JniResultExt trait, error context preservation
  gotcha: Mutability conflicts in exception message extraction (line 140)

- file: packages/tauri-plugin-ankidroid-android/src/android/content_provider.rs
  why: ContentProvider integration patterns using ContentValues builder
  pattern: Query/Insert/Update/Delete operations, URI handling, cursor management
  gotcha: ContentValuesBuilder ownership conflicts, temporary JValue lifetimes

- file: packages/tauri-plugin-ankidroid-android/src/mobile.rs
  why: High-level Tauri command integration and thread context management
  pattern: Async command handlers, Android context attachment, error propagation
  gotcha: Thread safety with JNI environments, proper resource cleanup

# AI DOCUMENTATION FOR COMPLEX PATTERNS  
- docfile: PRPs/ai_docs/ankidroid_api_reference.md
  why: Curated AnkiDroid ContentProvider API patterns and integration examples
  section: ContentProvider CRUD patterns, URI structure, permission handling

- docfile: PRPs/ai_docs/tauri_plugin_patterns.md
  why: Tauri mobile plugin architecture patterns and Android lifecycle management
  section: Plugin registration, command handling, Android-specific configuration
```

### Current Codebase tree

```bash
.
|-- packages
|   |-- tauri-plugin-ankidroid-android
|   |   |-- Cargo.toml                    # JNI dependencies (jni = "0.21")
|   |   |-- src
|   |   |   |-- android
|   |   |   |   |-- jni_helpers.rs        # SafeJNIEnv wrapper (42+ errors here)
|   |   |   |   |-- error.rs              # Error handling (mutability issues)
|   |   |   |   |-- content_provider.rs   # ContentProvider ops (lifetime issues)
|   |   |   |   |-- cursor.rs             # Result iteration (borrowing conflicts)
|   |   |   |   |-- cards.rs              # Card CRUD (ownership issues)
|   |   |   |   |-- decks.rs              # Deck operations (move conflicts)
|   |   |   |   `-- models.rs             # Model management
|   |   |   |-- mobile.rs                 # Tauri command layer
|   |   |   `-- lib.rs                    # Module declarations
|   |   `-- android                       # Android native code directory
|   |-- tauri-plugin-ankidroid-e2e-test-app    # Testing application
|   `-- tauri-plugin-ankidroid-js               # JavaScript bindings
|-- target                                      # Build artifacts
|   |-- aarch64-linux-android             # Android ARM64 target
|   |-- armv7-linux-androideabi           # Android ARM32 target  
|   |-- i686-linux-android                # Android x86 target
|   `-- x86_64-linux-android              # Android x64 target
`-- third-party-apks
    `-- AnkiDroid-2.22.3.apk             # Test APK for integration
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
packages/tauri-plugin-ankidroid-android/src/android/
|-- jni_helpers.rs          # UPDATED: Standardized SafeJNIEnv with reference-only API
|-- error.rs                # UPDATED: Fixed mutability in exception handling  
|-- content_provider.rs     # UPDATED: Fixed lifetime issues, builder patterns
|-- cursor.rs               # UPDATED: Fixed iterator borrowing conflicts
|-- cards.rs                # UPDATED: Standardized function signatures to &SafeJNIEnv
|-- decks.rs                # UPDATED: Fixed ownership conflicts in ContentValues
|-- models.rs               # UPDATED: Consistent patterns with other modules
|-- validation.rs           # NEW: Input validation helpers for JNI operations
|-- integration_tests.rs    # NEW: Integration tests for Android device testing
`-- test_helpers.rs         # NEW: Test utilities for JNI operations

# Testing infrastructure
packages/tauri-plugin-ankidroid-android/tests/
|-- integration/            # NEW: Real device integration tests
|   |-- card_operations.rs  # Test card CRUD on real AnkiDroid
|   |-- deck_operations.rs  # Test deck management
|   `-- error_scenarios.rs  # Test error conditions and edge cases
`-- common/                 # NEW: Shared test utilities
    |-- mod.rs               # Test helper modules
    |-- ankidroid_setup.rs   # AnkiDroid test environment setup
    `-- device_setup.rs      # Android device/emulator configuration
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: JNI 0.21+ Breaking Changes
// JNIEnv now requires mutable reference in all method calls
fn old_pattern(env: JNIEnv) -> Result<()> {}        // BROKEN
fn new_pattern(mut env: JNIEnv) -> Result<()> {}    // CORRECT

// CRITICAL: JValue now requires two lifetime parameters  
let old_value: JValue<'local> = (&obj).into();              // BROKEN
let new_value: JValue<'local, 'local> = (&obj).into();      // CORRECT

// CRITICAL: SafeJNIEnv ownership conflicts
fn broken_function(env: SafeJNIEnv) -> Result<()> {         // Takes ownership
    let builder = ContentValuesBuilder::new(env)?;          // Moves env
    env.call_method(/* cannot use env here */)?;             // COMPILE ERROR
}
fn fixed_function(env: &SafeJNIEnv) -> Result<()> {         // Takes reference
    let builder = ContentValuesBuilder::new(env)?;          // Borrows env
    env.call_method(/* env still usable */)?;                // WORKS
}

// CRITICAL: Pop local frame requires explicit unsafe block
env.pop_local_frame(&JObject::null());                      // BROKEN in 0.21+
unsafe { env.pop_local_frame(&JObject::null()); }           // CORRECT

// CRITICAL: Iterator borrowing conflicts in cursor processing
let mut cursor = /* get cursor */;
for row in cursor.iter() {                                  // Borrows cursor mutably
    cursor.move_to_next()?;                                  // COMPILE ERROR - already borrowed
}
// Fix: Use iterator OR manual iteration, not both

// GOTCHA: ContentProvider cannot be accessed directly from JNI
// All ContentProvider operations must go through Java bridge layer
// Use JNI to call Java methods that access ContentProvider

// GOTCHA: Android context management requires proper thread attachment
// JNI environments are thread-local and cannot be shared between threads
// Must use vm.attach_current_thread() for background operations
```

## Implementation Blueprint

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE android/validation.rs
  - IMPLEMENT: Input validation helpers for JNI parameters and Android-specific constraints
  - FUNCTIONS: validate_card_content, validate_deck_name, validate_model_id, sanitize_jni_string
  - NAMING: CamelCase for structs, snake_case for functions
  - DEPENDENCIES: None (pure validation logic)
  - PLACEMENT: packages/tauri-plugin-ankidroid-android/src/android/validation.rs

Task 2: MODIFY android/jni_helpers.rs 
  - UPDATE: SafeJNIEnv to use reference-only API patterns (&SafeJNIEnv everywhere)
  - FIX: Add explicit unsafe blocks for pop_local_frame operations
  - UPDATE: All method signatures to require &mut JNIEnv per JNI 0.21+ requirements
  - PATTERN: Follow RAII LocalFrame management from existing implementation
  - DEPENDENCIES: Task 1 (validation helpers)
  - GOTCHA: Must maintain backward compatibility for existing exception checking patterns

Task 3: MODIFY android/error.rs
  - FIX: Change get_exception_message signature to accept &mut JNIEnv instead of &JNIEnv
  - UPDATE: All JNI exception handling to use mutable environment references
  - MAINTAIN: Existing AndroidError enum and JniResultExt trait patterns
  - DEPENDENCIES: Task 2 (updated SafeJNIEnv)
  - PATTERN: Preserve comprehensive error context and exception message extraction

Task 4: MODIFY android/content_provider.rs
  - FIX: Update all JValue usage to include two lifetime parameters: JValue<'local, 'local>
  - UPDATE: ContentValuesBuilder to accept &SafeJNIEnv instead of owned SafeJNIEnv
  - RESOLVE: Temporary value lifetime issues in JValue::Object creation
  - DEPENDENCIES: Task 2, Task 3 (updated core JNI patterns)
  - PATTERN: Maintain builder pattern for ContentValues, fix ownership conflicts

Task 5: MODIFY android/cursor.rs
  - REDESIGN: Iterator to avoid mutable borrowing conflicts during row processing
  - IMPLEMENT: Either pure iterator pattern OR manual iteration, not both
  - UPDATE: All JNI method calls to use updated SafeJNIEnv patterns
  - DEPENDENCIES: Task 2, Task 3, Task 4 (core JNI foundation)
  - PATTERN: Follow Android Cursor best practices for resource cleanup

Task 6: MODIFY android/cards.rs, android/decks.rs, android/models.rs
  - STANDARDIZE: All function signatures to use &SafeJNIEnv instead of owned SafeJNIEnv
  - UPDATE: All ContentProvider operations to use fixed patterns from Task 4
  - APPLY: Fixed JNI patterns from previous tasks consistently across all modules
  - DEPENDENCIES: Task 1-5 (all core infrastructure fixed)
  - PATTERN: Maintain existing API contracts while fixing compilation issues

Task 7: MODIFY mobile.rs
  - UPDATE: All JNI environment handling to use fixed patterns
  - ENSURE: Proper thread attachment and context management for Android operations
  - INTEGRATE: Updated error handling patterns from Task 3
  - DEPENDENCIES: Task 1-6 (all JNI fixes complete)
  - PATTERN: Maintain async Tauri command structure with proper resource cleanup

Task 8: CREATE android/test_helpers.rs and android/integration_tests.rs
  - IMPLEMENT: Test utilities for JNI operations without requiring real Android device
  - CREATE: Integration test framework for real device validation
  - INCLUDE: Mock AnkiDroid setup and error condition testing
  - DEPENDENCIES: Task 1-7 (working JNI implementation)
  - PATTERN: Follow Rust testing best practices with proper resource management

Task 9: CREATE tests/integration/ directory with comprehensive device tests
  - IMPLEMENT: Real device tests for card/deck/model operations
  - CREATE: Error scenario testing (AnkiDroid not installed, permissions denied)
  - SETUP: AnkiDroid test environment configuration and data preparation
  - DEPENDENCIES: Task 8 (test infrastructure)
  - PATTERN: Use tokio::test for async operations, proper cleanup after tests

Task 10: UPDATE Cargo.toml dependencies and documentation
  - VERIFY: JNI dependency version is compatible (jni = "0.21" or newer)
  - UPDATE: Documentation comments throughout codebase for new patterns
  - CREATE: Usage examples and troubleshooting guide in code comments
  - DEPENDENCIES: Task 1-9 (complete implementation)
  - PATTERN: Follow Rust documentation standards with comprehensive examples
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Updated SafeJNIEnv signature (Task 2)
impl<'local> SafeJNIEnv<'local> {
    // All methods now take &self instead of self, enabling reuse
    pub fn call_method_checked<T>(&self, /* params */) -> AndroidResult<T>
    pub fn new_string_checked(&self, s: &str) -> AndroidResult<JString<'local>>
    
    // CRITICAL: Wrap unsafe operations explicitly
    pub fn pop_local_frame_safe(&self) -> AndroidResult<()> {
        unsafe {
            let _ = self.env.pop_local_frame(&JObject::null());
        }
        Ok(())
    }
}

// PATTERN: Fixed JValue usage with two lifetimes (Task 4)
fn create_content_values<'local>(
    env: &SafeJNIEnv<'local>, 
    values: &[(&str, &str)]
) -> AndroidResult<JObject<'local>> {
    // OLD (broken): JValue::Object(&string.into())
    // NEW (correct): Proper lifetime management
    let string_obj = env.new_string_checked(value)?;
    let jvalue: JValue<'local, 'local> = (&string_obj).into();
    
    // Use auto_local for automatic reference management
    let auto_string = env.auto_local(string_obj);
    // Reference automatically cleaned up when auto_local drops
}

// PATTERN: Fixed function signatures to use references (Task 6)
pub fn create_card(
    env: &SafeJNIEnv,           // Reference instead of owned
    activity: &JObject,
    deck_id: i64,
    front: &str,
    back: &str,
) -> AndroidResult<i64> {
    // Can reuse env multiple times without ownership conflicts
    let content_values = create_content_values(env, &[
        ("front", front),
        ("back", back),
    ])?;
    
    let result = env.call_method(/* uses env again */)?;
    Ok(result)
}

// PATTERN: Fixed iterator design (Task 5)
impl<'local> CursorIterator<'local> {
    // Either pure iterator pattern...
    pub fn collect_all(&mut self) -> AndroidResult<Vec<CursorRow>> {
        let mut results = Vec::new();
        while self.cursor.move_to_next()? {
            results.push(self.extract_current_row()?);
        }
        Ok(results)
    }
    
    // ...OR manual iteration, but not both simultaneously
    pub fn process_rows<F>(&mut self, mut processor: F) -> AndroidResult<()> 
    where F: FnMut(&CursorRow) -> AndroidResult<()> {
        while self.cursor.move_to_next()? {
            let row = self.extract_current_row()?;
            processor(&row)?;
        }
        Ok(())
    }
}

// PATTERN: Thread-safe Android context management (Task 7)
pub async fn execute_android_operation<F, T>(operation: F) -> AndroidResult<T>
where 
    F: FnOnce(&SafeJNIEnv) -> AndroidResult<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let vm = get_java_vm()?;
        let env = vm.attach_current_thread()?;
        let safe_env = SafeJNIEnv::new(env);
        
        // Execute operation with properly attached thread
        let result = operation(&safe_env)?;
        
        // Thread automatically detached when env drops
        Ok(result)
    }).await.map_err(|e| AndroidError::ThreadingError(e.to_string()))?
}
```

### Integration Points

```yaml
BUILD_CONFIGURATION:
  - cargo_toml: "Ensure jni = '0.21+' and Android targets configured"
  - android_manifest: "Verify AnkiDroid ContentProvider permissions declared"
  - rust_toolchain: "Android targets installed: aarch64/armv7/i686/x86_64-linux-android"

TESTING_INTEGRATION:
  - device_setup: "Android emulator or real device with AnkiDroid installed"
  - permission_setup: "Grant AnkiDroid API access permissions during test setup"
  - data_isolation: "Use test-specific AnkiDroid profiles to avoid data contamination"

TAURI_PLUGIN_REGISTRATION:
  - plugin_config: "Register with Tauri builder in src-tauri/src/main.rs"
  - permissions: "Configure Android permissions in tauri.conf.json"
  - build_scripts: "Ensure Android build scripts handle JNI compilation correctly"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --target aarch64-linux-android          # Android compilation check
cargo clippy --target aarch64-linux-android         # Android-specific linting
cargo fmt                                            # Consistent formatting

# Expected: Zero errors. Read output carefully and fix compilation issues before proceeding.
# Pay special attention to lifetime and ownership errors in JNI code.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test each component as it's created
cargo test --lib                                     # Unit tests (non-JNI components)
cargo test android::validation --target aarch64-linux-android    # Validation module tests

# Integration tests require Android device
cargo test --test integration --target aarch64-linux-android     # Device integration tests

# Expected: All tests pass. Debug JNI-related failures with adb logcat output.
```

### Level 3: Integration Testing (System Validation)

```bash
# Android emulator/device setup validation
adb devices                                          # Verify device connectivity
adb shell pm list packages | grep anki              # Verify AnkiDroid installed

# Plugin build and deployment
cargo build --target aarch64-linux-android          # Build for Android
# Deploy to test app and verify plugin loading

# AnkiDroid ContentProvider access validation
adb shell content query --uri content://com.ichi2.anki.flashcards/notes
# Expected: Returns AnkiDroid data or proper permission error

# E2E testing with real AnkiDroid integration
cd packages/tauri-plugin-ankidroid-e2e-test-app
npm run test:android                                 # Run WebDriver tests

# SCREENSHOT VALIDATION: Capture visual proof of working integration
adb shell screencap -p /sdcard/before_card_creation.png
adb pull /sdcard/before_card_creation.png ./screenshots/

# Test card creation through plugin
# [Execute card creation command through test app]

adb shell screencap -p /sdcard/after_card_creation.png  
adb pull /sdcard/after_card_creation.png ./screenshots/

# Open AnkiDroid to verify card was created
adb shell am start -n com.ichi2.anki/.DeckPicker
sleep 3
adb shell screencap -p /sdcard/ankidroid_with_new_card.png
adb pull /sdcard/ankidroid_with_new_card.png ./screenshots/

# Expected: All operations complete successfully, proper error handling for edge cases
# Expected: Screenshots show successful card creation visible in AnkiDroid interface
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Memory leak detection during extended use
adb shell dumpsys meminfo [app_package] | grep -A 5 "Native Heap"
# Run 100+ card operations and monitor for memory growth

# ANR (Application Not Responsive) detection
adb logcat | grep -i "anr\|application not responding"
# Verify no ANR errors during JNI operations

# JNI local reference monitoring
adb logcat | grep -i "jni.*reference\|local reference"
# Check for local reference table overflow warnings

# Performance benchmarking
time adb shell am start -n [test_app]/[main_activity]
# Measure app startup time with plugin loaded

# AnkiDroid compatibility testing across versions
# Test with AnkiDroid 2.22+, 2.21, and 2.20 if available
adb install --replace AnkiDroid-[version].apk
# Verify plugin works across AnkiDroid versions

# Thread safety validation under load
# Run concurrent operations from multiple threads
# Monitor for race conditions and deadlocks

# COMPREHENSIVE SCREENSHOT DOCUMENTATION
mkdir -p ./screenshots/validation/
adb shell screencap -p /sdcard/plugin_integration_test.png
adb pull /sdcard/plugin_integration_test.png ./screenshots/validation/

# Document multiple card creation scenarios
for i in {1..5}; do
    # Create card with different content
    # [Execute create card command with test data $i]
    adb shell screencap -p /sdcard/card_creation_$i.png
    adb pull /sdcard/card_creation_$i.png ./screenshots/validation/
done

# Final AnkiDroid state showing all created cards
adb shell am start -n com.ichi2.anki/.DeckPicker
sleep 2
adb shell screencap -p /sdcard/final_ankidroid_state.png
adb pull /sdcard/final_ankidroid_state.png ./screenshots/validation/

# Create summary documentation
echo "Screenshot validation completed: $(ls ./screenshots/validation/ | wc -l) images captured"

# Expected: No memory leaks, ANR errors, or performance degradation under normal usage
# Expected: Complete visual documentation of working card creation and AnkiDroid integration
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] Zero compilation errors: `cargo build --target aarch64-linux-android`
- [ ] Zero linting errors: `cargo clippy --target aarch64-linux-android`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test integration --target aarch64-linux-android`

### Feature Validation

- [ ] All AnkiDroid CRUD operations work on real device
- [ ] Error handling graceful for all failure conditions
- [ ] Performance acceptable: operations complete <2 seconds
- [ ] Memory safety confirmed: no leaks after 100+ operations
- [ ] Thread safety validated: concurrent operations work correctly
- [ ] AnkiDroid compatibility confirmed across supported versions
- [ ] **Screenshot evidence captured**: Before/after card creation and AnkiDroid verification
- [ ] **Visual validation complete**: Screenshots saved to `./screenshots/` directory

### Code Quality Validation

- [ ] Follows established SafeJNIEnv patterns consistently
- [ ] File placement matches desired codebase tree structure
- [ ] Anti-patterns avoided (ownership conflicts, unsafe code without justification)
- [ ] JNI resource management follows RAII principles
- [ ] Error context preserved through all layers

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] JNI operations have comprehensive error logging
- [ ] Integration examples provided for common use cases
- [ ] Troubleshooting guide updated for JNI-specific issues

---

## Anti-Patterns to Avoid

- ❌ Don't take ownership of SafeJNIEnv when reference would work
- ❌ Don't use old JNI patterns without checking 0.21+ compatibility
- ❌ Don't ignore thread safety - JNI environments are thread-local
- ❌ Don't skip explicit unsafe blocks for JNI operations requiring them
- ❌ Don't access ContentProvider directly from JNI - use Java bridge
- ❌ Don't mix iterator and manual cursor operations simultaneously
- ❌ Don't assume AnkiDroid is always available - handle graceful failure
- ❌ Don't skip local reference cleanup - use RAII patterns consistently