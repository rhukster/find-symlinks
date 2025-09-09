#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
STATE_DIR="$ROOT_DIR/build"
STATE_FILE="$STATE_DIR/build-number"
mkdir -p "$STATE_DIR"

N=0
if [[ -f "$STATE_FILE" ]]; then
  N=$(cat "$STATE_FILE" 2>/dev/null | tr -d '\r' | tr -d '\n' || echo 0)
  [[ "$N" =~ ^[0-9]+$ ]] || N=0
fi
N=$((N+1))
echo "$N" > "$STATE_FILE"
export BUILD_NUMBER="$N"

echo "BUILD_NUMBER=$BUILD_NUMBER" >&2
exec cargo build "$@"

