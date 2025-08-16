# PRP: Tauri AnkiDroid Plugin Bootstrap Implementation

## Goal

**Feature Goal**: Implement a Tauri v2 plugin that bridges Tauri apps with AnkiDroid API on Android, enabling direct flashcard management without external servers.

**Deliverable**: A working Tauri plugin with Rust core, Kotlin Android module, TypeScript bindings, and demo app showing round-trip "hello world" command execution.

**Success Definition**: `pnpm --filter demo-app android:dev` deploys to emulator, button tap shows string from plugin, all tests pass, AnkiDroid is installed and ready for future API integration.

## Context

```yaml
references:
  documentation:
    - name: "Tauri v2 Plugin Development"
      url: "https://v2.tauri.app/develop/plugins/"
      purpose: "Core plugin architecture and patterns"
    
    - name: "Tauri Mobile Plugin Development"
      url: "https://v2.tauri.app/develop/plugins/develop-mobile/"
      purpose: "Android/iOS specific implementation details"
    
    - name: "AnkiDroid API Documentation"
      url: "https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API"
      purpose: "ContentProvider API for note management"
    
    - name: "Tauri Plugin Permissions"
      url: "https://v2.tauri.app/learn/security/writing-plugin-permissions/"
      purpose: "Permission system implementation"
    
  codebase_patterns:
    - file: "PRPs/ai_docs/tauri_plugin_patterns.md"
      pattern: "Plugin initialization and command routing"
      usage: "Reference for exact Tauri plugin structure"
    
    - file: "PRPs/ai_docs/ankidroid_api_reference.md"
      pattern: "AnkiDroid ContentProvider integration"
      usage: "Reference for Android-specific AnkiDroid API calls"
    
    - file: "CLAUDE.md"
      pattern: "Project conventions and standards"
      usage: "Follow naming, testing, and structure guidelines"
    
  external_examples:
    - source: "GitHub - Official Tauri Plugins"
      url: "https://github.com/tauri-apps/plugins-workspace/tree/v2"
      relevance: "Reference implementations for mobile plugins"
    
    - source: "Tauri Plugin Biometric"
      url: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/biometric"
      relevance: "Android/iOS platform-specific implementation pattern"
    
  gotchas:
    - issue: "Android 11+ requires package visibility declaration"
      solution: "Add <queries> element in AndroidManifest.xml"
      reference: "PRPs/ai_docs/ankidroid_api_reference.md"
    
    - issue: "Commands not accessible from frontend"
      solution: "Define permissions in permissions/default.toml"
      reference: "PRPs/ai_docs/tauri_plugin_patterns.md"
    
    - issue: "Emulator GPU acceleration issues"
      solution: "Use -gpu swiftshader fallback in emu-create-and-start.sh"
      reference: "scripts/emu-create-and-start.sh"

project_specifics:
  naming_conventions:
    - type: "Plugin name"
      pattern: "tauri-plugin-ankidroid"
      example: "Rust crate and main directory"
    
    - type: "Command names"
      pattern: "snake_case in Rust, camelCase in JS"
      example: "add_note (Rust) → addNote (JS)"
    
    - type: "Permission identifiers"
      pattern: "allow-{command-name}"
      example: "allow-add-note, allow-get-decks"
  
  file_locations:
    - purpose: "Rust plugin core"
      path: "packages/tauri-plugin-ankidroid/src/"
      existing_examples: "lib.rs, commands.rs, desktop.rs, mobile.rs"
    
    - purpose: "Android Kotlin module"
      path: "packages/tauri-plugin-ankidroid/android/"
      existing_examples: "src/main/java/app/tauri/ankidroid/"
    
    - purpose: "TypeScript bindings"
      path: "packages/plugin-ankidroid-js/src/"
      existing_examples: "index.ts with exported functions"
    
    - purpose: "Demo application"
      path: "packages/demo-app/"
      existing_examples: "Tauri app with plugin integration"
  
  dependencies:
    - name: "tauri"
      version: "2.0+"
      usage: "Core framework for plugin"
    
    - name: "@tauri-apps/api"
      version: "^2.0.0"
      usage: "JavaScript API for invoke"
    
    - name: "com.github.ankidroid:Anki-Android"
      version: "api-v1.1.0"
      usage: "AnkiDroid API for Android module"

validation_commands:
  lint: "pnpm lint && cargo clippy --all-targets"
  test: "cargo test --workspace && pnpm test"
  build: "pnpm build"
  verify: "pnpm --filter demo-app android:dev"
```

## Implementation Tasks

