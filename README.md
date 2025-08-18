# Tauri Plugin Workspace with Android Emulator

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
6. **Develop** - Run `npm run android:dev` to test your Tauri app with the plugin

### Viewing the Android Emulator

The devcontainer includes a VNC desktop for viewing the Android emulator GUI. To access it:

1. **From VS Code**: Open the Ports panel and forward port 6080
2. **Open in browser**: Navigate to `http://localhost:6080`
3. **No password required**: The VNC server is configured with no password for convenience

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

- `npm run dev` - Start the demo app in development mode
- `npm run android:dev` - Start the demo app for Android development



