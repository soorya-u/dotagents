#!/usr/bin/env bash
set -euo pipefail

# Updates the Scoop manifest with new version and SHA256 hash.
# Required env: TAG, RELEASE_PAT, SHA_HASH
# Pre-release tags (e.g. v0.0.0-nightly, v0.0.0-alpha.1, v0.0.0-rc.1) update
# a channel-specific manifest (e.g. bucket/dotagents-nightly.json) instead of stable.

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')
RELEASE_PAT="${RELEASE_PAT:?RELEASE_PAT is required}"
SHA_HASH="${SHA_HASH:?SHA_HASH is required}"

# Determine which manifest to update based on pre-release channel
if [[ "$VERSION" == *-* ]]; then
  PRERELEASE="${VERSION#*-}"
  CHANNEL=$(echo "$PRERELEASE" | sed 's/[^a-zA-Z].*//')
  CHANNEL="${CHANNEL:-prerelease}"
  MANIFEST="bucket/dotagents-${CHANNEL}.json"
  COMMIT_MSG="chore: update dotagents-${CHANNEL} to v${VERSION}"
else
  MANIFEST="bucket/dotagents.json"
  COMMIT_MSG="chore: update dotagents to v${VERSION}"
fi

git clone "https://x-access-token:${RELEASE_PAT}@github.com/soorya-u/scoop-dotagents.git"
cd scoop-dotagents
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

if [ ! -f "$MANIFEST" ]; then
  echo "Warning: ${MANIFEST} not found in bucket, skipping Scoop update"
  exit 0
fi

jq --arg version "$VERSION" --arg hash "$SHA_HASH" \
  '.version = $version | .architecture."64bit".hash = $hash | .architecture."64bit".url = "https://github.com/soorya-u/dotagents/releases/download/v\($version)/dotagents-windows-x64.exe"' \
  "$MANIFEST" > "${MANIFEST}.tmp"
mv "${MANIFEST}.tmp" "$MANIFEST"

git add -A
git commit -m "$COMMIT_MSG"
git push origin main
