#!/bin/bash
set -euo pipefail

usage() {
  echo "Usage: $0 -i <input_dir>"
  echo "Splits shared libraries and binaries for Linux binary"
  echo "example:"
  echo "scripts/rename-linux-makemkv.sh -i tmp/linux/"
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

LIB_OUTPUT="$(realpath "./src-tauri/libraries/linux")"
BIN_OUTPUT="$(realpath "./src-tauri/binaries")"
mkdir -p "$LIB_OUTPUT" "$BIN_OUTPUT"

find "$INPUT_DIR" -type f -name "*.so*" | while read -r lib; do
  base=$(basename "$lib")
  cp -f "$lib" "$LIB_OUTPUT/$base"
  echo "Library: $lib > $LIB_OUTPUT/$base"
done

if [ -f "$INPUT_DIR/makemkvcon" ]; then
  cp -f "$INPUT_DIR/makemkvcon" "$BIN_OUTPUT/makemkvcon-x86_64-unknown-linux-gnu"
  echo "Binary: $INPUT_DIR/makemkvcon > $BIN_OUTPUT/makemkvcon-x86_64-unknown-linux-gnu"
else
  echo "makemkvcon not found in $INPUT_DIR"
fi

if [ -f "$INPUT_DIR/mmgplsrv" ]; then
  cp -f "$INPUT_DIR/mmgplsrv" "$BIN_OUTPUT/mmgplsrv-x86_64-unknown-linux-gnu"
  echo "Binary: $INPUT_DIR/mmgplsrv > $BIN_OUTPUT/mmgplsrv-x86_64-unknown-linux-gnu"
else
  echo "mmgplsrv not found in $INPUT_DIR"
fi

if [ -f "$INPUT_DIR/mmccextr" ]; then
  cp -f "$INPUT_DIR/mmccextr" "$BIN_OUTPUT/mmccextr-x86_64-unknown-linux-gnu"
  echo "Binary: $INPUT_DIR/mmccextr > $BIN_OUTPUT/mmccextr-x86_64-unknown-linux-gnu"
else
  echo "mmccextr not found in $INPUT_DIR"
fi
