#!/bin/bash

# Script to create Android Virtual Device (Pixel 7 emulator)

AVD_NAME="Pixel_7_API_35"
DEVICE_ID="pixel_7"
SYSTEM_IMAGE="system-images;android-35;google_apis;x86_64"
PACKAGE_PATH="system-images;android-35;google_apis;x86_64"

# SDK is already installed by devcontainer
export ANDROID_HOME=${ANDROID_HOME:-$ANDROID_SDK_ROOT}
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator

# Check if AVD already exists
if avdmanager list avd | grep -q "$AVD_NAME"; then
    echo "AVD $AVD_NAME already exists. Deleting and recreating..."
    avdmanager delete avd -n "$AVD_NAME"
fi

# Install the system image if not already installed
echo "Ensuring system image is installed..."
sdkmanager "$SYSTEM_IMAGE"

echo "Creating AVD: $AVD_NAME (Pixel 7)..."
echo "no" | avdmanager create avd \
    -n "$AVD_NAME" \
    -k "$PACKAGE_PATH" \
    -c 2048M \
    --force

# Configure AVD for better performance
AVD_CONFIG_DIR="$HOME/.android/avd/${AVD_NAME}.avd"
if [ -d "$AVD_CONFIG_DIR" ]; then
    echo "Configuring AVD for optimal performance..."
    cat >> "$AVD_CONFIG_DIR/config.ini" << EOF
hw.ramSize=4096
hw.gpu.enabled=yes
hw.gpu.mode=swiftshader_indirect
hw.keyboard=yes
hw.mainKeys=yes
showDeviceFrame=no
EOF
fi

echo "AVD $AVD_NAME created successfully!"
echo "Run 'npm run emu:start' to launch the emulator"