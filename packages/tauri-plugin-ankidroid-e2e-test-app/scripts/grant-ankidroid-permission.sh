#!/bin/bash

# Grant permissions to AnkiDroid if it's installed
echo "Checking for AnkiDroid and granting permissions..."

# Check if AnkiDroid is installed
if adb shell pm list packages | grep -q "com.ichi2.anki"; then
    echo "AnkiDroid found, granting storage permissions..."
    
    # Grant storage permissions to AnkiDroid (ignore errors for permissions that can't be granted)
    adb shell pm grant com.ichi2.anki android.permission.WRITE_EXTERNAL_STORAGE 2>/dev/null || true
    adb shell pm grant com.ichi2.anki android.permission.READ_EXTERNAL_STORAGE 2>/dev/null || true
    
    echo "Basic permissions granted to AnkiDroid"
else
    echo "AnkiDroid not found on device. Make sure it's installed before running e2e tests."
    echo "You can install it using: adb install path/to/ankidroid.apk"
fi

# Also grant permissions to our test app
echo "Granting permissions to test app..."
adb shell pm grant com.tauri.ankidroid.demo android.permission.WRITE_EXTERNAL_STORAGE 2>/dev/null || true
adb shell pm grant com.tauri.ankidroid.demo android.permission.READ_EXTERNAL_STORAGE 2>/dev/null || true

# Force stop and restart the test app to ensure it's in a clean state
echo "Restarting test app..."
adb shell am force-stop com.tauri.ankidroid.demo
sleep 1
adb shell am start -n com.tauri.ankidroid.demo/.MainActivity

echo "Permission setup complete."