#!/usr/bin/env bash
set -euo pipefail

# Where to place the downloaded APK (cached in repo or container)
APK_DIR="${APK_DIR:-/workspaces/tauri-plugin-ankidroid/third_party/apk}"
mkdir -p "${APK_DIR}"

# Choose a source. Options:
#  1) F-Droid stable URL pattern (replace VERSION as needed)
#  2) GitHub Releases asset URL (needs curl + jq to pick latest)
#
# For reproducibility, pin a version:
ANKI_VERSION="${ANKI_VERSION:-2.17.6}"   # example; update as needed
APK_NAME="org.ankidroid.android_${ANKI_VERSION}.apk"
APK_PATH="${APK_DIR}/${APK_NAME}"

# Try to download if missing. Update URL if F-Droid changes packaging.
if [ ! -f "${APK_PATH}" ]; then
  echo "Downloading AnkiDroid ${ANKI_VERSION}..."
  # Example F-Droid "universal" APK pattern (adjust if their naming changes):
  URL="https://f-droid.org/repo/org.ankidroid.android_${ANKI_VERSION}.apk"
  curl -fL "${URL}" -o "${APK_PATH}"
fi

echo "Installing ${APK_PATH} to emulator..."
adb install -r "${APK_PATH}"

# Simple verification: can we see the package?
adb shell pm list packages | grep -q "org.ankidroid.android" && echo "AnkiDroid installed."
