#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/macos_toolchain.sh"

APP_NAME="MarkHola"
APP_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
DIST_DIR="$ROOT_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
ICONSET_DIR="$DIST_DIR/$APP_NAME.icon-build"
ICNS_PATH="$RESOURCES_DIR/$APP_NAME.icns"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"
BUILD_MODE="host"

if [[ "${1:-}" == "--universal" ]]; then
  BUILD_MODE="universal"
  shift
fi

if [[ "$#" -ne 0 ]]; then
  print -u2 -- "Usage: $0 [--universal]"
  exit 1
fi

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    print -u2 -- "Missing required command: $command_name"
    exit 1
  fi
}

render_icon() {
  local size="$1"
  local output="$2"
  sips -z "$size" "$size" "$ROOT_DIR/assets/app-icon.png" --out "$output" >/dev/null
}

build_thin_binary() {
  local target="$1"
  print -r -- "==> Building ${target} release binary"
  markhola_cargo build \
    --release \
    --locked \
    --manifest-path "$ROOT_DIR/Cargo.toml" \
    --target "$target" \
    --bin markhola
}

assemble_executable() {
  local host_target
  host_target="$("$MARKHOLA_RUSTC_BIN" -Vv | sed -n 's/^host: //p')"

  if [[ "$BUILD_MODE" == "universal" ]]; then
    local target
    for target in "${MARKHOLA_MACOS_TARGETS[@]}"; do
      build_thin_binary "$target"
    done

    print -r -- "==> Creating Universal 2 executable"
    lipo -create \
      "$ROOT_DIR/target/aarch64-apple-darwin/release/markhola" \
      "$ROOT_DIR/target/x86_64-apple-darwin/release/markhola" \
      -output "$MACOS_DIR/$APP_NAME"
  else
    case "$host_target" in
      aarch64-apple-darwin|x86_64-apple-darwin) ;;
      *)
        print -u2 -- "Unsupported macOS Rust host: $host_target"
        exit 1
        ;;
    esac
    build_thin_binary "$host_target"
    cp "$ROOT_DIR/target/$host_target/release/markhola" "$MACOS_DIR/$APP_NAME"
  fi

  chmod +x "$MACOS_DIR/$APP_NAME"
}

render_resources() {
  print -r -- "==> Rendering macOS iconset"
  rm -rf "$ICONSET_DIR"
  mkdir -p "$ICONSET_DIR"

  render_icon 16 "$ICONSET_DIR/icon_16x16.png"
  render_icon 32 "$ICONSET_DIR/icon_32x32.png"
  render_icon 48 "$ICONSET_DIR/icon_48x48.png"
  render_icon 128 "$ICONSET_DIR/icon_128x128.png"
  render_icon 256 "$ICONSET_DIR/icon_256x256.png"
  render_icon 512 "$ICONSET_DIR/icon_512x512.png"
  render_icon 1024 "$ICONSET_DIR/icon_1024x1024.png"

  print -r -- "==> Creating icns"
  markhola_cargo run \
    --locked \
    --manifest-path "$ROOT_DIR/Cargo.toml" \
    --bin make_icns \
    -- \
    "$ICONSET_DIR" \
    "$ICNS_PATH"

  ditto "$ROOT_DIR/themes" "$RESOURCES_DIR/themes"
  ditto "$ROOT_DIR/assets/help" "$RESOURCES_DIR/help"
  cp "$ROOT_DIR/assets/logo.png" "$RESOURCES_DIR/logo.png"
}

write_info_plist() {
  cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.markhola.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    <key>LSSupportsOpeningDocumentsInPlace</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>UTImportedTypeDeclarations</key>
    <array>
      <dict>
        <key>UTTypeIdentifier</key>
        <string>net.daringfireball.markdown</string>
        <key>UTTypeDescription</key>
        <string>Markdown document</string>
        <key>UTTypeConformsTo</key>
        <array>
          <string>public.plain-text</string>
          <string>public.text</string>
          <string>public.data</string>
        </array>
        <key>UTTypeTagSpecification</key>
        <dict>
          <key>public.filename-extension</key>
          <array>
            <string>md</string>
            <string>markdown</string>
          </array>
          <key>public.mime-type</key>
          <array>
            <string>text/markdown</string>
            <string>text/x-markdown</string>
          </array>
        </dict>
      </dict>
    </array>
    <key>CFBundleDocumentTypes</key>
    <array>
      <dict>
        <key>CFBundleTypeName</key>
        <string>Markdown Document</string>
        <key>CFBundleTypeRole</key>
        <string>Editor</string>
        <key>LSHandlerRank</key>
        <string>Owner</string>
        <key>CFBundleTypeExtensions</key>
        <array>
          <string>md</string>
          <string>markdown</string>
        </array>
        <key>CFBundleTypeMIMETypes</key>
        <array>
          <string>text/markdown</string>
          <string>text/x-markdown</string>
        </array>
        <key>LSItemContentTypes</key>
        <array>
          <string>net.daringfireball.markdown</string>
        </array>
      </dict>
    </array>
    <key>NSDocumentsFolderUsageDescription</key>
    <string>MarkHola needs access to your Documents folder to open Markdown files and load referenced local assets (images, diagrams) located alongside your documents.</string>
    <key>NSDesktopFolderUsageDescription</key>
    <string>MarkHola needs access to your Desktop folder to open Markdown files and load referenced local assets (images, diagrams) located alongside your documents.</string>
    <key>NSDownloadsFolderUsageDescription</key>
    <string>MarkHola needs access to your Downloads folder to open Markdown files and load referenced local assets (images, diagrams) located alongside your documents.</string>
    <key>NSHighResolutionCapable</key>
    <true/>
  </dict>
</plist>
PLIST
}

sign_app_bundle() {
  print -r -- "==> Preparing app bundle for signing"
  xattr -cr "$APP_DIR"

  if [[ -n "$CODESIGN_IDENTITY" ]]; then
    print -r -- "==> Signing app bundle with Developer ID"
    codesign \
      --force \
      --deep \
      --options runtime \
      --timestamp \
      --sign "$CODESIGN_IDENTITY" \
      "$APP_DIR"
  else
    print -r -- "==> Signing app bundle with ad-hoc signature"
    codesign --force --deep --sign - "$APP_DIR"
    print -u2 -- "Warning: CODESIGN_IDENTITY is not set; Developer ID signing is skipped."
  fi

  codesign --verify --deep --strict --verbose=2 "$APP_DIR"
}

require_command sips
require_command lipo
require_command ditto
require_command codesign
require_command xattr
require_command xcrun
markhola_prepare_rust_toolchain

mkdir -p "$DIST_DIR"
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

assemble_executable
render_resources
write_info_plist
sign_app_bundle

if [[ "$BUILD_MODE" == "universal" ]]; then
  "$ROOT_DIR/scripts/verify_macos_architectures.sh" --app "$APP_DIR" --universal
else
  "$ROOT_DIR/scripts/verify_macos_architectures.sh" --app "$APP_DIR" --host
fi

print -r -- "==> Done: $APP_DIR (${BUILD_MODE})"
