#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

LIB_NAME=libjova_spike.a
HEADERS_DIR=generated/swift

mkdir -p target/mac-universal
lipo -create \
  target/aarch64-apple-darwin/release/$LIB_NAME \
  target/x86_64-apple-darwin/release/$LIB_NAME \
  -output target/mac-universal/$LIB_NAME

rm -rf generated/JovaSpikeFFI.xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/$LIB_NAME      -headers $HEADERS_DIR \
  -library target/aarch64-apple-ios-sim/release/$LIB_NAME  -headers $HEADERS_DIR \
  -library target/mac-universal/$LIB_NAME                  -headers $HEADERS_DIR \
  -output generated/JovaSpikeFFI.xcframework

echo "[ok] XCFramework built at generated/JovaSpikeFFI.xcframework"
