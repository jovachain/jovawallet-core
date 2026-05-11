#!/usr/bin/env bash
set -euo pipefail
# Script lives at bindings/swift/scripts/; change to bindings/swift/
cd "$(dirname "$0")/.."

# Source cargo env so uniffi-bindgen is on PATH.
# shellcheck source=/dev/null
. "$HOME/.cargo/env"

# Build for every Apple target.
# iOS sim is arm64-only — Apple has fully deprecated the Intel iOS simulator.
for target in aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build -p jova-core-ffi --release --target "$target" --manifest-path ../../Cargo.toml
done

# Mac slice is universal (arm64 + x86_64) via lipo.
mkdir -p ../../target/mac-universal
lipo -create \
  ../../target/aarch64-apple-darwin/release/libjova_core_ffi.a \
  ../../target/x86_64-apple-darwin/release/libjova_core_ffi.a \
  -output ../../target/mac-universal/libjova_core_ffi.a

# Generate the Swift bindings (modulemap + header + .swift).
# uniffi-bindgen (0.30+) generates lowercase filenames: jova_core_ffi.swift etc.
uniffi-bindgen generate \
  --library ../../target/aarch64-apple-darwin/release/libjova_core_ffi.dylib \
  --language swift \
  --out-dir Sources/JovaCore

# Copy only the C header + modulemap to a dedicated headers dir for the XCFramework.
# The .swift file lives in Sources/JovaCore and is compiled by SPM as a Swift source.
rm -rf xcframework-headers
mkdir -p xcframework-headers
cp Sources/JovaCore/jova_core_ffiFFI.h xcframework-headers/
# Clang implicit module lookup requires the canonical name "module.modulemap".
cp Sources/JovaCore/jova_core_ffiFFI.modulemap xcframework-headers/module.modulemap

# Build the XCFramework: device, simulator (arm64-only), macOS universal.
rm -rf JovaCoreFFI.xcframework
xcodebuild -create-xcframework \
  -library ../../target/aarch64-apple-ios/release/libjova_core_ffi.a      -headers xcframework-headers \
  -library ../../target/aarch64-apple-ios-sim/release/libjova_core_ffi.a  -headers xcframework-headers \
  -library ../../target/mac-universal/libjova_core_ffi.a                  -headers xcframework-headers \
  -output JovaCoreFFI.xcframework

echo "XCFramework at $(pwd)/JovaCoreFFI.xcframework"
