# Tauri Plugin Workspace with Android Emulator

## Package Overview

This workspace contains multiple packages for building a Tauri plugin that interfaces with AnkiDroid:

### Core Packages

- **[`ankidroid-api-rust`](./packages/ankidroid-api-rust)** - Low-level Rust library providing complete AnkiDroid API bindings
  - Direct JNI integration with Android ContentResolver
  - Type-safe Rust API for all AnkiDroid operations
  - Support for notes, decks, models, and media management
  - Comprehensive error handling and validation
  - Can be used independently of Tauri for any Rust Android project

- **[`tauri-plugin-ankidroid-android`](./packages/tauri-plugin-ankidroid-android)** - Tauri plugin wrapper around ankidroid-api-rust
  - Exposes AnkiDroid functionality to Tauri applications
  - Handles Tauri-specific integration and commands
  - Manages permissions and Android context
  - Provides simplified API for common operations

### JavaScript/TypeScript Bindings

- **[`tauri-plugin-ankidroid-js`](./packages/tauri-plugin-ankidroid-js)** - JavaScript/TypeScript bindings for the plugin
  - Clean, promise-based API for frontend applications
  - Full TypeScript support with type definitions
  - Handles serialization/deserialization
  - Provides both high-level and low-level API access

### Test Application

- **[`tauri-plugin-ankidroid-e2e-test-app`](./packages/tauri-plugin-ankidroid-e2e-test-app)** - Example Tauri application
  - Demonstrates all plugin capabilities
  - Includes comprehensive E2E tests
  - Shows best practices for integration
  - Provides UI for testing CRUD operations

## Architecture

The plugin follows a layered architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                   Tauri Application (Frontend)              │
│                     React/Vue/Svelte/etc                    │
└──────────────────────┬──────────────────────────────────────┘
                       │ JavaScript API calls
┌──────────────────────▼──────────────────────────────────────┐
│              tauri-plugin-ankidroid-js                      │
│                 TypeScript bindings                         │
└──────────────────────┬──────────────────────────────────────┘
                       │ Tauri IPC
┌──────────────────────▼──────────────────────────────────────┐
│           tauri-plugin-ankidroid-android                    │
│                  Tauri Plugin (Rust)                        │
└──────────────────────┬──────────────────────────────────────┘
                       │ Uses
┌──────────────────────▼──────────────────────────────────────┐
│               ankidroid-api-rust                            │
│            Low-level AnkiDroid API (Rust)                   │
└──────────────────────┬──────────────────────────────────────┘
                       │ JNI + ContentResolver
┌──────────────────────▼──────────────────────────────────────┐
│                    AnkiDroid App                            │
│                  (External Android App)                     │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Separation of Concerns**: The low-level API (`ankidroid-api-rust`) is separate from the Tauri plugin, allowing it to be used in other Rust Android projects.

2. **ContentResolver Only**: We use Android's ContentResolver API exclusively for communication with AnkiDroid. We don't attempt to instantiate AnkiDroid's internal classes (like `AddContentApi`) since they're not available in external apps.

3. **Permission-Based Access**: The app requires `com.ichi2.anki.permission.READ_WRITE_DATABASE` permission to access AnkiDroid's data.

4. **Type Safety**: Full type safety from Rust through TypeScript to the frontend application.

## Development Environment

We strongly recommend using the provided devcontainer for development. It includes everything you need:

- **Rust 1.89** - For building the Tauri plugin
- **Node.js 22** - For the JavaScript/TypeScript toolchain
- **Android SDK** - Platform 35 with build tools
- **Android Emulator** - Pre-configured with x86_64 system images
- **Java 17** - Required for Android development
- **VNC Desktop** - For viewing the Android emulator GUI

**Note:** The initial devcontainer build will take some time (10-20 minutes) as it downloads and installs all dependencies, including the Android SDK and system images. Subsequent rebuilds will be much faster.

## Development Workflow

1. **Start the devcontainer** - Open the project in VS Code with the Dev Containers extension
2. **Create the AVD** - Run `npm run emu:create` (only needed once)
3. **Start the emulator** - Run `npm run emu:start`
4. **View the emulator** - Open `http://localhost:6080` in your browser
5. **Install AnkiDroid** - Run `npm run emu:install-anki`
6. **Develop** - Run `npm run dev` to start development with hot reload and DevTools

### Hot Reload Development

The `npm run dev` command provides a complete development environment with:

- **Hot Module Replacement (HMR)** - React changes update instantly without losing state
- **Rust Auto-rebuild** - Plugin code rebuilds automatically on changes
- **Chrome DevTools** - Full debugging capabilities for your app
- **Network Inspection** - Monitor API calls and responses

### Using Chrome DevTools

