#!/usr/bin/env bash
# Builds astro-ffi for iOS, generates the Swift bindings with UniFFI and packages the static
# libraries as build/ios/AstroCore.xcframework. Drag it into the Xcode (SwiftUI) project together
# with build/ios/AstroCore.swift.
#
# Targets: aarch64-apple-ios (devices) and aarch64-apple-ios-sim (Simulator on Apple Silicon).
# Requires Xcode (for xcodebuild) and the Rust targets listed in rust-toolchain.toml.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

LIB="libastro_ffi.a"
DEVICE_TARGET="aarch64-apple-ios"
SIMULATOR_TARGET="aarch64-apple-ios-sim"
OUT="$ROOT/build/ios"
STAGING="$OUT/staging"

if ! xcodebuild -version >/dev/null 2>&1; then
  echo "error: xcodebuild is unavailable. Install Xcode, then run:" >&2
  echo "  sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer" >&2
  exit 1
fi

rm -rf "$OUT"
mkdir -p "$STAGING/headers"

# 1. Static libraries only: the app links them, so iOS never needs the cdylib.
for target in "$DEVICE_TARGET" "$SIMULATOR_TARGET"; do
  cargo rustc --package astro-ffi --lib --release --target "$target" --crate-type staticlib
done

# 2. Swift bindings. Library mode reads the UniFFI metadata compiled into the archive and
#    crates/astro-ffi/uniffi.toml (Swift module AstroCore, C module AstroCoreFFI).
cargo run --quiet --package astro-ffi --features bindgen --bin uniffi-bindgen -- generate \
  --library "target/$DEVICE_TARGET/release/$LIB" \
  --language swift \
  --out-dir "$STAGING/bindings"

cp "$STAGING/bindings/AstroCoreFFI.h" "$STAGING/headers/"
cp "$STAGING/bindings/AstroCoreFFI.modulemap" "$STAGING/headers/module.modulemap"

# 3. One XCFramework slice per platform, with the Swift sources beside it.
xcodebuild -create-xcframework \
  -library "target/$DEVICE_TARGET/release/$LIB" -headers "$STAGING/headers" \
  -library "target/$SIMULATOR_TARGET/release/$LIB" -headers "$STAGING/headers" \
  -output "$OUT/AstroCore.xcframework"
cp "$STAGING/bindings/AstroCore.swift" "$OUT/"
rm -rf "$STAGING"

echo "Drag both into the Xcode project:"
echo "  $OUT/AstroCore.xcframework"
echo "  $OUT/AstroCore.swift"
