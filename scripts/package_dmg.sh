#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/macos_toolchain.sh"
APP_NAME="MarkHola"
APP_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
DIST_DIR="$ROOT_DIR/dist"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"
NOTARY_PROFILE="${NOTARY_PROFILE:-}"

AS_LABEL="apple-silicon"
AS_TARGET="aarch64-apple-darwin"
AS_ARCH="arm64"
AS_APP="$DIST_DIR/MarkHola-$AS_LABEL.app"
AS_DMG="$DIST_DIR/MarkHola-$APP_VERSION-$AS_LABEL.dmg"

INTEL_LABEL="intel"
INTEL_TARGET="x86_64-apple-darwin"
INTEL_ARCH="x86_64"
INTEL_APP="$DIST_DIR/MarkHola-$INTEL_LABEL.app"
INTEL_DMG="$DIST_DIR/MarkHola-$APP_VERSION-$INTEL_LABEL.dmg"

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    print -u2 -- "Missing required command: $command_name"
    exit 1
  fi
}

verify_clean_source() {
  if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    PAIR_SOURCE_COMMIT="unavailable"
    return
  fi

  if [[ -n "$(git -C "$ROOT_DIR" status --porcelain --untracked-files=normal)" ]]; then
    print -u2 -- "Tracked source must be clean before paired release packaging."
    exit 1
  fi
  PAIR_SOURCE_COMMIT="$(git -C "$ROOT_DIR" rev-parse HEAD)"
}

build_thin_app() {
  local target="$1"
  local architecture="$2"
  local app_path="$3"

  "$ROOT_DIR/scripts/build_app.sh" \
    --target "$target" \
    --app "$app_path"
  "$ROOT_DIR/scripts/verify_macos_architectures.sh" \
    --app "$app_path" \
    --architecture "$architecture"
}

write_resource_manifest() {
  local app_path="$1"
  local output_path="$2"

  (
    cd "$app_path"
    {
      shasum -a 256 "Contents/Info.plist"
      find "Contents/Resources" -type f -print0 \
        | sort -z \
        | xargs -0 shasum -a 256
    }
  ) >"$output_path"
}

verify_resource_parity() {
  local as_manifest="$DIST_DIR/MarkHola-$AS_LABEL.resources.sha256"
  local intel_manifest="$DIST_DIR/MarkHola-$INTEL_LABEL.resources.sha256"

  write_resource_manifest "$AS_APP" "$as_manifest"
  write_resource_manifest "$INTEL_APP" "$intel_manifest"
  if ! cmp -s "$as_manifest" "$intel_manifest"; then
    print -u2 -- "Architecture-specific App resources do not match."
    diff -u "$as_manifest" "$intel_manifest" >&2 || true
    exit 1
  fi
  PAIR_RESOURCE_SHA="$(shasum -a 256 "$as_manifest" | awk '{print $1}')"
  print -r -- "==> Resource parity verified: $PAIR_RESOURCE_SHA"
}

create_dmg() {
  local label="$1"
  local app_path="$2"
  local dmg_path="$3"
  local dmg_root="$DIST_DIR/dmg-root-$label"

  print -r -- "==> Preparing $label DMG root"
  rm -rf "$dmg_root"
  mkdir -p "$dmg_root"
  ditto "$app_path" "$dmg_root/$APP_NAME.app"
  ln -s /Applications "$dmg_root/Applications"

  print -r -- "==> Creating compressed UDZO DMG: ${dmg_path:t}"
  rm -f "$dmg_path"
  hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$dmg_root" \
    -ov \
    -format UDZO \
    "$dmg_path"
  xattr -cr "$dmg_path"
}

sign_and_notarize_dmg() {
  local dmg_path="$1"

  if [[ -z "$CODESIGN_IDENTITY" ]]; then
    print -u2 -- "Warning: CODESIGN_IDENTITY is not set; DMG signing is skipped for ${dmg_path:t}."
    if [[ -n "$NOTARY_PROFILE" ]]; then
      print -u2 -- "Warning: NOTARY_PROFILE is ignored without CODESIGN_IDENTITY."
    fi
    return
  fi

  print -r -- "==> Signing ${dmg_path:t}"
  codesign --force --timestamp --sign "$CODESIGN_IDENTITY" "$dmg_path"
  codesign --verify --verbose=2 "$dmg_path"

  if [[ -z "$NOTARY_PROFILE" ]]; then
    print -u2 -- "Warning: NOTARY_PROFILE is not set; notarization is skipped for ${dmg_path:t}."
    return
  fi

  print -r -- "==> Notarizing ${dmg_path:t}"
  xcrun notarytool submit "$dmg_path" \
    --keychain-profile "$NOTARY_PROFILE" \
    --wait
  xcrun stapler staple "$dmg_path"
  xcrun stapler validate "$dmg_path"
}

