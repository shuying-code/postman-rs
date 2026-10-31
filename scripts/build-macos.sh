#!/usr/bin/env bash
# Build macOS binaries (Apple Silicon + Intel -> Universal). MUST run on macOS.
set -euo pipefail
cd "$(dirname "$0")/.."

NAME=pmrs
mkdir -p dist

echo ">> adding targets"
rustup target add aarch64-apple-darwin x86_64-apple-darwin

echo ">> building arm64"
cargo build --release --target aarch64-apple-darwin
echo ">> building x86_64"
cargo build --release --target x86_64-apple-darwin

echo ">> creating universal binary"
lipo -create \
  -output "dist/${NAME}-macos-universal" \
  "target/aarch64-apple-darwin/release/${NAME}" \
  "target/x86_64-apple-darwin/release/${NAME}"
chmod +x "dist/${NAME}-macos-universal"

# Ad-hoc sign so Gatekeeper is less grumpy on the build machine (harmless if skipped).
codesign --force --sign - "dist/${NAME}-macos-universal" 2>/dev/null || true

echo ">> creating universal GUI binary"
lipo -create \
  -output "dist/${NAME}-gui-macos-universal" \
  "target/aarch64-apple-darwin/release/${NAME}-gui" \
  "target/x86_64-apple-darwin/release/${NAME}-gui"
chmod +x "dist/${NAME}-gui-macos-universal"
codesign --force --sign - "dist/${NAME}-gui-macos-universal" 2>/dev/null || true

echo ">> done: dist/${NAME}-macos-universal and dist/${NAME}-gui-macos-universal"
lipo -info "dist/${NAME}-macos-universal"
lipo -info "dist/${NAME}-gui-macos-universal"
