#!/usr/bin/env bash
set -euo pipefail

# Publishes platform-specific npm packages and the root dotagents shim package.
# Required env: TAG, GH_TOKEN, NODE_AUTH_TOKEN

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')

declare -A PLATFORM_MAP
PLATFORM_MAP[linux-x64-musl]="linux-x64"
PLATFORM_MAP[linux-arm64-musl]="linux-arm64"
PLATFORM_MAP[macos-arm64]="darwin-arm64"
PLATFORM_MAP[macos-x86]="darwin-x64"
PLATFORM_MAP[windows-x64]="win32-x64"

declare -A OS_MAP
OS_MAP[linux-x64-musl]="linux"
OS_MAP[linux-arm64-musl]="linux"
OS_MAP[macos-arm64]="darwin"
OS_MAP[macos-x86]="darwin"
OS_MAP[windows-x64]="win32"

declare -A CPU_MAP
CPU_MAP[linux-x64-musl]="x64"
CPU_MAP[linux-arm64-musl]="arm64"
CPU_MAP[macos-arm64]="arm64"
CPU_MAP[macos-x86]="x64"
CPU_MAP[windows-x64]="x64"

# Download binaries
for platform in linux-x64-musl linux-arm64-musl macos-arm64 macos-x86 windows-x64; do
  if [ "$platform" = "windows-x64" ]; then
    gh release download "$TAG" -p "dotagents-${platform}.exe" -D "./npm-packages/${platform}/"
  else
    gh release download "$TAG" -p "dotagents-${platform}" -D "./npm-packages/${platform}/"
  fi
done

# Publish platform packages
for platform in linux-x64-musl linux-arm64-musl macos-arm64 macos-x86 windows-x64; do
  PKG_NAME="@soorya-u/dotagents-${PLATFORM_MAP[$platform]}"
  PKG_DIR="./npm-packages/${platform}"

  if [ "$platform" = "windows-x64" ]; then
    BIN_NAME="dotagents-${platform}.exe"
  else
    BIN_NAME="dotagents-${platform}"
    chmod +x "${PKG_DIR}/${BIN_NAME}"
  fi

  cat > "${PKG_DIR}/package.json" <<EOF
{
  "name": "${PKG_NAME}",
  "version": "${VERSION}",
  "description": "Platform-specific binary for dotagents (${PLATFORM_MAP[$platform]})",
  "os": ["${OS_MAP[$platform]}"],
  "cpu": ["${CPU_MAP[$platform]}"],
  "main": "${BIN_NAME}",
  "files": ["${BIN_NAME}"]
}
EOF
  cd "${PKG_DIR}"
  npm publish --access public
  cd -
done

# Publish root shim package
mkdir -p ./npm-root/bin

cat > ./npm-root/package.json <<EOF
{
  "name": "@soorya-u/dotagents",
  "version": "${VERSION}",
  "description": "An agent configuration manager and templater",
  "bin": {
    "dotagents": "./bin/dotagents"
  },
  "scripts": {
    "postinstall": "node ./postinstall.js"
  },
  "optionalDependencies": {
    "@soorya-u/dotagents-linux-x64": "${VERSION}",
    "@soorya-u/dotagents-linux-arm64": "${VERSION}",
    "@soorya-u/dotagents-darwin-arm64": "${VERSION}",
    "@soorya-u/dotagents-darwin-x64": "${VERSION}",
    "@soorya-u/dotagents-win32-x64": "${VERSION}"
  },
  "files": ["bin/", "postinstall.js"]
}
EOF

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "${SCRIPT_DIR}/npm_postinstall.js" ./npm-root/postinstall.js

cd ./npm-root
npm publish --access public