When running in the devcontainer, Chrome DevTools are automatically configured. To access them from your host machine:

1. **Start development**: Run `npm run dev` in the devcontainer terminal
2. **Open Chrome/Edge** on your host machine
3. **Navigate to** `chrome://inspect` (or `edge://inspect` for Edge)
4. **Find your app** under "Remote Target" section
5. **Click "inspect"** to open DevTools

The devcontainer automatically forwards these ports:
- **5173** - Vite dev server (frontend with HMR)
- **6080** - VNC desktop for viewing emulator
- **9222** - Chrome DevTools Protocol
- **9229** - Node.js debugging (if needed)

### Viewing the Android Emulator

The devcontainer includes a VNC desktop for viewing the Android emulator GUI:

1. **Open in browser**: Navigate to `http://localhost:6080`
2. **No password required**: The VNC server is configured with no password for convenience
3. **VS Code will auto-forward** the port when you start the emulator

### Available NPM Scripts

#### Android Emulator Management

- `npm run emu:create` - Create a Pixel 7 AVD (Android Virtual Device)
- `npm run emu:start` - Start the Android emulator
- `npm run emu:stop` - Stop the running emulator
- `npm run emu:install-anki` - Install AnkiDroid on the emulator

#### Build & Test

- `npm run build` - Build all packages in parallel
- `npm run test` - Run JavaScript/TypeScript tests
- `npm run test:all` - Run both Cargo and JS tests
- `npm run lint` - Lint all packages
- `npm run format` - Format code in all packages
- `npm run clean` - Clean build artifacts

#### Development

