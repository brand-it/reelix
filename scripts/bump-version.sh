#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <major|minor|bug>"
  exit 1
}

if [ "${1:-}" = "" ]; then
  usage
fi

bump_type="${1,,}"
if [ "$bump_type" = "patch" ]; then
  bump_type="bug"
fi

case "$bump_type" in
major|minor|bug) ;;
*) usage ;;
esac

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package_json="$root_dir/package.json"
package_lock="$root_dir/package-lock.json"
cargo_toml="$root_dir/src-tauri/Cargo.toml"
tauri_conf="$root_dir/src-tauri/tauri.conf.json"

current_version="$(grep -m1 -oE '"version"[[:space:]]*:[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"' "$package_json" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')"

if [ "$current_version" = "" ]; then
  echo "Unable to read version from package.json"
  exit 1
fi

IFS='.' read -r major minor bug <<< "$current_version"

case "$bump_type" in
  major)
    major=$((major + 1))
    minor=0
    bug=0
    ;;
  minor)
    minor=$((minor + 1))
    bug=0
    ;;
  bug)
    bug=$((bug + 1))
    ;;
esac

new_version="$major.$minor.$bug"

perl -0777 -i -pe "s/^(\s*\"version\"\s*:\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/m" "$package_json"

if [ -f "$package_lock" ]; then
  perl -0777 -i -pe "s/^(\s*\"version\"\s*:\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/m; s/(\"packages\"\s*:\s*\{\s*\"\"\s*:\s*\{[^}]*?\"version\"\s*:\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/s" "$package_lock"
fi

perl -0777 -i -pe "s/(\[package\][\s\S]*?\nversion\s*=\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/" "$cargo_toml"
perl -0777 -i -pe "s/^(\s*\"version\"\s*:\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/m" "$tauri_conf"

echo "Version bumped ($bump_type): $current_version -> $new_version"
