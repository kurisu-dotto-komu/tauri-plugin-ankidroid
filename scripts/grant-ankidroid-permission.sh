#!/bin/bash

# Grant AnkiDroid database permission to the test app
echo "Granting AnkiDroid permission to com.tauri.ankidroid.demo..."

# First check if the app is installed
if adb shell pm list packages | grep -q "com.tauri.ankidroid.demo"; then
    # Grant the AnkiDroid permission
    adb shell pm grant com.tauri.ankidroid.demo com.ichi2.anki.permission.READ_WRITE_DATABASE 2>/dev/null
    
    # Check if the command succeeded
    if [ $? -eq 0 ]; then
        echo "✅ Permission granted successfully"
    else
        echo "⚠️  Could not grant permission automatically. This permission might need manual approval."
        echo "    Please go to: Settings > Apps > AnkiDroid Demo > Permissions"
        echo "    And manually enable the AnkiDroid permission."
    fi
else
    echo "❌ App com.tauri.ankidroid.demo is not installed"
    exit 1
fi