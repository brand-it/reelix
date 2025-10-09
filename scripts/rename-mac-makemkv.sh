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
    i) INPUT_DIR="$OPTARG" ;;
    *) usage ;;
  esac
done

[ -z "${INPUT_DIR:-}" ] && usage
[ -d "$INPUT_DIR" ] || { echo "Input directory not found: $INPUT_DIR"; exit 1; }

LIB_OUTPUT="./src-tauri/libraries/mac-osx"
BIN_OUTPUT="./src-tauri/binaries"
mkdir -p "$LIB_OUTPUT" "$BIN_OUTPUT"

find "$INPUT_DIR" -type f -name "*.dylib" | while read -r dylib; do
  base=$(basename "$dylib")
  cp -f "$dylib" "$LIB_OUTPUT/$base"
  echo "Library: $base"
done

if [ -f "$INPUT_DIR/makemkvcon" ]; then
  cp -f "$INPUT_DIR/makemkvcon" "$BIN_OUTPUT/makemkvcon"
  echo "Binary: makemkvcon"
else
  echo "makemkvcon not found in $INPUT_DIR"
fi
