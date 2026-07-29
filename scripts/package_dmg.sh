#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="MarkHola"
APP_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
DIST_DIR="$ROOT_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
DMG_ROOT="$DIST_DIR/dmg-root"
DMG_PATH="$DIST_DIR/${APP_NAME}-${APP_VERSION}.dmg"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"
NOTARY_PROFILE="${NOTARY_PROFILE:-}"

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    print -u2 -- "Missing required command: $command_name"
    exit 1
  fi
}

create_dmg() {
  print -r -- "==> Preparing DMG root"
  rm -rf "$DMG_ROOT"
  mkdir -p "$DMG_ROOT"
  ditto "$APP_DIR" "$DMG_ROOT/$APP_NAME.app"
  ln -s /Applications "$DMG_ROOT/Applications"

  print -r -- "==> Creating compressed UDZO DMG"
  rm -f "$DMG_PATH"
  hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$DMG_ROOT" \
    -ov \
    -format UDZO \
    "$DMG_PATH"
  xattr -cr "$DMG_PATH"
}

sign_and_notarize_dmg() {
  if [[ -z "$CODESIGN_IDENTITY" ]]; then
    print -u2 -- "Warning: CODESIGN_IDENTITY is not set; DMG signing is skipped."
    if [[ -n "$NOTARY_PROFILE" ]]; then
      print -u2 -- "Warning: NOTARY_PROFILE is ignored without CODESIGN_IDENTITY."
    fi
    return
  fi

  print -r -- "==> Signing disk image"
  codesign \
    --force \
    --timestamp \
    --sign "$CODESIGN_IDENTITY" \
    "$DMG_PATH"
  codesign --verify --verbose=2 "$DMG_PATH"

  if [[ -z "$NOTARY_PROFILE" ]]; then
    print -u2 -- "Warning: NOTARY_PROFILE is not set; notarization is skipped."
    return
  fi

  print -r -- "==> Notarizing disk image"
  xcrun notarytool submit "$DMG_PATH" \
    --keychain-profile "$NOTARY_PROFILE" \
    --wait

  print -r -- "==> Stapling notarization ticket"
  xcrun stapler staple "$DMG_PATH"
  xcrun stapler validate "$DMG_PATH"
}

require_command codesign
require_command ditto
require_command hdiutil
require_command xattr
if [[ -n "$NOTARY_PROFILE" ]]; then
  require_command xcrun
fi

"$ROOT_DIR/scripts/build_app.sh" --universal
"$ROOT_DIR/scripts/verify_macos_architectures.sh" --app "$APP_DIR" --universal
create_dmg
sign_and_notarize_dmg

print -r -- "==> Done"
print -r -- "App bundle: $APP_DIR"
print -r -- "Disk image: $DMG_PATH"
