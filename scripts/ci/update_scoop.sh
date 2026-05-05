#!/usr/bin/env bash
set -euo pipefail

# Updates the Scoop manifest with new version and SHA256 hash.
# Required env: TAG, RELEASE_PAT, SHA_HASH

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')
RELEASE_PAT="${RELEASE_PAT:?RELEASE_PAT is required}"
SHA_HASH="${SHA_HASH:?SHA_HASH is required}"

git clone "https://x-access-token:${RELEASE_PAT}@github.com/soorya-u/scoop-dotagents.git"
cd scoop-dotagents
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

jq --arg version "$VERSION" --arg hash "$SHA_HASH" \
  '.version = $version | .architecture."64bit".hash = $hash | .architecture."64bit".url = "https://github.com/soorya-u/dotagents/releases/download/v\($version)/dotagents-windows-x64.exe"' \
  bucket/dotagents.json > bucket/dotagents.json.tmp
mv bucket/dotagents.json.tmp bucket/dotagents.json

git add -A
git commit -m "chore: update dotagents to v${VERSION}"
git push origin main
