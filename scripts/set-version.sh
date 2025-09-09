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
  # Fallback: replace the first package version line
  if sed --version >/dev/null 2>&1; then
    # GNU sed
    sed -i "0,/^version = \".*\"/s//version = \"$NEW_VER\"/" Cargo.toml
  else
    # BSD/macOS sed
    sed -i '' "0,/^version = \".*\"/s//version = \"$NEW_VER\"/" Cargo.toml
  fi
fi

echo "Set Cargo.toml version to ${NEW_VER}" >&2
