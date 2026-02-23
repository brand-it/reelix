#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <major|minor|bug>"
  exit 1
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command not found: $command_name"
    exit 1
  fi
}

resolve_main_ref() {
  if git rev-parse --verify --quiet refs/remotes/origin/main >/dev/null; then
    echo "origin/main"
    return
  fi

  if git rev-parse --verify --quiet refs/heads/main >/dev/null; then
    echo "main"
    return
  fi

  echo ""
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

require_command git
require_command perl
require_command npm
require_command cargo

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package_json="$root_dir/package.json"
cargo_toml="$root_dir/src-tauri/Cargo.toml"
cargo_lock="$root_dir/src-tauri/Cargo.lock"
tauri_conf="$root_dir/src-tauri/tauri.conf.json"

cd "$root_dir"

# Always fetch latest main from origin if available
if git remote get-url origin >/dev/null 2>&1; then
  echo "Fetching latest main from origin..."
  git fetch origin main
fi

main_ref="$(resolve_main_ref)"
if [ -z "$main_ref" ]; then
  echo "Unable to find main branch reference (origin/main or main)"
  exit 1
fi

current_version="$(git show "$main_ref:package.json" | grep -m1 -oE '"version"[[:space:]]*:[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"' | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')"

if [ "$current_version" = "" ]; then
  echo "Unable to read version from package.json in $main_ref"
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

perl -0777 -i -pe "s/(\[package\][\s\S]*?\nversion\s*=\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/" "$cargo_toml"
perl -0777 -i -pe "s/^(\s*\"version\"\s*:\s*\")\d+\.\d+\.\d+(\")/\${1}$new_version\${2}/m" "$tauri_conf"

echo "Refreshing lock files..."
npm install --package-lock-only --no-audit --no-fund
cargo build --manifest-path "$cargo_toml"

if [ ! -f "$cargo_lock" ]; then
  echo "Failed to generate src-tauri/Cargo.lock"
  exit 1
fi

echo "Version bumped ($bump_type): $current_version -> $new_version"
