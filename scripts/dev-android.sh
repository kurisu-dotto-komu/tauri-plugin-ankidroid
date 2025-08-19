#!/bin/bash

# Script to run the app in dev mode on Android emulator

# Ensure emulator is running
echo "Checking if emulator is running..."
if ! adb devices | grep -q "emulator.*device"; then
    echo "Starting emulator..."
    bash scripts/emu-start.sh || exit 1
fi

# Build the web app only
echo "Building web app..."
cd packages/tauri-plugin-ankidroid-e2e-test-app
npm run build || exit 1

# Set NDK_HOME
export NDK_HOME=/usr/local/lib/android/ndk/25.2.9519653

# Build Android app with Tauri in debug mode
echo "Building Android app..."
npx tauri android build --debug || exit 1

# Install the app
echo "Installing app on emulator..."
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk || exit 1

# Launch the app
echo "Launching app..."
adb shell am force-stop com.tauri.ankidroid.demo
adb shell am start -n com.tauri.ankidroid.demo/.MainActivity

echo "App launched successfully! Connect via VNC to view the emulator (display :1, port 5901)"