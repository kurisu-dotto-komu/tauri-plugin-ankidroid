# Tauri AnkiDroid Plugin Architecture Plan

## Overview

Transform the current architecture from direct JNI calls in Rust to a cleaner RPC-style pattern where Kotlin handles all Android/AnkiDroid logic, and Rust acts as a thin proxy layer.

## Current Problems

- **Complex JNI code**: 1000+ lines of error-prone JNI code in `mobile.rs`
- **Difficult error handling**: JNI exceptions are hard to manage properly
- **Data structure complexity**: Manual field access and type conversions
- **Maintenance burden**: Debugging JNI issues is challenging
- **Code duplication**: Both Kotlin and Rust implement similar logic

## Proposed Architecture

### 1. Three-Layer Design

```
┌─────────────────┐
│   JavaScript    │  Frontend API
│  (TypeScript)   │  Clean, typed interface
└────────┬────────┘
         │
┌────────▼────────┐
│      Rust       │  Thin RPC Proxy
│  (commands.rs)  │  Single entry point
└────────┬────────┘
         │
┌────────▼────────┐
│     Kotlin      │  Business Logic
│(AnkiDroidPlugin)│  All Android/AnkiDroid interaction
└─────────────────┘
```

### 2. RPC Communication Pattern

#### Single Entry Point

- One Rust command: `execute_rpc(method: String, params: Value)`
- One Kotlin command: `executeRpc(invoke: Invoke)`
- Standardized request/response format

#### Message Format

```typescript
// Request
{
  method: "createCard" | "listCards" | "addMedia" | ...,
  params: { /* method-specific parameters */ }
}

// Response
{
  success: boolean,
  data?: any,
  error?: string
}
```

### 3. Component Responsibilities

#### Rust Layer (`mobile.rs`)

- **Role**: Ultra-thin proxy
- **Responsibilities**:
  - Serialize/deserialize messages
  - Forward calls to Kotlin via `run_mobile_plugin`
  - Return responses to JavaScript
- **Code size**: ~50-100 lines total

#### Kotlin Layer (`AnkiDroidPlugin.kt`)

- **Role**: Business logic and Android integration
- **Responsibilities**:
  - All AnkiDroid API interactions
  - Error handling and recovery
  - Data validation
  - Complex operations (media handling, deck management)
- **Components**:
  - `AnkiDroidPlugin.kt` - Main plugin class with RPC dispatcher
  - `AnkiDroidHelper.kt` - Helper utilities (ported from Capacitor)
  - `models/` - Data classes for requests/responses

#### JavaScript/TypeScript Layer

- **Role**: Frontend API
- **Responsibilities**:
  - Type-safe API for consumers
  - Request/response transformation
  - Promise-based interface

### 4. Key Features to Implement

#### Core Functionality

- Card CRUD operations (create, read, update, delete)
- Deck management (create, list, find by name)
- Model/note type management
- Tag management

#### Advanced Features (from Capacitor implementation)

- **Media handling**: Images, audio, fonts
  - Support for URLs, data URIs, and file paths
  - FileProvider integration for sharing with AnkiDroid
- **Duplicate detection**: Prevent duplicate cards
- **Deck hierarchy**: Support for subdecks (e.g., "Japanese::Reading")
- **Card suspension**: Suspend/unsuspend cards
- **Card movement**: Move cards between decks
- **Permission management**: Handle AnkiDroid API permissions

#### Helper Utilities

- **Model persistence**: Remember model IDs even after renaming
- **Deck persistence**: Track deck IDs across renames
- **API availability checks**: Verify AnkiDroid is installed and accessible
- **Batch operations**: Efficient bulk card creation

### 5. Benefits of This Architecture

#### Development Benefits

- **Easier debugging**: Kotlin stack traces vs JNI crashes
- **Better IDE support**: Full Kotlin autocomplete and type checking
- **Faster iteration**: Changes only require Kotlin compilation
- **Unit testing**: Can test Kotlin logic independently

#### Maintenance Benefits

- **Clear separation of concerns**: Each layer has a specific role
- **Reduced complexity**: No JNI boilerplate
- **Better error messages**: Kotlin exceptions are more informative
- **Easier to extend**: Add new methods without touching Rust

#### Performance Benefits

- **Reduced serialization**: Single RPC call vs multiple JNI calls
- **Efficient data transfer**: JSON serialization is optimized
- **Caching opportunities**: Can cache in Kotlin layer

### 6. Migration Strategy

#### Phase 1: Setup Infrastructure

1. Create `AnkiDroidHelper.kt` with utility functions
2. Implement RPC dispatcher in `AnkiDroidPlugin.kt`
3. Create thin RPC proxy in `mobile.rs`

#### Phase 2: Port Core Features

1. Basic CRUD operations (create, list, update, delete)
2. Deck and model management
3. Permission handling

#### Phase 3: Port Advanced Features

1. Media handling (images, audio)
2. Duplicate detection
3. Card suspension and movement
4. Batch operations

#### Phase 4: Cleanup

1. Remove old JNI code from `mobile.rs`
2. Update tests
3. Update documentation

### 7. Data Flow Examples

#### Example: Create Card with Image

```
1. JS calls: createCard({ front, back, image, deck })
2. Rust forwards: execute_rpc("createCard", params)
3. Kotlin:
   - Validates input
   - Downloads/processes image via addMedia()
   - Finds or creates deck
   - Creates card via AnkiDroid API
   - Handles any errors
4. Returns: { success: true, noteId: 12345 }
```

#### Example: Batch Import

```
1. JS calls: batchImport({ cards: [...], options })
2. Rust forwards: execute_rpc("batchImport", params)
3. Kotlin:
   - Checks for duplicates
   - Processes media for all cards
   - Creates cards in transaction
   - Returns progress updates via channel
4. Returns: { success: true, imported: 95, skipped: 5 }
```

### 8. Error Handling Strategy

#### Kotlin Layer

- Try-catch blocks around all AnkiDroid API calls
- Specific error types for different failures
- Fallback strategies (e.g., use default deck if creation fails)

#### Response Format

```kotlin
sealed class ApiError {
    data class PermissionDenied(val message: String)
    data class AnkiDroidNotInstalled(val message: String)
    data class ModelNotFound(val modelName: String)
    data class DeckNotFound(val deckName: String)
    data class NetworkError(val url: String, val error: String)
    data class Unknown(val message: String)
}
```

### 9. Testing Strategy

#### Unit Tests

- Kotlin: Test helper functions independently
- Mock AnkiDroid API for testing business logic

#### Integration Tests

- Test RPC communication between layers
- Verify serialization/deserialization

#### E2E Tests

- Full flow from JavaScript to AnkiDroid
- Test on actual Android emulator

## Conclusion

This architecture provides a clean, maintainable, and extensible foundation for the Tauri AnkiDroid plugin. By leveraging Kotlin for all Android-specific logic and using Rust as a thin proxy, we achieve better error handling, easier debugging, and more maintainable code while preserving all the advanced features from the Capacitor implementation.
