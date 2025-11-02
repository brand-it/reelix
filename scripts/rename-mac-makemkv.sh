#!/bin/bash
set -euo pipefail

usage() {
  echo "Usage: $0 -i <input_dir>"
  echo "Splits dylibs and binaries for macOS binary"
  echo "example:"
  echo "scripts/rename-mac-makemkv.sh -i tmp/mac/"
  exit 1
}

while getopts ":i:" opt; do
  case $opt in
    i) INPUT_DIR="$(realpath "$OPTARG")" ;;
    *) usage ;;
  esac
done

[ -z "${INPUT_DIR:-}" ] && usage
[ -d "$INPUT_DIR" ] || { echo "Input directory not found: $INPUT_DIR"; exit 1; }

LIB_OUTPUT="$(realpath "./src-tauri/libraries/mac-osx")"
BIN_OUTPUT="$(realpath "./src-tauri/binaries")"
mkdir -p "$LIB_OUTPUT" "$BIN_OUTPUT"

find "$INPUT_DIR" -type f -name "*.dylib" | while read -r dylib; do
  base=$(basename "$dylib")
  cp -f "$dylib" "$LIB_OUTPUT/$base"
  echo "Library: $dylib > $LIB_OUTPUT/$base"
done

if [ -f "$INPUT_DIR/makemkvcon" ]; then
  cp -f "$INPUT_DIR/makemkvcon" "$BIN_OUTPUT/makemkvcon-aarch64-apple-darwin"
   echo "Binary: $INPUT_DIR/makemkvcon > $BIN_OUTPUT/makemkvcon-aarch64-apple-darwin"
  cp -f "$INPUT_DIR/makemkvcon" "$BIN_OUTPUT/makemkvcon-x86_64-apple-darwin"
  echo "Binary: $INPUT_DIR/makemkvcon > $BIN_OUTPUT/makemkvcon-x86_64-apple-darwin"
else
  echo "makemkvcon not found in $INPUT_DIR"
fi
