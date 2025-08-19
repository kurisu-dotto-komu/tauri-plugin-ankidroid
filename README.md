# Tauri Plugin Workspace with Android Emulator

## Package Overview

This workspace contains multiple packages for building a Tauri plugin that interfaces with AnkiDroid:

- **[`tauri-plugin-ankidroid-android`](./packages/tauri-plugin-ankidroid-android)** - The core Rust plugin providing native Android integration with AnkiDroid's content provider API
- **[`tauri-plugin-ankidroid-js`](./packages/tauri-plugin-ankidroid-js)** - JavaScript/TypeScript bindings for the plugin, providing a clean API for Tauri apps
- **[`tauri-plugin-ankidroid-e2e-test-app`](./packages/tauri-plugin-ankidroid-e2e-test-app)** - Example Tauri application demonstrating plugin usage and testing capabilities

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



