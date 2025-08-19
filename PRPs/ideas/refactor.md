# Refactor

## Overview

We are refactoring our WIP Tauri plugin project to embrace JNI calls in Rust while maintaining a clean, modular, and maintainable architecture. This will be a mobile-only plugin specifically for Android/AnkiDroid integration, with no desktop support.

## Strategy

We will leverage Rust's JNI capabilities directly, but structure the code in a maintainable way with proper abstractions, error handling, and modular organization. This approach keeps everything within the Rust ecosystem while ensuring the code remains clean and easy to work with.

**Key Decisions:**
- **No Desktop Support**: Remove all desktop mock implementations and `#[cfg(not(target_os = "android"))]` code blocks
- **Pure Rust Implementation**: No Java/Kotlin files in the Rust project - all JNI calls will be made directly from Rust
- **Mobile-Only Build Target**: Configure the plugin to only build for Android targets

### Current Problems to Address

After analyzing the existing implementation, we've identified several critical issues:

1. **Deeply Nested Cursor Operations**: The code has excessive nesting when iterating through database cursors (200+ lines for a single function), making it hard to read and maintain.

2. **Repetitive Exception Handling**: The same exception checking and clearing pattern is repeated throughout, without a reusable abstraction.

3. **Manual Resource Management**: Cursors and other JNI resources are manually managed without RAII patterns, risking resource leaks.

4. **Monolithic Functions**: Single functions handle multiple responsibilities (querying models, creating decks, inserting notes) in 600+ lines.

5. **No Type Safety**: Raw JNI calls without type-safe wrappers, making errors likely and debugging difficult.

6. **Hardcoded String Literals**: ContentProvider URIs and column names scattered throughout as string literals.

The key diagram to understand our new architecture is below:

```
┌─────────────────┐
│   JavaScript    │  Frontend API
│  (TypeScript)   │  Clean, typed interface
└────────┬────────┘
         │ JSON
┌────────▼────────┐
│      Rust       │  Business Logic + JNI
│  (mobile.rs)    │  Modular JNI implementation
│                 │  Direct AnkiDroid interaction
└─────────────────┘
         │
         │ JNI Calls
         ▼
┌─────────────────┐
│   AnkiDroid     │  Android App
│     (Java)      │  Content Provider API
└─────────────────┘
```

The Rust layer will be well-structured with:
- Separate modules for different concerns (cards, decks, models, etc.)
- Proper JNI abstractions and helper functions
- Strong error handling with custom error types
- Clean separation between JNI boilerplate and business logic

### Proposed Solutions

1. **Cursor Iterator Pattern**: Create a reusable `CursorIterator` that encapsulates cursor navigation and automatically handles resource cleanup.

2. **JNI Helper Module**: Build a `jni_helpers` module with functions like `with_exception_check`, `safe_call_method`, and `get_content_resolver`.

3. **RAII Wrappers**: Implement Drop traits for JNI resources to ensure automatic cleanup.

4. **Feature Modules**: Split functionality into focused modules:
   - `content_provider.rs` - ContentProvider abstraction
   - `cursor.rs` - Cursor iteration helpers
   - `cards.rs` - Card CRUD operations
   - `decks.rs` - Deck operations
   - `models.rs` - Model operations
   - `error.rs` - Custom error types

5. **Type-Safe Wrappers**: Create structs for AnkiDroid entities (Card, Deck, Model) with proper serialization.

6. **Constants Module**: Define all ContentProvider URIs and column names as constants.

Eventually, we want to implement all of the default AnkiDroid methods, but for now just focus on Card CRUD.

We will implement helper methods directly in Rust, like ensuring a deck id is valid, or ensuring a model id is valid. The REFERENCE_IMPLEMENTATION.md file can provide guidance on what validation and helpers are needed.

See `PRPs/planning/API.md` for the proposed full JS API - we will not implement all of this yet, but it should give you an idea of where we are going, and again, not a final spec.

