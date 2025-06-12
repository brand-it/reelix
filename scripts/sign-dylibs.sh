#!/usr/bin/env bash
set -euo pipefail

# ----------------------------------------
# ENVIRONMENT
# ----------------------------------------
: "${CERT_ID:?Need CERT_ID (e.g. \"Developer ID Application: Foo Bar (TEAMID)\" or the hash)}"

INPUT_PATH="${1:?Usage: $0 /path/to/directory}"

# Try to resolve the full absolute path
if ! DIR_PATH="$(cd "$INPUT_PATH" 2>/dev/null && pwd)"; then
  echo "❌ Directory not found or inaccessible: $INPUT_PATH"
  echo "🔍 Current working directory: $(pwd)"
  exit 1
fi

echo "📁 Using directory: $DIR_PATH"

for DYLIB_PATH in "$DIR_PATH"/*.dylib; do
  if [[ -f "$DYLIB_PATH" ]]; then
    echo "✍️  Signing $DYLIB_PATH with identity: $CERT_ID"
    codesign --force \
             --options runtime \
             --timestamp \
             --sign "$CERT_ID" \
             "$DYLIB_PATH"

    echo "🔍 Verifying signature for $DYLIB_PATH..."
    codesign --verify --strict --verbose=2 "$DYLIB_PATH"
    echo "✅ Success! $DYLIB_PATH is now signed."
  fi
done