### Task 1: Initialize Workspace Structure
**Objective**: Create monorepo structure with pnpm and Cargo workspaces
**Location**: Repository root
**Pattern Reference**: Workspace configuration from research
**Key Implementation Points**:
- Create `/pnpm-workspace.yaml` with packages configuration
- Create root `/package.json` with workspace scripts
- Create root `/Cargo.toml` with workspace members
- Initialize `.npmrc` with workspace protocol settings

**Validation Gate**: 
```bash
pnpm install && cargo check --workspace
```

### Task 2: Implement Rust Plugin Core
**Dependencies**: Task 1
**Objective**: Create Tauri plugin with hello command
**Location**: `packages/tauri-plugin-ankidroid/`
**Pattern Reference**: PRPs/ai_docs/tauri_plugin_patterns.md - Plugin Entry Point
**Key Implementation Points**:
- Create `src/lib.rs` with plugin initialization using Builder::new("ankidroid")
- Create `src/commands.rs` with `#[tauri::command] async fn hello(name: String)`
- Create `src/desktop.rs` and `src/mobile.rs` for platform implementations
- Create `build.rs` with COMMANDS array containing ["hello"]
- Create `Cargo.toml` with tauri and serde dependencies

**Validation Gate**: 
```bash
cd packages/tauri-plugin-ankidroid && cargo build --release
```

### Task 3: Configure Permissions
**Dependencies**: Task 2
**Objective**: Define plugin permissions for commands
**Location**: `packages/tauri-plugin-ankidroid/permissions/`
**Pattern Reference**: PRPs/ai_docs/tauri_plugin_patterns.md - Permission Configuration
**Key Implementation Points**:
- Create `default.toml` with allow-hello permission
- Define default permission set including allow-hello
- Add schema reference "$schema" = "../schemas/schema.json"

**Validation Gate**: 
```bash
# Build should generate permission artifacts
cd packages/tauri-plugin-ankidroid && cargo build
```

### Task 4: Create Android Kotlin Module
**Dependencies**: Task 2
**Objective**: Implement Android-specific plugin functionality
**Location**: `packages/tauri-plugin-ankidroid/android/`
**Pattern Reference**: PRPs/ai_docs/tauri_plugin_patterns.md - Android Module Pattern
**Key Implementation Points**:
- Create `build.gradle.kts` with Android library configuration
- Create `src/main/java/app/tauri/ankidroid/AnkiDroidPlugin.kt`
- Implement `@TauriPlugin` class with `@Command fun hello(invoke: Invoke)`
- Add project(":tauri-android") dependency in gradle
- Configure namespace as "app.tauri.ankidroid"

**Validation Gate**: 
```bash
cd packages/tauri-plugin-ankidroid/android && ./gradlew build
```

### Task 5: Implement TypeScript Bindings
**Dependencies**: Task 2
**Objective**: Create JavaScript API for plugin commands
**Location**: `packages/plugin-ankidroid-js/`
**Pattern Reference**: PRPs/ai_docs/tauri_plugin_patterns.md - JavaScript Bindings
**Key Implementation Points**:
- Create `src/index.ts` with exported hello function
- Use invoke with "plugin:ankidroid|hello" identifier
- Create `package.json` with @tauri-apps/api dependency
- Configure rollup.config.js for module bundling
- Set up TypeScript configuration with strict mode

**Validation Gate**: 
```bash
cd packages/plugin-ankidroid-js && pnpm build
```

### Task 6: Create Demo Application
**Dependencies**: Tasks 2, 5
**Objective**: Tauri app demonstrating plugin usage
**Location**: `packages/demo-app/`
**Pattern Reference**: Standard Tauri v2 app structure
**Key Implementation Points**:
- Initialize Tauri app with `npx create-tauri-app`
- Add plugin dependency in `src-tauri/Cargo.toml`
- Register plugin in `src-tauri/src/lib.rs` with `.plugin(tauri_plugin_ankidroid::init())`
- Create UI button that calls hello("World") command
- Configure tauri.conf.json with plugin and permissions

**Validation Gate**: 
```bash
pnpm --filter demo-app dev  # Desktop test
```

### Task 7: Configure Android Build
**Dependencies**: Tasks 4, 6
**Objective**: Enable Android deployment for demo app
**Location**: `packages/demo-app/src-tauri/`
**Pattern Reference**: Tauri mobile configuration
**Key Implementation Points**:
- Run `pnpm --filter demo-app tauri android init`
- Update gen/android/app/build.gradle.kts with plugin module
- Add AnkiDroid queries to AndroidManifest.xml
- Configure Android SDK paths in local.properties

**Validation Gate**: 
```bash
pnpm --filter demo-app tauri android build --apk
```

