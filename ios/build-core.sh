#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET=16.0
platform="${PLATFORM_NAME:-iphoneos}"
if [ "$platform" = iphonesimulator ]; then
  if [ "${NATIVE_ARCH_ACTUAL:-arm64}" = x86_64 ]; then target=x86_64-apple-ios; else target=aarch64-apple-ios-sim; fi
else
  target=aarch64-apple-ios
fi
rustup target add "$target"
cargo build --locked --lib --release --no-default-features --features route-ios --target "$target"
mkdir -p "ios/Native/$platform"
cp "target/$target/release/libsportssimulation.a" "ios/Native/$platform/libsportssimulation.a"
