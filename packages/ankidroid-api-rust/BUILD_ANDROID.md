# Building for Android

This package requires the Android NDK to build for Android targets.

## Prerequisites

1. Install Android NDK (version 25 or later recommended)
2. Set the `NDK_HOME` environment variable to point to your NDK installation
3. Install Rust Android targets:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```

## Building

### Option 1: Using cargo with proper configuration

Ensure you have a `.cargo/config.toml` file in the project root with:

```toml
[target.aarch64-linux-android]
linker = "aarch64-linux-android30-clang"

[target.armv7-linux-androideabi]
linker = "armv7a-linux-androideabi30-clang"

[target.i686-linux-android]
linker = "i686-linux-android30-clang"

[target.x86_64-linux-android]
linker = "x86_64-linux-android30-clang"
```

Then build:
```bash
export PATH=$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
cargo build --target aarch64-linux-android
```

### Option 2: Using cargo-ndk (recommended)

Install cargo-ndk:
```bash
cargo install cargo-ndk
```

Build:
```bash
cargo ndk -t arm64-v8a build
```

## Available npm scripts

```bash
# Build for all Android architectures
npm run build:all-android

# Build for specific architectures
npm run build:android       # aarch64 (64-bit ARM)
npm run build:android-arm   # armv7 (32-bit ARM)
npm run build:android-x86   # x86 (32-bit Intel)
npm run build:android-x86_64 # x86_64 (64-bit Intel)
```

## Troubleshooting

### Linker errors

If you get linker errors like "file in wrong format", ensure:
1. NDK is properly installed
2. The correct linker is in your PATH
3. The `.cargo/config.toml` file points to the correct linker

### Missing NDK

If NDK is not installed, you can:
1. Install Android Studio and use SDK Manager to install NDK
2. Or download NDK directly from: https://developer.android.com/ndk/downloads

### Cross-compilation in CI

For CI environments, consider using Docker images with pre-installed Android toolchains:
- `rust-android-docker`
- Official Android Docker images with Rust installed