### Task 8: Implement Tests
**Dependencies**: Tasks 2, 4, 5
**Objective**: Add unit tests for all components
**Location**: Various test directories
**Pattern Reference**: CLAUDE.md testing requirements
**Key Implementation Points**:
- Rust unit tests in `src/` with `#[cfg(test)]` modules
- Kotlin tests with Robolectric in `android/src/test/`
- TypeScript tests with vitest in `plugin-ankidroid-js/`
- Test hello command returns expected format

**Validation Gate**: 
```bash
cargo test --workspace && pnpm test
```

### Task 9: Emulator Integration
**Dependencies**: Task 7
**Objective**: Deploy and test on Android emulator
**Location**: Repository root and scripts
**Pattern Reference**: scripts/emu-create-and-start.sh, scripts/emu-install-ankidroid.sh
**Key Implementation Points**:
- Start emulator with existing script
- Install AnkiDroid APK with existing script
- Deploy demo app with `pnpm --filter demo-app android:dev`
- Verify button tap shows plugin response

**Validation Gate**: 
```bash
bash scripts/emu-create-and-start.sh
bash scripts/emu-install-ankidroid.sh
pnpm --filter demo-app android:dev
```

## Error Handling

### Expected Errors
```yaml
- error: "error: linking with `cc` failed"
  cause: "Android NDK not configured"
  solution: "Set ANDROID_NDK_HOME environment variable"
  
- error: "Module with the Main dispatcher not found"
  cause: "Missing kotlinx-coroutines dependency"
  solution: "Add implementation 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.0'"
  
- error: "Command not found: plugin:ankidroid|hello"
  cause: "Plugin not registered or permissions missing"
  solution: "Ensure plugin registered in lib.rs and permission in tauri.conf.json"
  
- error: "INSTALL_FAILED_NO_MATCHING_ABIS"
  cause: "Architecture mismatch with emulator"
  solution: "Use x86_64 emulator or build for arm64"
```

## Testing Strategy

### Unit Tests
**Location**: `packages/tauri-plugin-ankidroid/src/tests.rs`
**Pattern**: Standard Rust testing with `#[test]` attribute
**Coverage Requirements**:
- hello command returns "Hello, {name} from AnkiDroid plugin!"
- Command serialization/deserialization works
- Error cases handled properly

### Integration Tests
**Location**: `packages/demo-app/tests/`
**Setup Requirements**: Running emulator with AnkiDroid
**Key Scenarios**:
- Hello command round-trip through all layers
- Permission denial handled gracefully
- AnkiDroid installation check works

## Final Validation Checklist

### Functional Requirements
- [ ] `hello("World")` returns expected string format
- [ ] Plugin loads successfully in Tauri app
- [ ] Android module responds to commands
- [ ] TypeScript bindings export correct functions

### Code Quality
- [ ] All tests pass: `cargo test --workspace && pnpm test`
- [ ] Linting passes: `cargo clippy --all-targets && pnpm lint`
- [ ] Build succeeds: `pnpm build`
- [ ] Type checking passes: `pnpm --filter plugin-ankidroid-js tsc --noEmit`

### Project Conventions
- [ ] Follows naming pattern: tauri-plugin-ankidroid
- [ ] Located in correct directory: packages/
- [ ] Includes required documentation: README.md in each package
- [ ] Permissions configured: permissions/default.toml exists

### Android Integration
- [ ] Emulator boots with GUI: `bash scripts/emu-create-and-start.sh`
- [ ] AnkiDroid installs: `bash scripts/emu-install-ankidroid.sh`
- [ ] Demo app deploys: `pnpm --filter demo-app android:dev`
- [ ] Button tap shows plugin response in app

## Additional Notes

### Development Workflow
1. Always develop inside devcontainer for consistency
2. Run `xhost +local:` on host before starting emulator
3. Use scripts in scripts/ directory for emulator management
4. Follow incremental build approach: desktop first, then Android

### Future Enhancements (Not for Bootstrap)
- Implement actual AnkiDroid API calls (add_note, get_decks)
- Add permission scoping for specific deck access
- Implement sync functionality
- Add desktop AnkiConnect fallback
- Create comprehensive API documentation

### Critical Files to Reference
- `PRPs/ai_docs/tauri_plugin_patterns.md` - Exact implementation patterns
- `PRPs/ai_docs/ankidroid_api_reference.md` - AnkiDroid API details
- `CLAUDE.md` - Project conventions and standards
- `PLAN.md` - Original requirements and milestones

---

**PRP Confidence Score**: 9/10
**Estimated Implementation Time**: 4-6 hours
**Risk Areas**: Android build configuration, emulator GPU settings