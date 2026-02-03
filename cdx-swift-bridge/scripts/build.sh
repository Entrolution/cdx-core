#!/bin/bash
# Build script for cdx-swift-bridge
# Generates Swift bindings and XCFramework from the Rust library
#
# Usage:
#   ./cdx-swift-bridge/scripts/build.sh [output-dir]
#
# The output directory defaults to cdx-swift-bridge/output/

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BRIDGE_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${1:-$BRIDGE_DIR/output}"

echo "Building cdx-swift-bridge..."

cd "$BRIDGE_DIR"

# Build for both architectures
echo "Building for arm64..."
cargo build --release --target aarch64-apple-darwin

echo "Building for x86_64..."
cargo build --release --target x86_64-apple-darwin

# Generate Swift bindings (use the aarch64 dylib for binding generation)
echo "Generating Swift bindings..."
cargo run --bin uniffi-bindgen -- generate \
    --library target/aarch64-apple-darwin/release/libcdx_swift_bridge.dylib \
    --language swift \
    --out-dir ./generated

# Create universal binary
echo "Creating universal binary..."
mkdir -p target/universal-macos/release
lipo -create \
    target/aarch64-apple-darwin/release/libcdx_swift_bridge.a \
    target/x86_64-apple-darwin/release/libcdx_swift_bridge.a \
    -output target/universal-macos/release/libcdx_swift_bridge.a

# Create XCFramework structure
echo "Creating XCFramework..."
XCFRAMEWORK_DIR="$OUTPUT_DIR/CdxSwiftBridgeFFI.xcframework"
rm -rf "$XCFRAMEWORK_DIR"
mkdir -p "$XCFRAMEWORK_DIR/macos-arm64_x86_64/Headers"

cp generated/CdxSwiftBridgeFFI.h "$XCFRAMEWORK_DIR/macos-arm64_x86_64/Headers/"
cp generated/CdxSwiftBridgeFFI.modulemap "$XCFRAMEWORK_DIR/macos-arm64_x86_64/Headers/module.modulemap"
cp target/universal-macos/release/libcdx_swift_bridge.a "$XCFRAMEWORK_DIR/macos-arm64_x86_64/"

# Create Info.plist
cat > "$XCFRAMEWORK_DIR/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AvailableLibraries</key>
    <array>
        <dict>
            <key>HeadersPath</key>
            <string>Headers</string>
            <key>LibraryIdentifier</key>
            <string>macos-arm64_x86_64</string>
            <key>LibraryPath</key>
            <string>libcdx_swift_bridge.a</string>
            <key>SupportedArchitectures</key>
            <array>
                <string>arm64</string>
                <string>x86_64</string>
            </array>
            <key>SupportedPlatform</key>
            <string>macos</string>
        </dict>
    </array>
    <key>CFBundlePackageType</key>
    <string>XFWK</string>
    <key>XCFrameworkFormatVersion</key>
    <string>1.0</string>
</dict>
</plist>
EOF

# Copy Swift bindings to output
mkdir -p "$OUTPUT_DIR"
cp generated/CdxSwiftBridge.swift "$OUTPUT_DIR/"

echo "Done! XCFramework created at: $XCFRAMEWORK_DIR"
echo "Swift bindings at: $OUTPUT_DIR/CdxSwiftBridge.swift"
