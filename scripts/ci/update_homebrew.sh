#!/usr/bin/env bash
set -euo pipefail

# Updates the Homebrew formula with new version and SHA256 hashes.
# Required env: TAG, RELEASE_PAT, SHA_ARM64, SHA_X86
# Pre-release tags (e.g. v0.0.0-nightly, v0.0.0-alpha.1, v0.0.0-rc.1) update
# a channel-specific formula (e.g. Formula/dotagents-nightly.rb) instead of stable.

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')
RELEASE_PAT="${RELEASE_PAT:?RELEASE_PAT is required}"
SHA_ARM64="${SHA_ARM64:?SHA_ARM64 is required}"
SHA_X86="${SHA_X86:?SHA_X86 is required}"

# Determine which formula to update based on pre-release channel
if [[ "$VERSION" == *-* ]]; then
  PRERELEASE="${VERSION#*-}"
  CHANNEL=$(echo "$PRERELEASE" | sed 's/[^a-zA-Z].*//')
  CHANNEL="${CHANNEL:-prerelease}"
  FORMULA="Formula/dotagents-${CHANNEL}.rb"
  COMMIT_MSG="chore: update dotagents-${CHANNEL} to v${VERSION}"
else
  FORMULA="Formula/dotagents.rb"
  COMMIT_MSG="chore: update dotagents to v${VERSION}"
fi

git clone "https://x-access-token:${RELEASE_PAT}@github.com/soorya-u/homebrew-dotagents.git"
cd homebrew-dotagents
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

if [ ! -f "$FORMULA" ]; then
  echo "Warning: ${FORMULA} not found in tap, skipping Homebrew update"
  exit 0
fi

sed -i "s/version \".*\"/version \"${VERSION}\"/" "$FORMULA"
sed -i "0,/sha256 \".*\"/s//sha256 \"${SHA_ARM64}\"/" "$FORMULA"
sed -i "0,/sha256 \".*\"/!{0,/sha256 \".*\"/s//sha256 \"${SHA_X86}\"/}" "$FORMULA"

git add -A
git commit -m "$COMMIT_MSG"
git push origin main
