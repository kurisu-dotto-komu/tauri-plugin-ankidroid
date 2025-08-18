#!/bin/bash

# Script to start/stop Android emulator with VNC display

AVD_NAME="Pixel_7_API_35"
ACTION="${1:-start}"

# Setup environment - SDK already installed by devcontainer
export ANDROID_HOME=${ANDROID_HOME:-$ANDROID_SDK_ROOT}
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator
export DISPLAY=:1

if [ "$ACTION" = "stop" ]; then
    echo "Stopping emulator..."
    adb emu kill 2>/dev/null || true
    pkill -f "emulator.*$AVD_NAME" 2>/dev/null || true
    echo "Emulator stopped"
    exit 0
fi

# Check if emulator is already running
if pgrep -f "emulator.*$AVD_NAME" > /dev/null; then
    echo "Emulator is already running"
    echo "Connect via VNC on display :1 (port 5901)"
    exit 0
fi

# Check if AVD exists
if ! avdmanager list avd | grep -q "$AVD_NAME"; then
    echo "Error: AVD $AVD_NAME not found. Run 'npm run emu:create' first"
    exit 1
fi

echo "Starting Android emulator on display :1 (VNC)..."
echo "This may take a few minutes on first boot..."

# Start emulator with specific display settings for VNC
nohup emulator \
    -avd "$AVD_NAME" \
    -no-audio \
    -no-boot-anim \
    -gpu swiftshader_indirect \
    -memory 4096 \
    -no-snapshot-load \
    > /tmp/emulator.log 2>&1 &

EMULATOR_PID=$!
echo "Emulator starting with PID: $EMULATOR_PID"

# Wait for emulator to boot
echo "Waiting for emulator to boot..."
timeout 120 bash -c '
while ! adb devices | grep -q "emulator.*device"; do
    echo -n "."
    sleep 2
done
'

if [ $? -eq 0 ]; then
    echo ""
    echo "Emulator started successfully!"
    echo "Connect via VNC to view the emulator (display :1, port 5901)"
    
    # Wait for boot to complete
    echo "Waiting for boot to complete..."
    adb wait-for-device
    adb shell 'while [[ -z $(getprop sys.boot_completed) ]]; do sleep 1; done'
    
    echo "Emulator is ready!"
    echo "Run 'npm run emu:install-anki' to install AnkiDroid"
else
    echo ""
    echo "Error: Emulator failed to start within timeout"
    echo "Check /tmp/emulator.log for details"
    exit 1
fi