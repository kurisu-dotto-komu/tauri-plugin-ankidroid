# Tauri AnkiDroid Plugin - Project Guidelines

## Core Principles
- **KISS**: Keep implementations simple and focused
- **YAGNI**: Don't add features until they're needed
- **Type Safety First**: Leverage Rust's type system and TypeScript for safety
- **Permission-Driven**: All AnkiDroid operations require explicit permissions

## Project Structure
```
tauri-plugin-ankidroid/
├── packages/                    # Monorepo packages
│   ├── tauri-plugin-ankidroid/ # Rust plugin core
│   ├── plugin-ankidroid-js/    # TypeScript bindings
│   └── demo-app/                # Test application
├── scripts/                     # Dev automation
├── .devcontainer/              # Linux dev environment
└── docs/                       # Documentation
```

## Development Environment
- **Container-based**: Always develop inside the devcontainer
- **Linux-only**: GUI Android emulator requires Linux + KVM
- **X11 forwarding**: Required for emulator GUI display
- **Prerequisite check**: Run `xhost +local:` on host before starting

## Code Standards

### Rust (Plugin Core)
- Maximum file length: 500 lines
- Maximum function length: 50 lines
- Use `rustfmt` and `clippy` with default settings
- All public APIs must have doc comments
- Use `Result<T, E>` for fallible operations
- Prefer `async` for I/O operations

### Kotlin (Android Module)
- Follow official Kotlin style guide
- Use `@Command` annotation for Tauri commands
- Handle null safety explicitly
- Maximum class length: 300 lines
- Use coroutines for async operations

### TypeScript (Bindings)
- Strict mode always enabled
- All exports must have type definitions
- Use JSDoc comments for public APIs
- Prefer async/await over callbacks
- Maximum file length: 300 lines

## Testing Requirements
- **Unit tests**: Required for all public APIs
- **Integration tests**: Required for plugin commands
- **Coverage target**: 80% for core functionality
- **Test location**: Tests live next to implementation files

### Test Commands
```bash
# Rust tests
cargo test --workspace

# TypeScript tests
pnpm test

# Kotlin/Android tests (JVM only)
cd packages/tauri-plugin-ankidroid/mobile/android
./gradlew test

# Full test suite
pnpm run test:all
```

## Build & Development Commands
```bash
# Install all dependencies
pnpm install

# Build all packages
pnpm build

# Start emulator (inside devcontainer)
bash scripts/emu-create-and-start.sh

# Install AnkiDroid
bash scripts/emu-install-ankidroid.sh

# Run demo app on Android
pnpm --filter demo-app android:dev

# Run demo app on desktop
pnpm --filter demo-app dev

# Lint all code
pnpm lint

# Format all code
pnpm format
```

## Android Emulator Management
- **AVD Name**: `tauri-ankidroid-test`
- **API Level**: 34 (Android 14)
- **Architecture**: x86_64 (for performance)
- **GPU**: Host mode preferred, swiftshader fallback
- **AnkiDroid Version**: 2.17+ (pinned in scripts)

## Permission Model
- All commands require explicit permissions in TOML
- Permissions are deny-by-default
- Each command has allow/deny variants
- App must declare capabilities in tauri.conf.json

### Example Permission
```toml
[[permission]]
identifier = "allow-add-note"
description = "Allows adding notes to AnkiDroid"
commands.allow = ["add_note"]
```

## Error Handling
- Never panic in production code
- Return descriptive error messages
- Log errors with appropriate levels
- Provide user-actionable error messages
- Use error enums for typed errors

## Git Workflow
- **Main branch**: `main` (production-ready)
- **Feature branches**: `feature/descriptive-name`
- **Commit style**: Conventional commits (feat, fix, docs, etc.)
- **PR requirement**: All tests must pass
- **Review requirement**: At least one approval

## Documentation Standards
- README must explain setup and usage
- All public APIs need doc comments
- Complex logic requires inline comments
- Keep CLAUDE.md and PRP.md updated
- Provide examples for key features

## Security Guidelines
- Never log sensitive information
- Validate all inputs from JavaScript
- Use Content Security Policy in demo app
- Sanitize HTML in note fields
- Rate-limit bulk operations

## Performance Targets
- Plugin initialization: < 100ms
- Command round-trip: < 50ms
- Emulator boot time: < 30s
- Build time (incremental): < 5s
- Test suite execution: < 60s

## Common Issues & Solutions

### Emulator Won't Start
- Check `/dev/kvm` exists and is accessible
- Verify DISPLAY environment variable
- Try `-gpu swiftshader` if GPU issues

### AnkiDroid API Not Working
- Ensure AnkiDroid 2.15+ is installed
- Check permissions in AndroidManifest
- Verify ContentProvider URI is correct

### Build Failures
- Clean build: `cargo clean && pnpm clean`
- Check Rust toolchain: `rustup update`
- Verify Android SDK: `sdkmanager --list`

## Key Libraries & Versions
```toml
# Rust
tauri = "2.0+"
serde = "1.0"
tokio = "1.0"

# Android
kotlin = "1.9+"
gradle = "8.0+"
android-sdk = "34"

# Node/TypeScript
node = "20.x"
pnpm = "8.x"
typescript = "5.0+"
vite = "5.0+"
```

## Behavioral Guidelines for AI Agents
- Always check existing code patterns before implementing
- Run tests after making changes
- Use the provided scripts for emulator management
- Follow the established project structure
- Ask for clarification if requirements are ambiguous
- Prioritize working code over perfect code
- Keep implementations minimal and focused

## Quality Checklist
Before committing:
- [ ] All tests pass
- [ ] Code is formatted (`pnpm format`)
- [ ] Linting passes (`pnpm lint`)
- [ ] Documentation is updated
- [ ] Permissions are properly configured
- [ ] Error cases are handled
- [ ] Performance targets are met