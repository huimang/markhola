#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/macos_toolchain.sh"

APP_NAME="MarkHola"
APP_VERSION="$(sed -n 's/^version = \"\(.*\)\"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
DIST_DIR="$ROOT_DIR/dist"
AS_APP="$DIST_DIR/MarkHola-apple-silicon.app"
INTEL_APP="$DIST_DIR/MarkHola-intel.app"
MAS_APP="$DIST_DIR/$APP_NAME.app"
PKG_PATH="$DIST_DIR/$APP_NAME-$APP_VERSION-macOS.pkg"
ENTITLEMENTS="$ROOT_DIR/entitlements.mas.plist"

SIGNING_IDENTITY="${SIGNING_IDENTITY:-}"
INSTALLER_IDENTITY="${INSTALLER_IDENTITY:-}"
PROVISIONING_PROFILE="${PROVISIONING_PROFILE:-}"

die() {
  print -u2 -- "$1"
  exit 1
}

[[ -n "$SIGNING_IDENTITY" ]] || die "Set SIGNING_IDENTITY to the Mac App Store application signing identity."
[[ -n "$INSTALLER_IDENTITY" ]] || die "Set INSTALLER_IDENTITY to the Mac App Store installer signing identity."
[[ -f "$PROVISIONING_PROFILE" ]] || die "Set PROVISIONING_PROFILE to the App Store provisioning profile."
[[ -f "$ENTITLEMENTS" ]] || die "Missing $ENTITLEMENTS"

"$ROOT_DIR/scripts/build_app.sh" \
  --target aarch64-apple-darwin \
  --app "$AS_APP"
"$ROOT_DIR/scripts/build_app.sh" \
  --target x86_64-apple-darwin \
  --app "$INTEL_APP"

rm -rf "$MAS_APP"
ditto "$AS_APP" "$MAS_APP"
lipo -create \
  "$AS_APP/Contents/MacOS/$APP_NAME" \
  "$INTEL_APP/Contents/MacOS/$APP_NAME" \
  -output "$MAS_APP/Contents/MacOS/$APP_NAME"

cp "$PROVISIONING_PROFILE" "$MAS_APP/Contents/embedded.provisionprofile"
xattr -cr "$MAS_APP"
codesign --force --deep --options runtime --timestamp \
  --entitlements "$ENTITLEMENTS" \
  --sign "$SIGNING_IDENTITY" \
  "$MAS_APP"
codesign --verify --deep --strict --verbose=2 "$MAS_APP"

rm -f "$PKG_PATH"
productbuild \
  --component "$MAS_APP" /Applications \
  --sign "$INSTALLER_IDENTITY" \
  "$PKG_PATH"

pkgutil --check-signature "$PKG_PATH"
print -r -- "==> App Store package ready: $PKG_PATH"
