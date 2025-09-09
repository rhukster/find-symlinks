#!/usr/bin/env bash
set -euo pipefail

if [[ ${1-} == "" ]]; then
  echo "Usage: scripts/make-formula.sh <version>  # e.g. 0.1.0" >&2
  exit 1
fi

VER="$1"
if [[ ! "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "Error: version must be semver like 0.1.0" >&2
  exit 1
fi

# Derive owner/repo from git remote
REMOTE=$(git config --get remote.origin.url)
if [[ "$REMOTE" =~ github.com[:/]{1}([^/]+)/([^/.]+)(\.git)?$ ]]; then
  OWNER="${BASH_REMATCH[1]}"
  REPO="${BASH_REMATCH[2]}"
else
  echo "Cannot parse origin remote: $REMOTE" >&2
  exit 1
fi

URL="https://github.com/${OWNER}/${REPO}/archive/refs/tags/v${VER}.tar.gz"
TMP=$(mktemp)
echo "Fetching ${URL}" >&2
curl -fsSL "$URL" -o "$TMP"
SHA=$(shasum -a 256 "$TMP" | awk '{print $1}')
rm -f "$TMP"

mkdir -p HomebrewFormula
FORMULA="HomebrewFormula/${REPO}.rb"

# Compute a Ruby-safe CamelCase class name from repo (e.g., find-symlinks -> FindSymlinks)
CLASS_NAME=$(python3 - <<PY
import re, sys
r = "${REPO}"
parts = re.split(r"[^A-Za-z0-9]+", r)
print(''.join(p.capitalize() for p in parts if p))
PY
)

cat > "$FORMULA" <<EOF
class ${CLASS_NAME} < Formula
  desc "Fast symlink finder"
  homepage "https://github.com/${OWNER}/${REPO}"
  url "${URL}"
  sha256 "${SHA}"
  head "https://github.com/${OWNER}/${REPO}.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "find-symlinks", shell_output("#{bin}/find-symlinks --version")
  end
end
EOF

echo "Wrote $FORMULA"
