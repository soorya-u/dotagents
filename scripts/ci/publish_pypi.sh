#!/usr/bin/env bash
set -euo pipefail

# Builds platform-specific wheels and publishes to PyPI using trusted publishing.
# Required env: TAG, GH_TOKEN

TAG="${TAG:?TAG is required}"
VERSION=$(echo "$TAG" | sed 's/^v//')
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
  WHEEL_NAME="${DIST_NAME}-${VERSION}-py3-none-${PLAT_TAG}.whl"
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
  DATA_DIR="${WORK_DIR}/${DIST_NAME}-${VERSION}.data/scripts"
  DIST_INFO="${WORK_DIR}/${DIST_NAME}-${VERSION}.dist-info"
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

  # RECORD (empty — pip doesn't strictly require hashes for install)
  touch "${DIST_INFO}/RECORD"

  # Build the wheel (zip with .whl extension)
  cd "${WORK_DIR}"
  zip -r "${WHEEL_NAME}" "${DIST_NAME}-${VERSION}.data/" "${DIST_NAME}-${VERSION}.dist-info/"
  mv "${WHEEL_NAME}" "${OLDPWD}/dist/"
  cd "${OLDPWD}"

  rm -rf "${WORK_DIR}"
done

# Wheels are in dist/ — pypa/gh-action-pypi-publish handles the upload
