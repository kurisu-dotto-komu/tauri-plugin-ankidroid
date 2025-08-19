#!/bin/bash

# Script to start/stop Android emulator with VNC display

AVD_NAME="Pixel_7_API_35"
ACTION="${1:-start}"

# Check for libpulse0 and install if missing
if ! ldconfig -p | grep -q libpulse.so.0; then
    echo "libpulse0 not found, installing..."
    sudo apt-get update && sudo apt-get install -y libpulse0
    if [ $? -ne 0 ]; then
        echo "Failed to install libpulse0. Please install it manually."
        exit 1
    fi
fi

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
    -no-metrics \
    > /tmp/emulator.log 2>&1 &

EMULATOR_PID=$!
echo "Emulator starting with PID: $EMULATOR_PID"
echo "Log file: /tmp/emulator.log"
echo ""

# Monitor emulator log for errors in background
tail -f /tmp/emulator.log 2>/dev/null | while IFS= read -r line; do
    # Filter for important messages
    if echo "$line" | grep -E "(ERROR|FATAL|WARNING|Failed|KVM|accel)" >/dev/null 2>&1; then
        echo "[EMU LOG] $line"
    fi
done &
TAIL_PID=$!

# Wait for emulator to boot
echo "Waiting for emulator to boot (timeout: 5 minutes)..."
MAX_WAIT=300  # 5 minutes
WAIT_COUNT=0

while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    # Check if emulator process is still running
    if ! kill -0 $EMULATOR_PID 2>/dev/null; then
        echo ""
        echo "Error: Emulator process died unexpectedly"
        echo ""
        echo "Last 50 lines of emulator log:"
        echo "================================"
        tail -50 /tmp/emulator.log
        kill $TAIL_PID 2>/dev/null
        exit 1
    fi
    
    # Check if device appears in adb
    if adb devices | grep -q "emulator.*device"; then
        echo ""
        echo "Device detected in adb, checking boot status..."
        
        # Wait for boot to complete with timeout
        BOOT_WAIT=0
        while [ $BOOT_WAIT -lt 60 ]; do
            BOOT_STATUS=$(adb shell getprop sys.boot_completed 2>/dev/null || echo "")
            BOOT_ANIM=$(adb shell getprop init.svc.bootanim 2>/dev/null || echo "")
            
            if [ "$BOOT_STATUS" = "1" ]; then
                echo ""
                echo "✓ Emulator booted successfully!"
                
                # Set 3-button navigation mode
                echo "Setting 3-button navigation mode..."
                adb shell cmd overlay enable com.android.internal.systemui.navbar.threebutton 2>/dev/null || true
                
                echo "Connect via VNC to view the emulator (display :1, port 5901)"
                echo "Run 'npm run emu:install-anki' to install AnkiDroid"
                kill $TAIL_PID 2>/dev/null
                exit 0
            fi
            
            echo "Boot status: boot_completed=$BOOT_STATUS, bootanim=$BOOT_ANIM"
            sleep 2
            BOOT_WAIT=$((BOOT_WAIT + 2))
        done
    fi
    
    # Show progress every 10 seconds
    if [ $((WAIT_COUNT % 10)) -eq 0 ] && [ $WAIT_COUNT -gt 0 ]; then
        echo "Still waiting... ($WAIT_COUNT/$MAX_WAIT seconds)"
        # Check adb devices status
        echo "ADB devices status:"
        adb devices
    else
        echo -n "."
    fi
    
    sleep 2
    WAIT_COUNT=$((WAIT_COUNT + 2))
done

echo ""
echo "Error: Emulator failed to start within timeout ($MAX_WAIT seconds)"
echo ""
echo "Last 50 lines of emulator log:"
echo "================================"
tail -50 /tmp/emulator.log
kill $EMULATOR_PID 2>/dev/null
kill $TAIL_PID 2>/dev/null
exit 1