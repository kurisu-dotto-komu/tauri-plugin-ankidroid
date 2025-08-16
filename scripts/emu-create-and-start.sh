#!/usr/bin/env bash
set -euo pipefail

AVD_NAME="${AVD_NAME:-ankidroid-test}"
SYSTEM_IMAGE="${SYSTEM_IMAGE:-system-images;android-34;google_apis;x86_64}"
DEVICE_ID="${DEVICE_ID:-pixel_4}"
EMULATOR_FLAGS=${EMULATOR_FLAGS:-"-gpu host -netdelay none -netspeed full -no-snapshot"}

# Create AVD if it doesn't exist
if ! avdmanager list avd | grep -q "Name: ${AVD_NAME}"; then
  sdkmanager --install "${SYSTEM_IMAGE}"
  echo "no" | avdmanager create avd -n "${AVD_NAME}" -k "${SYSTEM_IMAGE}" --device "${DEVICE_ID}" --force
fi

# Start GUI emulator (X11 forwarded from host)
# NOTE: Requires host to allow X access: run `xhost +local:` once on host (dev-only)
# For safer: `xhost +SI:localuser:$(id -un)`
emulator -avd "${AVD_NAME}" ${EMULATOR_FLAGS} &
EMU_PID=$!

# Wait for device to finish booting
adb wait-for-device
BOOTED=""
until [ "$BOOTED" = "1" ]; do
  sleep 2
  BOOTED=$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')
done

echo "Emulator booted (PID ${EMU_PID})."