write_asset_manifest() {
  local label="$1"
  local target="$2"
  local architecture="$3"
  local app_path="$4"
  local dmg_path="$5"
  local output_path="$DIST_DIR/MarkHola-$APP_VERSION-$label.manifest.txt"
  local dmg_sha
  local dmg_size
  local dmg_mtime
  local lock_sha
  local app_signature
  local dmg_signature
  local notary_state

  dmg_sha="$(shasum -a 256 "$dmg_path" | awk '{print $1}')"
  dmg_size="$(stat -f '%z' "$dmg_path")"
  dmg_mtime="$(stat -f '%m' "$dmg_path")"
  lock_sha="$(shasum -a 256 "$ROOT_DIR/Cargo.lock" | awk '{print $1}')"
  app_signature="$(codesign -dv --verbose=4 "$app_path" 2>&1 \
    | awk -F= '$1 == "Signature" {print $2; exit}')"
  if [[ -n "$CODESIGN_IDENTITY" ]]; then
    dmg_signature="signed"
  else
    dmg_signature="unsigned"
  fi
  if [[ -n "$CODESIGN_IDENTITY" && -n "$NOTARY_PROFILE" ]]; then
    notary_state="submitted-stapled-validated"
  else
    notary_state="not-configured"
  fi

  {
    print -r -- "asset_name=${dmg_path:t}"
    print -r -- "asset_path=$dmg_path"
    print -r -- "architecture=$architecture"
    print -r -- "rust_target=$target"
    print -r -- "source_commit=$PAIR_SOURCE_COMMIT"
    print -r -- "cargo_lock_sha256=$lock_sha"
    print -r -- "rust_toolchain=$MARKHOLA_RUST_TOOLCHAIN"
    print -r -- "rustc_version=$("$MARKHOLA_RUSTC_BIN" --version)"
    print -r -- "cargo_version=$("$MARKHOLA_CARGO_BIN" --version)"
    print -r -- "release_profile=release"
    print -r -- "macosx_deployment_target=$MARKHOLA_MACOS_DEPLOYMENT_TARGET"
    print -r -- "bundle_version=$APP_VERSION"
    print -r -- "bundle_identifier=com.markhola.app"
    print -r -- "resource_manifest_sha256=$PAIR_RESOURCE_SHA"
    print -r -- "dmg_format=UDZO"
    print -r -- "dmg_size=$dmg_size"
    print -r -- "dmg_mtime_epoch=$dmg_mtime"
    print -r -- "dmg_sha256=$dmg_sha"
    print -r -- "app_signature=${app_signature:-unknown}"
    print -r -- "dmg_signature=$dmg_signature"
    print -r -- "notary_state=$notary_state"
  } >"$output_path"
}

verify_pair_inputs_unchanged() {
  if [[ "$PAIR_SOURCE_COMMIT" == "unavailable" ]]; then
    return
  fi
  if [[ "$(git -C "$ROOT_DIR" rev-parse HEAD)" != "$PAIR_SOURCE_COMMIT" ]] \
    || [[ -n "$(git -C "$ROOT_DIR" status --porcelain --untracked-files=normal)" ]]; then
    print -u2 -- "Source inputs changed while building the paired assets; both outputs are invalid."
    exit 1
  fi
}

require_command codesign
require_command cmp
require_command ditto
require_command git
require_command hdiutil
require_command shasum
require_command stat
require_command xargs
require_command xattr
if [[ -n "$NOTARY_PROFILE" ]]; then
  require_command xcrun
fi

verify_clean_source
markhola_prepare_rust_toolchain
mkdir -p "$DIST_DIR"

build_thin_app "$AS_TARGET" "$AS_ARCH" "$AS_APP"
build_thin_app "$INTEL_TARGET" "$INTEL_ARCH" "$INTEL_APP"
verify_pair_inputs_unchanged
verify_resource_parity

create_dmg "$AS_LABEL" "$AS_APP" "$AS_DMG"
sign_and_notarize_dmg "$AS_DMG"
hdiutil verify "$AS_DMG"

create_dmg "$INTEL_LABEL" "$INTEL_APP" "$INTEL_DMG"
sign_and_notarize_dmg "$INTEL_DMG"
hdiutil verify "$INTEL_DMG"

verify_pair_inputs_unchanged
write_asset_manifest "$AS_LABEL" "$AS_TARGET" "$AS_ARCH" "$AS_APP" "$AS_DMG"
write_asset_manifest "$INTEL_LABEL" "$INTEL_TARGET" "$INTEL_ARCH" "$INTEL_APP" "$INTEL_DMG"

print -r -- "==> Paired architecture-specific DMGs complete"
print -r -- "Apple Silicon: $AS_DMG"
print -r -- "Intel: $INTEL_DMG"
