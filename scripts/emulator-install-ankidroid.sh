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
ANKI_VERSION="${ANKI_VERSION:-2.22.3}"   # latest stable as of 2024
APK_NAME="AnkiDroid-${ANKI_VERSION}-full-universal.apk"
APK_PATH="${APK_DIR}/${APK_NAME}"

# Try to download if missing. Use GitHub releases.
if [ ! -f "${APK_PATH}" ]; then
  echo "Downloading AnkiDroid ${ANKI_VERSION}..."
  # GitHub releases URL for universal APK
  URL="https://github.com/ankidroid/Anki-Android/releases/download/v${ANKI_VERSION}/AnkiDroid-${ANKI_VERSION}-full-universal.apk"
  curl -fL "${URL}" -o "${APK_PATH}"
fi

echo "Installing ${APK_PATH} to emulator..."
adb install -r "${APK_PATH}"

# Simple verification: can we see the package?
adb shell pm list packages | grep -q "com.ichi2.anki" && echo "AnkiDroid installed."
