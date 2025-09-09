#!/usr/bin/env bash
set -euo pipefail

if [[ ${1-} == "" ]]; then
  echo "Usage: scripts/set-version.sh <semver>" >&2
  exit 1
fi

NEW_VER="$1"
if [[ ! "$NEW_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "Error: version must be semver (e.g., 0.2.3)" >&2
  exit 1
fi

if command -v cargo >/dev/null 2>&1 && cargo --list | grep -q "set-version"; then
  cargo set-version "$NEW_VER"
else
  # Robust fallback: update only the [package] section's version using awk
  tmp=$(mktemp)
  awk -v new="$NEW_VER" '
    BEGIN{in_pkg=0; done=0}
    /^\[package\]/{in_pkg=1}
    /^\[/{if($0!~/^\[package\]/){in_pkg=0}}
    in_pkg && /^version *= *\"/ && !done { sub(/version *= *\"[^\"]+\"/, "version = \"" new "\""); done=1 }
    { print }
  ' Cargo.toml > "$tmp"
  mv "$tmp" Cargo.toml
fi

echo "Set Cargo.toml version to ${NEW_VER}" >&2
