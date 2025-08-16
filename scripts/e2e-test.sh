#!/bin/bash
set -e

echo "🧪 E2E Integration Tests for AnkiDroid Plugin"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if emulator is running
print_info "Checking emulator status..."
if ! adb devices | grep -q "emulator-5555"; then
    print_error "Android emulator not running. Please start it first."
    exit 1
fi
print_status "Emulator is running"

# Check if AnkiDroid is installed
print_info "Checking AnkiDroid installation..."
if ! adb shell pm list packages | grep -q "com.ichi2.anki"; then
    print_error "AnkiDroid not installed. Installing..."
    bash "$(dirname "$0")/emulator-install-ankidroid.sh"
fi
print_status "AnkiDroid is installed"

# Setup AnkiDroid with test data
print_info "Setting up AnkiDroid test data..."

# Grant AnkiDroid permissions
adb shell pm grant com.ichi2.anki android.permission.WRITE_EXTERNAL_STORAGE || true
adb shell pm grant com.ichi2.anki android.permission.READ_EXTERNAL_STORAGE || true

# Start AnkiDroid to initialize database
print_info "Starting AnkiDroid to initialize..."
adb shell am start -n com.ichi2.anki/.DeckPicker
sleep 5

# Force close AnkiDroid
adb shell am force-stop com.ichi2.anki
sleep 2

# Create test cards using AnkiDroid's database
print_info "Creating test cards via AnkiDroid API..."

# Note: We'll use a simple approach - let AnkiDroid create its default deck
# and we'll test reading whatever data is available

# Install our test app
print_info "Installing test app..."
APK_PATH="/workspaces/tauri-plugin-ankidroid/packages/demo-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"
if [ ! -f "$APK_PATH" ]; then
    print_error "Test app not built. Please run 'pnpm tauri android build --debug' first."
    exit 1
fi

adb install -r "$APK_PATH"
print_status "Test app installed"

# Grant our app permissions
print_info "Granting app permissions..."
adb shell pm grant com.tauri.ankidroid.demo android.permission.QUERY_ALL_PACKAGES || true

# Start our test app
print_info "Starting test app..."
adb shell am start -n com.tauri.ankidroid.demo/.MainActivity
sleep 3

# Clear logs and prepare for testing
adb logcat -c
print_status "App started, ready for testing"

print_info "============================================="
print_info "E2E Test Environment Ready!"
print_info "============================================="
print_info "1. AnkiDroid is installed and initialized"
print_info "2. Test app is installed and running"
print_info "3. Permissions are granted"
print_info ""
print_info "Manual Test Steps:"
print_info "1. Tap 'Read AnkiDroid Cards' button in the app"
print_info "2. Check if it crashes or returns data"
print_info ""
print_info "To capture crash logs, run:"
print_info "  adb logcat | grep -E '(demo-app|tauri|ankidroid|FATAL|AndroidRuntime)'"
print_info ""
print_info "To run automated test, run:"
print_info "  bash scripts/e2e-test-automated.sh"