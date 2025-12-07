#!/bin/bash

set -e

# Usage message
usage() {
  echo "Usage: $0 -i <input_dir> -o <output_dir> [-s <suffix>]"
  echo "  -i   Input directory containing files to rename"
  echo "  -o   Output directory to write renamed files"
  echo "  -s   Suffix to append (default: -aarch64-apple-darwin)"
  exit 1
}


# Parse args
while getopts ":i:o:s:" opt; do
  case $opt in
    i) INPUT_DIR="$OPTARG" ;;
    o) OUTPUT_DIR="$OPTARG" ;;
    s) SUFFIX="$OPTARG" ;;
    *) usage ;;
  esac
done

# Validate input
[ -z "$INPUT_DIR" ] || [ -z "$OUTPUT_DIR" ] && usage
[ -d "$INPUT_DIR" ] || { echo "Input directory not found: $INPUT_DIR"; exit 1; }
mkdir -p "$OUTPUT_DIR"

# Rename files
for file in "$INPUT_DIR"/*; do
  [ -f "$file" ] || continue

  filename=$(basename "$file")

  # Skip if already contains suffix
  if [[ "$filename" == *"$SUFFIX"* ]]; then
    continue
  fi

  # Simply append the suffix to the original filename
  newname="${filename}${SUFFIX}"

  cp -f "$file" "$OUTPUT_DIR/$newname"
  echo "Copied & Renamed: $filename -> $newname"
done
