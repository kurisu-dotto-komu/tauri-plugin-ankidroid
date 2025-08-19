#!/bin/bash

# Development script with hot reload and Chrome DevTools support
set -e

echo "🚀 Starting Tauri Android development with hot reload and DevTools..."

# Ensure emulator is running
echo "📱 Checking Android emulator..."
if ! adb devices | grep -q "emulator.*device"; then
    echo "Starting emulator..."
    bash scripts/emu-start.sh &
    echo "Waiting for emulator to be ready..."
    adb wait-for-device
    sleep 10
fi

# Set NDK_HOME for Android builds
export NDK_HOME=/usr/local/lib/android/ndk/25.2.9519653

# Enable Chrome DevTools debugging
echo "🔧 Setting up Chrome DevTools..."
# Forward the Chrome DevTools port
adb forward tcp:9222 localabstract:chrome_devtools_remote || true
# additional debug ports
# adb forward tcp:9229 tcp:9229 || true  # Node.js inspector if needed

# Navigate to test app directory
cd packages/tauri-plugin-ankidroid-e2e-test-app

# Start dev server with DevTools integration
echo ""
echo "🔥 Starting hot reload dev server with DevTools..."
echo ""
echo "📍 Access points when running in devcontainer:"
echo "   Frontend Dev Server: http://localhost:5173"
echo "   Chrome DevTools: chrome://inspect/#devices"
echo "   Direct DevTools: http://localhost:9222"
echo ""
echo "🔄 Features enabled:"
echo "   ✅ Hot Module Replacement (HMR) for React"
echo "   ✅ Auto-rebuild for Rust changes"
echo "   ✅ Chrome DevTools inspection"
echo "   ✅ Network & Console debugging"
echo ""
echo "💡 To inspect the app:"
echo "   1. Open Chrome/Edge on your host machine"
echo "   2. Navigate to chrome://inspect"
echo "   3. Your app should appear under 'Remote Target'"
echo ""

# Run with --open flag to auto-open DevTools
npx tauri android dev --open