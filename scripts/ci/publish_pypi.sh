#!/usr/bin/env bash
set -euo pipefail

# Builds platform-specific wheels and publishes to PyPI using trusted publishing.
# Required env: TAG, GH_TOKEN
# Supports pre-release tags (e.g. v0.0.0-nightly, v0.0.0-alpha.1, v0.0.0-rc.1)
# by converting semver pre-release identifiers to PEP 440 format.

TAG="${TAG:?TAG is required}"
SEMVER=$(echo "$TAG" | sed 's/^v//')

# Convert semver pre-release to PEP 440
# e.g. 0.0.0-nightly.20250507 -> 0.0.0.dev20250507, 0.0.0-alpha.1 -> 0.0.0a1,
#      0.0.0-beta.2 -> 0.0.0b2, 0.0.0-rc.1 -> 0.0.0rc1
if [[ "$SEMVER" == *-* ]]; then
  BASE_VERSION="${SEMVER%%-*}"
  PRERELEASE="${SEMVER#*-}"

  if [[ "$PRERELEASE" =~ ^alpha\.?([0-9]*)$ ]]; then
    VERSION="${BASE_VERSION}a${BASH_REMATCH[1]:-0}"
  elif [[ "$PRERELEASE" =~ ^beta\.?([0-9]*)$ ]]; then
    VERSION="${BASE_VERSION}b${BASH_REMATCH[1]:-0}"
  elif [[ "$PRERELEASE" =~ ^rc\.?([0-9]*)$ ]]; then
    VERSION="${BASE_VERSION}rc${BASH_REMATCH[1]:-0}"
  elif [[ "$PRERELEASE" =~ ^nightly\.?([0-9]*)$ ]]; then
    VERSION="${BASE_VERSION}.dev${BASH_REMATCH[1]:-0}"
  else
    # Generic pre-release (test, dev, etc.) -> .dev0
    VERSION="${BASE_VERSION}.dev0"
  fi
else
  VERSION="$SEMVER"
fi
PKG_NAME="py_dotagents"
DIST_NAME="py-dotagents"

declare -A WHEEL_PLATFORM
WHEEL_PLATFORM[linux-x64-musl]="manylinux_2_17_x86_64.manylinux2014_x86_64.musllinux_1_1_x86_64"
WHEEL_PLATFORM[linux-arm64-musl]="manylinux_2_17_aarch64.manylinux2014_aarch64.musllinux_1_1_aarch64"
WHEEL_PLATFORM[macos-arm64]="macosx_11_0_arm64"
WHEEL_PLATFORM[macos-x86]="macosx_10_12_x86_64"
WHEEL_PLATFORM[windows-x64]="win_amd64"

mkdir -p dist

for platform in linux-x64-musl linux-arm64-musl macos-arm64 macos-x86 windows-x64; do
  PLAT_TAG="${WHEEL_PLATFORM[$platform]}"
  WHEEL_NAME="${PKG_NAME}-${VERSION}-py3-none-${PLAT_TAG}.whl"
  WORK_DIR=$(mktemp -d)

  # Download binary
  if [ "$platform" = "windows-x64" ]; then
    gh release download "$TAG" -p "dotagents-${platform}.exe" -D "${WORK_DIR}/"
    BIN_NAME="dotagents.exe"
    mv "${WORK_DIR}/dotagents-${platform}.exe" "${WORK_DIR}/${BIN_NAME}"
  else
    gh release download "$TAG" -p "dotagents-${platform}" -D "${WORK_DIR}/"
    BIN_NAME="dotagents"
    mv "${WORK_DIR}/dotagents-${platform}" "${WORK_DIR}/${BIN_NAME}"
    chmod +x "${WORK_DIR}/${BIN_NAME}"
  fi

  # Create wheel structure
  DATA_DIR="${WORK_DIR}/${PKG_NAME}-${VERSION}.data/scripts"
  DIST_INFO="${WORK_DIR}/${PKG_NAME}-${VERSION}.dist-info"
  mkdir -p "${DATA_DIR}" "${DIST_INFO}"

  mv "${WORK_DIR}/${BIN_NAME}" "${DATA_DIR}/${BIN_NAME}"

  # METADATA
  cat > "${DIST_INFO}/METADATA" <<EOF
Metadata-Version: 2.1
Name: ${DIST_NAME}
Version: ${VERSION}
Summary: An agent configuration manager and templater
Home-page: https://github.com/soorya-u/dotagents
License: MIT OR Apache-2.0
Author: Soorya U
Author-email: sooryau7@gmail.com
Classifier: Development Status :: 4 - Beta
Classifier: Environment :: Console
Classifier: License :: OSI Approved :: MIT License
Classifier: License :: OSI Approved :: Apache Software License
Classifier: Operating System :: OS Independent
Classifier: Programming Language :: Rust
EOF

  # WHEEL
  cat > "${DIST_INFO}/WHEEL" <<EOF
Wheel-Version: 1.0
Generator: dotagents-release
Root-Is-Purelib: false
Tag: py3-none-${PLAT_TAG}
EOF

  # RECORD — list every file in the wheel with sha256 hash and size
  RECORD_FILE="${DIST_INFO}/RECORD"
  : > "${RECORD_FILE}"
  cd "${WORK_DIR}"
  for f in $(find "${PKG_NAME}-${VERSION}.data" "${PKG_NAME}-${VERSION}.dist-info" -type f ! -name RECORD); do
    HASH=$(sha256sum "$f" | cut -d' ' -f1)
    HASH_B64=$(echo -n "$HASH" | xxd -r -p | base64 | tr -d '=\n' | tr '+/' '-_')
    SIZE=$(wc -c < "$f")
    echo "${f},sha256=${HASH_B64},${SIZE}" >> "${RECORD_FILE}"
  done
  echo "${PKG_NAME}-${VERSION}.dist-info/RECORD,," >> "${RECORD_FILE}"
  cd "${OLDPWD}"

  # Build the wheel (zip with .whl extension)
  cd "${WORK_DIR}"
  zip -r "${WHEEL_NAME}" "${PKG_NAME}-${VERSION}.data/" "${PKG_NAME}-${VERSION}.dist-info/"
  mv "${WHEEL_NAME}" "${OLDPWD}/dist/"
  cd "${OLDPWD}"

  rm -rf "${WORK_DIR}"
done

# Wheels are in dist/ — pypa/gh-action-pypi-publish handles the upload
