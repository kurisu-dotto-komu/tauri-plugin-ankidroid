#!/bin/bash

# Script to install AnkiDroid on the running emulator

# AnkiDroid APK details
ANKIDROID_VERSION="2.22.3"
ANKIDROID_APK_URL="https://github.com/ankidroid/Anki-Android/releases/download/v${ANKIDROID_VERSION}/AnkiDroid-${ANKIDROID_VERSION}-full-universal.apk"
APK_DIR="third-party-apks"
APK_FILE="${APK_DIR}/AnkiDroid-${ANKIDROID_VERSION}.apk"

# Setup environment - SDK already installed by devcontainer
export ANDROID_HOME=${ANDROID_HOME:-$ANDROID_SDK_ROOT}
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator

echo "Checking if emulator is running..."
if ! adb devices | grep -q "emulator.*device"; then
    echo "Error: No emulator found. Please start the emulator first with 'npm run emu:start'"
    exit 1
fi

# Check if AnkiDroid is already installed
if adb shell pm list packages | grep -q "com.ichi2.anki"; then
    echo "AnkiDroid is already installed"
    read -p "Do you want to reinstall? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
    echo "Uninstalling existing AnkiDroid..."
    adb uninstall com.ichi2.anki
fi

# Create APK directory if it doesn't exist
mkdir -p "$APK_DIR"

# Download APK if not already present
if [ -f "$APK_FILE" ]; then
    echo "Using existing AnkiDroid APK v${ANKIDROID_VERSION}..."
else
    echo "Downloading AnkiDroid v${ANKIDROID_VERSION} from GitHub..."
    curl -L -o "$APK_FILE" "$ANKIDROID_APK_URL"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to download AnkiDroid APK"
        exit 1
    fi
    echo "Downloaded successfully to ${APK_FILE}"
fi

echo "Installing AnkiDroid on emulator..."
adb install "$APK_FILE"

if [ $? -eq 0 ]; then
    echo "AnkiDroid installed successfully!"
    
    # Grant necessary permissions
    echo "Granting storage permissions..."
    adb shell pm grant com.ichi2.anki android.permission.READ_EXTERNAL_STORAGE 2>/dev/null || true
    adb shell pm grant com.ichi2.anki android.permission.WRITE_EXTERNAL_STORAGE 2>/dev/null || true
    
    # For Android 13+ (API 33+), grant new media permissions
    echo "Granting media permissions for Android 13+..."
    adb shell pm grant com.ichi2.anki android.permission.READ_MEDIA_IMAGES 2>/dev/null || true
    adb shell pm grant com.ichi2.anki android.permission.READ_MEDIA_VIDEO 2>/dev/null || true
    adb shell pm grant com.ichi2.anki android.permission.READ_MEDIA_AUDIO 2>/dev/null || true
    
    echo "Launching AnkiDroid..."
    adb shell monkey -p com.ichi2.anki -c android.intent.category.LAUNCHER 1
    
    echo ""
    echo "AnkiDroid is now installed and running!"
    echo "You can view it through VNC on display :1 (port 5901)"
else
    echo "Error: Failed to install AnkiDroid"
    exit 1
fi