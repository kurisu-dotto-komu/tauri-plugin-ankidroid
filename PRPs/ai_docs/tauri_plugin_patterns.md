# Tauri v2 Plugin Implementation Patterns

## Critical Implementation Reference

This document contains essential Tauri v2 plugin patterns extracted from official documentation and real-world implementations. Reference this document for exact implementation patterns.

## Plugin Structure Requirements

### Mandatory Directory Structure
```
tauri-plugin-ankidroid/
├── src/
│   ├── lib.rs         # Plugin entry point
│   ├── commands.rs    # Command definitions
│   ├── desktop.rs     # Desktop implementation
│   └── mobile.rs      # Mobile implementation
├── permissions/       # Permission definitions
│   └── default.toml   # Default permissions
├── android/           # Android Kotlin module
├── guest-js/         # TypeScript bindings
├── build.rs          # Build script
└── Cargo.toml        # Rust configuration
```

## Core Implementation Patterns

### 1. Plugin Entry Point (src/lib.rs)
```rust
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
mod desktop;
mod mobile;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ankidroid")
        .invoke_handler(tauri::generate_handler![
            commands::hello,
            commands::add_note,
            commands::get_decks,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            {
                let handle = mobile::init(app, api)?;
                app.manage(handle);
            }
            #[cfg(desktop)]
            {
                let handle = desktop::init(app, api)?;
                app.manage(handle);
            }
            Ok(())
        })
        .build()
}
```

### 2. Command Pattern (src/commands.rs)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AddNoteRequest {
    pub front: String,
    pub back: String,
    pub deck: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddNoteResponse {
    pub note_id: i64,
    pub success: bool,
}

#[tauri::command]
pub async fn add_note(request: AddNoteRequest) -> Result<AddNoteResponse, String> {
    #[cfg(mobile)]
    {
        mobile::add_note(request).await
    }
    #[cfg(desktop)]
    {
        desktop::add_note(request).await
    }
}
```

### 3. Build Script (build.rs)
```rust
const COMMANDS: &[&str] = &[
    "hello",
    "add_note",
    "get_decks",
    "sync_collection",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
```

## Android Module Pattern

### Plugin Class (android/src/main/java/.../AnkiDroidPlugin.kt)
```kotlin
package app.tauri.ankidroid

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import app.tauri.plugin.JSObject

@TauriPlugin
class AnkiDroidPlugin(private val activity: Activity): Plugin(activity) {
    
    @Command
    fun hello(invoke: Invoke) {
        val args = invoke.parseArgs(HelloArgs::class.java)
        val ret = JSObject()
        ret.put("value", "Hello, ${args.name} from Android!")
        invoke.resolve(ret)
    }
    
    @Command
    fun addNote(invoke: Invoke) {
        val args = invoke.parseArgs(AddNoteArgs::class.java)
        
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val noteId = performAddNote(args)
                val ret = JSObject()
                ret.put("noteId", noteId)
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject("Failed to add note: ${e.message}")
            }
        }
    }
}

data class HelloArgs(val name: String)
data class AddNoteArgs(
    val front: String,
    val back: String,
    val deck: String?
)
```

## Permission Configuration

### permissions/default.toml
```toml
"$schema" = "../schemas/schema.json"

[[permission]]
identifier = "allow-hello"
description = "Allows the hello command"
commands.allow = ["hello"]

[[permission]]
identifier = "allow-add-note"
description = "Allows adding notes to AnkiDroid"
commands.allow = ["add_note"]

[[permission]]
identifier = "allow-get-decks"
description = "Allows querying available decks"
commands.allow = ["get_decks"]

[[permission]]
identifier = "default"
description = "Default permissions for the plugin"
permissions = ["allow-hello"]
```

## JavaScript/TypeScript Bindings

### guest-js/index.ts
```typescript
import { invoke } from '@tauri-apps/api/core'

export interface AddNoteRequest {
    front: string
    back: string
    deck?: string
}

export interface AddNoteResponse {
    noteId: number
    success: boolean
}

export async function hello(name: string): Promise<string> {
    return await invoke('plugin:ankidroid|hello', { name })
}

export async function addNote(request: AddNoteRequest): Promise<AddNoteResponse> {
    return await invoke('plugin:ankidroid|add_note', request)
}

export async function getDecks(): Promise<string[]> {
    return await invoke('plugin:ankidroid|get_decks')
}
```

## Package Configuration

### Cargo.toml
```toml
[package]
name = "tauri-plugin-ankidroid"
version = "0.1.0"
edition = "2021"
rust-version = "1.77.2"

[build-dependencies]
tauri-plugin = { version = "2.0", features = ["build"] }

[dependencies]
tauri = { version = "2.0" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
log = "0.4"

[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
```

### package.json (guest-js)
```json
{
  "name": "tauri-plugin-ankidroid-api",
  "version": "0.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  }
}
```

## Critical Integration Points

### 1. Command Naming Convention
- Rust function: `add_note`
- JavaScript invoke: `plugin:ankidroid|add_note`
- Permission: `allow-add-note`

### 2. Platform-Specific Code
```rust
#[cfg(mobile)]
use crate::mobile;

#[cfg(desktop)]  
use crate::desktop;
```

### 3. Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("AnkiDroid not installed")]
    NotInstalled,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}
```

## Testing Requirements

1. **Rust Unit Tests**: In `src/` directory
2. **Android Tests**: Using Robolectric
3. **Integration Tests**: Full round-trip testing
4. **Permission Tests**: Verify allow/deny behavior

## Build Commands

```bash
# Build plugin
cargo build --release

# Build TypeScript bindings
cd guest-js && npm run build

# Build Android module
cd android && ./gradlew build

# Test everything
cargo test --workspace
```