The JS api will include the full method names and expose typed parameters, calling into our Rust implementation.

## Goals

The primary goal is maintainability and readability.

We want to ensure that we keep file sizes down, and not have mega files, so we want modular files organized by feature (cards.rs, decks.rs, models.rs, etc.).

Keep the existing `tauri-plugin-ankidroid-android` package structure but refactor the Rust code within it to be more modular and maintainable.

Focus on keeping things simple while embracing Rust's JNI capabilities.

We want to ensure coverage with unit tests and end to end tests. This will help us develop faster and more confidently.

We can save e2e tests for a later pass, which will use the `e2e-test-app` package. For now, include unit tests for the Rust JNI implementation.

For the first pass, keep the scope limited to card CRUD. We will need to fetch a model id and deck id to enable this, probably.

We will build out other features later (such as media handling, deck management, etc.)

During the refactor, get rid of all superfluous/dead code, across all of our packages.

We need to include a strong verification loop for each package, utilizing tests and the `npm run quickfix` scripts made available in each package.

Make sure we are target builds to deploy only on mobile. Remove all desktop support and mock implementations. Ensure dev deployments are the correct architecture for Android only.

## Plan

A high level overview of the plan:

1. **Remove Desktop Support and Java/Kotlin Files**:
   - Delete all `#[cfg(not(target_os = "android"))]` code blocks
   - Remove `desktop.rs` and all mock implementations
   - Delete Java/Kotlin files from `android/src/main/java/`
   - Update Cargo.toml to only target Android

2. **Create Core JNI Infrastructure**:
   - Build `jni_helpers.rs` with reusable patterns for exception handling
   - Implement `cursor.rs` with iterator pattern for database operations
   - Create `content_provider.rs` for ContentProvider abstraction
   - Define `constants.rs` with all URIs and column names

3. **Refactor the `tauri-plugin-ankidroid-android` package's Rust code**:
   - Extract card operations from the monolithic functions into `cards.rs`
   - Extract deck operations into `decks.rs`
   - Extract model operations into `models.rs`
   - Create `error.rs` with custom error types and proper error propagation
   - Replace all inline JNI calls with helper functions
   - Add comprehensive unit tests for each module

4. **Update the TypeScript package (`tauri-plugin-ankidroid-js`)** to match the new Rust API

5. **Clean up all dead code and unused imports** across packages

6. **Update frontend test app** to use the refactored implementation

7. **Ensure we can build and run the app on Android** (no desktop builds)

8. **Write basic happy-path e2e tests** using the `e2e-test-app` package for Android only

In total we will have 3 packages:

- `tauri-plugin-ankidroid-js` (TypeScript bindings)
- `tauri-plugin-ankidroid-android` (Rust with JNI)
- `tauri-plugin-ankidroid-e2e-test-app` (Test application)

The focus is on making the Rust JNI code maintainable through proper modularization and abstractions, rather than adding additional layers of indirection.

## File Structure

The refactored `tauri-plugin-ankidroid-android/src` directory will have:

```
src/
├── lib.rs              # Main plugin entry point (Android-only)
├── commands.rs         # Tauri command definitions
├── android/            # All Android implementations
│   ├── mod.rs          # Module exports and main entry
│   ├── constants.rs    # ContentProvider URIs and column names
│   ├── error.rs        # Custom error types
│   ├── jni_helpers.rs  # JNI utility functions
│   ├── cursor.rs       # Cursor iterator and helpers
│   ├── content_provider.rs # ContentProvider abstraction
│   ├── models.rs       # Model operations
│   ├── decks.rs        # Deck operations
│   └── cards.rs        # Card CRUD operations
└── types.rs            # Shared type definitions
```

Note: No `desktop.rs` file - this is a mobile-only plugin. The existing Java/Kotlin files in `android/src/main/java/` will be removed as all JNI interactions will be handled directly from Rust.
