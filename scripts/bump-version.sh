#!/usr/bin/env bash
set -euo pipefail

KIND=${1-}
if [[ -z "$KIND" || ! "$KIND" =~ ^(patch|minor|major)$ ]]; then
  echo "Usage: scripts/bump-version.sh <patch|minor|major>" >&2
  exit 1
fi

CUR=$(grep -m1 '^version = "' Cargo.toml | sed -E 's/version = "([^"]+)"/\1/')
IFS=. read -r MA MI PA <<<"$CUR"
MA=${MA:-0}; MI=${MI:-0}; PA=${PA:-0}
case "$KIND" in
  patch) PA=$((PA+1));;
  minor) MI=$((MI+1)); PA=0;;
  major) MA=$((MA+1)); MI=0; PA=0;;
esac
NEW="${MA}.${MI}.${PA}"
scripts/set-version.sh "$NEW"
echo "$CUR -> $NEW"