- `npm run dev` - Start development with hot reload and Chrome DevTools
- `npm run dev:build` - Manual full rebuild and deploy (when hot reload isn't sufficient)
- `npm run android:dev` - Lower-level Android development command
- `npm run android:deploy` - Deploy release build to emulator/device

## Testing

This project includes multiple testing layers to ensure reliability and functionality. Here's a comprehensive guide to understanding and using the different testing options:

### Testing Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Pyramid                          │
├─────────────────────────────────────────────────────────────┤
│  E2E Tests (Appium + WebDriverIO)                          │
│  ├─ Real Android emulator testing                          │
│  ├─ Full app + plugin integration                          │
│  └─ User interaction simulation                            │
├─────────────────────────────────────────────────────────────┤
│  Integration Tests                                          │
│  ├─ Rust plugin compilation                                │
│  ├─ Frontend app building                                  │
│  └─ API contract validation                               │
├─────────────────────────────────────────────────────────────┤
│  Unit Tests                                                │
│  ├─ Rust function testing                                 │
│  ├─ JavaScript/TypeScript testing                         │
│  └─ Component testing                                     │
└─────────────────────────────────────────────────────────────┘
```

### 1. Manual Testing (Interactive Development)

**Best for**: Daily development, debugging, visual verification

**How to run**:
```bash
# Start the full development environment
npm run dev

# Or manually deploy to emulator
npm run android:deploy
```

**What it does**:
- Builds the Tauri app with your plugin
- Deploys to Android emulator
- Provides hot reload for frontend changes
- Opens in browser at `http://localhost:6080` (VNC to see emulator)
- You can interact with the app manually to test features

**When to use**: When you want to see the actual UI and test user flows

### 2. End-to-End (E2E) Tests

**Best for**: Automated validation of complete user workflows

**Technologies used**:
- **Appium**: Mobile automation framework (like Selenium for mobile)
- **WebDriverIO**: Test runner and browser/app automation library
- **Mocha**: JavaScript test framework for writing test cases
- **Android UIAutomator2**: Driver for Android app automation

**How to run**:
```bash
# Run all E2E tests
npm run test:e2e

# Run specific CRUD tests
npm run e2e

# Run with manual Appium server (if auto-start fails)
# Terminal 1:
appium server --address 127.0.0.1 --port 4723 --log-level info
# Terminal 2:
npm run e2e
```

**What it tests**:
- ✅ **CREATE**: Card creation with front/back/deck/tags
- ✅ **READ**: Retrieving and displaying cards
- ✅ **UPDATE**: Modifying existing cards
- ✅ **DELETE**: Removing cards
- ✅ **BULK**: Multiple card operations
- ✅ **EDGE CASES**: Special characters, long text
- ✅ **PERFORMANCE**: Rapid operations, stress testing

**Test Flow**:
1. Appium starts and connects to Android emulator
2. Launches your app (`com.tauri.ankidroid.demo`)
3. Simulates user interactions (taps, typing, scrolling)
4. Verifies expected UI changes and data
5. Generates test report

**When E2E tests fail**:
- Check emulator is running: `adb devices`
- Check app is installed: `adb shell pm list packages | grep tauri`
- Check Appium is accessible: `curl http://127.0.0.1:4723/status`
- View emulator at `http://localhost:6080` to see what's happening

### 3. Integration Tests

**Best for**: Verifying that different parts work together

**How to run**:
```bash
# Test Rust compilation
cargo check --workspace

# Test frontend building  
npm run build

# Test full integration
npm run test:all
```

**What it tests**:
- Plugin compiles correctly
- Frontend builds successfully
- Dependencies resolve properly
- API contracts are maintained

### 4. Unit Tests

**Best for**: Testing individual functions and components

**How to run**:
```bash
# Rust unit tests
cargo test

# JavaScript/TypeScript tests
npm test

# All tests together
npm run test:all
```

### Understanding Test Results

#### E2E Test Output Example:
```
✅ CREATE: Cards can be created with all fields
✅ READ: Created cards can be retrieved and displayed  
✅ UPDATE: Card information can be modified
❌ DELETE: Button selector not found (UI issue, not plugin issue)
✅ BULK: Multiple cards handled efficiently
✅ EDGE CASES: Special characters and long text handled
✅ PERFORMANCE: App remains stable under stress

9/10 tests passing (90% success rate)
```

**What this means**:
- Your plugin functionality is working correctly
- 1 test failed due to UI selector issue (not a critical failure)
- Overall system is stable and functional

### Testing Workflow for Development

#### During Feature Development:
1. **Write code** → **Manual testing** (`npm run dev`)
2. **Fix issues** → **Unit tests** (`cargo test`)
3. **Ready for review** → **E2E tests** (`npm run e2e`)

#### Before Committing:
```bash
# Quick validation
npm run lint && npm run test:all

# Full validation including E2E
npm run e2e
```

### Troubleshooting Common Test Issues

#### "No emulator found" or "Connection refused"
```bash
# Check emulator status
adb devices
# Should show: emulator-5554    device

# If not running, start it
npm run emu:start

# Wait 30 seconds, then check again
adb devices
```

#### "App not installed" 
```bash
# Check if app is installed
adb shell pm list packages | grep tauri

# If not found, build and install
npm run android:deploy
```

#### "Appium connection failed"
```bash
# Check Appium is running
curl http://127.0.0.1:4723/status

# If not, start manually
appium server --address 127.0.0.1 --port 4723 --log-level info
```

#### "Frontend shows network errors"
This usually means the app isn't properly built/deployed:
```bash
# Force rebuild and redeploy
npm run android:deploy

# Check app launches correctly
adb shell am start -n com.tauri.ankidroid.demo/.MainActivity
```

### Test Configuration Files

- **`wdio.android.conf.js`** - WebDriverIO configuration for Android testing
- **`tests/e2e/`** - E2E test files (867 lines of comprehensive tests)
- **`Cargo.toml`** - Rust test configuration
- **`package.json`** - JavaScript test scripts
- **`scripts/grant-ankidroid-permission.sh`** - Permission setup for testing

### Best Practices

1. **Always start with manual testing** to see what's actually happening
2. **Use E2E tests for regression testing** of critical user flows  
3. **Run unit tests frequently** during development
4. **Check the emulator GUI** at `http://localhost:6080` when tests fail
5. **Look at test logs** to understand what the automation is trying to do

## Using ankidroid-api-rust Independently

The `ankidroid-api-rust` package can be used independently in any Rust Android project:

### Add to your Cargo.toml:
```toml
[dependencies]
ankidroid-api-rust = { path = "../path/to/ankidroid-api-rust" }
```

### Basic Usage:
```rust
use ankidroid_api_rust::AnkiDroidApi;
use jni::objects::JObject;
use jni::JNIEnv;

fn use_ankidroid(env: JNIEnv, context: JObject) -> Result<(), Box<dyn std::error::Error>> {
    // Check if AnkiDroid is available
    if !AnkiDroidApi::is_available(env, &context)? {
        return Err("AnkiDroid not installed".into());
    }
    
    // Create API instance
    let mut api = AnkiDroidApi::try_new(env, &context)?;
    
    // Get deck list
    let decks = api.get_deck_list()?;
    println!("Found {} decks", decks.len());
    
    // Add a note
    let note_id = api.add_note(
        model_id,
        deck_id,
        &["Question", "Answer"],
        Some(&["tag1", "tag2"])
    )?;
    
    Ok(())
}
```

### Features:
- ✅ Full AnkiDroid API coverage
- ✅ Type-safe Rust bindings
- ✅ Comprehensive error handling
- ✅ Support for notes, decks, models, media
- ✅ Direct ContentResolver access
- ✅ No Tauri dependency required

See the [ankidroid-api-rust README](./packages/ankidroid-api-rust/README.md) for detailed API documentation.


