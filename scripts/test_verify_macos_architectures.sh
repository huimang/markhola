#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
VERIFY_SCRIPT="$ROOT_DIR/scripts/verify_macos_architectures.sh"
FIXTURE_SOURCE="$ROOT_DIR/scripts/fixtures/minimal_macos_main.c"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/markhola-architecture-gate.XXXXXX")"

trap 'rm -rf "$TEST_ROOT"' EXIT

make_plist() {
  local app_dir="$1"

  plutil -create xml1 "$app_dir/Contents/Info.plist"
  plutil -insert CFBundleExecutable -string MarkHola "$app_dir/Contents/Info.plist"
  plutil -insert CFBundleIdentifier -string com.markhola.architecture-gate-test \
    "$app_dir/Contents/Info.plist"
  plutil -insert CFBundlePackageType -string APPL "$app_dir/Contents/Info.plist"
  plutil -insert LSMinimumSystemVersion -string 14.0 "$app_dir/Contents/Info.plist"
}

make_app() {
  local app_dir="$1"
  local executable="$2"

  mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
  cp "$executable" "$app_dir/Contents/MacOS/MarkHola"
  chmod +x "$app_dir/Contents/MacOS/MarkHola"
  make_plist "$app_dir"
}

sign_app() {
  codesign --force --deep --sign - "$1"
}

expect_failure() {
  local name="$1"
  shift

  if "$@" >"$TEST_ROOT/${name}.stdout" 2>"$TEST_ROOT/${name}.stderr"; then
    print -u2 -- "Expected architecture gate failure: $name"
    return 1
  fi

  print -r -- "PASS (expected failure): $name"
}

ARM14="$TEST_ROOT/main-arm64-macos14"
X86_14="$TEST_ROOT/main-x86_64-macos14"
X86_13="$TEST_ROOT/main-x86_64-macos13"
ARM13="$TEST_ROOT/main-arm64-macos13"
UNIVERSAL14="$TEST_ROOT/main-universal-macos14"
UNIVERSAL_MIXED="$TEST_ROOT/main-universal-mixed-minos"
HELPER_BAD_MINOS="$TEST_ROOT/helper-universal-mixed-minos"

xcrun clang -target arm64-apple-macos14.0 "$FIXTURE_SOURCE" -o "$ARM14"
xcrun clang -target x86_64-apple-macos14.0 "$FIXTURE_SOURCE" -o "$X86_14"
xcrun clang -target x86_64-apple-macos13.0 "$FIXTURE_SOURCE" -o "$X86_13"
xcrun clang -target arm64-apple-macos13.0 "$FIXTURE_SOURCE" -o "$ARM13"
lipo -create "$ARM14" "$X86_14" -output "$UNIVERSAL14"
lipo -create "$ARM14" "$X86_13" -output "$UNIVERSAL_MIXED"
lipo -create "$ARM13" "$X86_14" -output "$HELPER_BAD_MINOS"

HOST_ARM_APP="$TEST_ROOT/HostArm.app"
make_app "$HOST_ARM_APP" "$ARM14"
sign_app "$HOST_ARM_APP"
"$VERIFY_SCRIPT" --app "$HOST_ARM_APP" --host
print -r -- "PASS: valid host arm64 app"

HOST_X86_APP="$TEST_ROOT/HostX86.app"
make_app "$HOST_X86_APP" "$X86_14"
sign_app "$HOST_X86_APP"
"$VERIFY_SCRIPT" --app "$HOST_X86_APP" --host
print -r -- "PASS: valid host x86_64 app"

SUCCESS_APP="$TEST_ROOT/Success.app"
make_app "$SUCCESS_APP" "$UNIVERSAL14"
sign_app "$SUCCESS_APP"
"$VERIFY_SCRIPT" --app "$SUCCESS_APP" --universal
print -r -- "PASS: valid Universal 2 app"

MISSING_SLICE_APP="$TEST_ROOT/MissingSlice.app"
make_app "$MISSING_SLICE_APP" "$ARM14"
sign_app "$MISSING_SLICE_APP"
expect_failure missing-slice \
  "$VERIFY_SCRIPT" --app "$MISSING_SLICE_APP" --universal

BAD_MINOS_APP="$TEST_ROOT/BadDeploymentTarget.app"
make_app "$BAD_MINOS_APP" "$UNIVERSAL_MIXED"
sign_app "$BAD_MINOS_APP"
expect_failure deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$BAD_MINOS_APP" --universal

BAD_PLIST_MINOS_APP="$TEST_ROOT/BadPlistDeploymentTarget.app"
make_app "$BAD_PLIST_MINOS_APP" "$UNIVERSAL14"
plutil -replace LSMinimumSystemVersion -string 13.0 \
  "$BAD_PLIST_MINOS_APP/Contents/Info.plist"
sign_app "$BAD_PLIST_MINOS_APP"
expect_failure plist-deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$BAD_PLIST_MINOS_APP" --universal

BAD_SIGNATURE_APP="$TEST_ROOT/BadSignature.app"
make_app "$BAD_SIGNATURE_APP" "$UNIVERSAL14"
sign_app "$BAD_SIGNATURE_APP"
print -n -- "signature mutation" >>"$BAD_SIGNATURE_APP/Contents/MacOS/MarkHola"
expect_failure invalid-signature \
  "$VERIFY_SCRIPT" --app "$BAD_SIGNATURE_APP" --universal

THIN_HELPER_APP="$TEST_ROOT/ThinHelper.app"
make_app "$THIN_HELPER_APP" "$UNIVERSAL14"
cp "$ARM14" "$THIN_HELPER_APP/Contents/Resources/thin-helper"
chmod +x "$THIN_HELPER_APP/Contents/Resources/thin-helper"
sign_app "$THIN_HELPER_APP"
expect_failure single-architecture-helper \
  "$VERIFY_SCRIPT" --app "$THIN_HELPER_APP" --universal

HELPER_BAD_MINOS_APP="$TEST_ROOT/HelperBadDeploymentTarget.app"
make_app "$HELPER_BAD_MINOS_APP" "$UNIVERSAL14"
cp "$HELPER_BAD_MINOS" "$HELPER_BAD_MINOS_APP/Contents/Resources/helper-bad-minos"
chmod +x "$HELPER_BAD_MINOS_APP/Contents/Resources/helper-bad-minos"
sign_app "$HELPER_BAD_MINOS_APP"
expect_failure helper-deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$HELPER_BAD_MINOS_APP" --universal

print -r -- "All architecture gate tests passed."
