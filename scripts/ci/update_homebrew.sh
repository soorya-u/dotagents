#!/usr/bin/env bash
set -euo pipefail

# Updates the Homebrew formula with new version and SHA256 hashes.
# Required env: TAG, RELEASE_PAT, SHA_ARM64, SHA_X86

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')
RELEASE_PAT="${RELEASE_PAT:?RELEASE_PAT is required}"
SHA_ARM64="${SHA_ARM64:?SHA_ARM64 is required}"
SHA_X86="${SHA_X86:?SHA_X86 is required}"

git clone "https://x-access-token:${RELEASE_PAT}@github.com/soorya-u/homebrew-dotagents.git"
cd homebrew-dotagents
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

sed -i "s/version \".*\"/version \"${VERSION}\"/" Formula/dotagents.rb
sed -i "0,/sha256 \".*\"/s//sha256 \"${SHA_ARM64}\"/" Formula/dotagents.rb
sed -i "0,/sha256 \".*\"/!{0,/sha256 \".*\"/s//sha256 \"${SHA_X86}\"/}" Formula/dotagents.rb

git add -A
git commit -m "chore: update dotagents to v${VERSION}"
git push origin main
