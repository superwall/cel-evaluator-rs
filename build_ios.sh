#!/bin/bash
set -e

echo "Building the rust library for binding generation."
cargo build --lib


echo "Generating Swift bindings in .target/ios/"
# Generate bindings
cargo run --features=uniffi/cli \
     --bin uniffi-bindgen \
     generate src/cel.udl \
     --language swift \
     --out-dir ./target/ios/

# Add the iOS targets and build
for TARGET in \
        aarch64-apple-darwin \
        aarch64-apple-ios \
        aarch64-apple-ios-sim \
        x86_64-apple-darwin \
        x86_64-apple-ios

do
    rustup target add $TARGET
    cargo build --target=$TARGET --lib --release
done


# Rename *.modulemap to module.modulemap
mv ./target/ios/celFFI.modulemap ./target/ios/module.modulemap

rm -rf target/ios/ios.xcframework
rm -rf target/ios/macos.xcframework

mkdir -p ./target/ios-sim/release/
mkdir -p ./target/ios/release/
mkdir -p ./target/macos/release/

# Clean up previous build artifacts
rm -rf ./target/xcframeworks

# Create directories for headers
mkdir -p ./target/xcframeworks/headers/ios-simulator
mkdir -p ./target/xcframeworks/headers/ios-device
mkdir -p ./target/xcframeworks/headers/macos

# Copy headers
cp -R ./target/ios/* ./target/xcframeworks/headers/ios-simulator/
cp -R ./target/ios/* ./target/xcframeworks/headers/ios-device/
cp -R ./target/ios/* ./target/xcframeworks/headers/macos/

# iOS Simulator (combined arm64 and x86_64)
echo "Preparing iOS Simulator library (universal binary)"
mkdir -p ./target/xcframeworks/ios-simulator
lipo -create ./target/aarch64-apple-ios-sim/release/libcel_eval.a \
    ./target/x86_64-apple-ios/release/libcel_eval.a \
    -output ./target/xcframeworks/ios-simulator/libcel.a

# iOS Device (arm64)
echo "Preparing iOS Device (arm64) library"
mkdir -p ./target/xcframeworks/ios-device
cp ./target/aarch64-apple-ios/release/libcel_eval.a ./target/xcframeworks/ios-device/libcel.a

# macOS (combined arm64 and x86_64)
echo "Preparing macOS library (universal binary)"
mkdir -p ./target/xcframeworks/macos
lipo -create ./target/aarch64-apple-darwin/release/libcel_eval.a \
    ./target/x86_64-apple-darwin/release/libcel_eval.a \
    -output ./target/xcframeworks/macos/libcel.a

echo "Building XCFramework"
xcodebuild -create-xcframework \
    -library ./target/xcframeworks/ios-simulator/libcel.a -headers ./target/xcframeworks/headers/ios-simulator \
    -library ./target/xcframeworks/ios-device/libcel.a -headers ./target/xcframeworks/headers/ios-device \
    -library ./target/xcframeworks/macos/libcel.a -headers ./target/xcframeworks/headers/macos \
    -output ./target/xcframeworks/libcel.xcframework

echo "XCFramework built at ./target/xcframeworks/libcel.xcframework"

