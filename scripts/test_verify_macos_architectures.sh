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
ARM13="$TEST_ROOT/main-arm64-macos13"
X86_14="$TEST_ROOT/main-x86_64-macos14"
X86_13="$TEST_ROOT/main-x86_64-macos13"

xcrun clang -target arm64-apple-macos14.0 "$FIXTURE_SOURCE" -o "$ARM14"
xcrun clang -target arm64-apple-macos13.0 "$FIXTURE_SOURCE" -o "$ARM13"
xcrun clang -target x86_64-apple-macos14.0 "$FIXTURE_SOURCE" -o "$X86_14"
xcrun clang -target x86_64-apple-macos13.0 "$FIXTURE_SOURCE" -o "$X86_13"

ARM_APP="$TEST_ROOT/Arm.app"
make_app "$ARM_APP" "$ARM14"
sign_app "$ARM_APP"
"$VERIFY_SCRIPT" --app "$ARM_APP" --architecture arm64
print -r -- "PASS: valid arm64-only app"

X86_APP="$TEST_ROOT/Intel.app"
make_app "$X86_APP" "$X86_14"
sign_app "$X86_APP"
"$VERIFY_SCRIPT" --app "$X86_APP" --architecture x86_64
print -r -- "PASS: valid x86_64-only app"

expect_failure wrong-main-architecture \
  "$VERIFY_SCRIPT" --app "$ARM_APP" --architecture x86_64

BAD_MAIN_MINOS_APP="$TEST_ROOT/BadMainMinos.app"
make_app "$BAD_MAIN_MINOS_APP" "$ARM13"
sign_app "$BAD_MAIN_MINOS_APP"
expect_failure main-deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$BAD_MAIN_MINOS_APP" --architecture arm64

BAD_PLIST_APP="$TEST_ROOT/BadPlist.app"
make_app "$BAD_PLIST_APP" "$X86_14"
plutil -replace LSMinimumSystemVersion -string 13.0 "$BAD_PLIST_APP/Contents/Info.plist"
sign_app "$BAD_PLIST_APP"
expect_failure plist-deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$BAD_PLIST_APP" --architecture x86_64

BAD_SIGNATURE_APP="$TEST_ROOT/BadSignature.app"
make_app "$BAD_SIGNATURE_APP" "$ARM14"
sign_app "$BAD_SIGNATURE_APP"
print -n -- "signature mutation" >>"$BAD_SIGNATURE_APP/Contents/MacOS/MarkHola"
expect_failure invalid-signature \
  "$VERIFY_SCRIPT" --app "$BAD_SIGNATURE_APP" --architecture arm64

WRONG_HELPER_APP="$TEST_ROOT/WrongHelper.app"
make_app "$WRONG_HELPER_APP" "$ARM14"
cp "$X86_14" "$WRONG_HELPER_APP/Contents/Resources/wrong-helper"
chmod +x "$WRONG_HELPER_APP/Contents/Resources/wrong-helper"
sign_app "$WRONG_HELPER_APP"
expect_failure helper-architecture-mismatch \
  "$VERIFY_SCRIPT" --app "$WRONG_HELPER_APP" --architecture arm64

BAD_HELPER_MINOS_APP="$TEST_ROOT/BadHelperMinos.app"
make_app "$BAD_HELPER_MINOS_APP" "$X86_14"
cp "$X86_13" "$BAD_HELPER_MINOS_APP/Contents/Resources/bad-minos-helper"
chmod +x "$BAD_HELPER_MINOS_APP/Contents/Resources/bad-minos-helper"
sign_app "$BAD_HELPER_MINOS_APP"
expect_failure helper-deployment-target-mismatch \
  "$VERIFY_SCRIPT" --app "$BAD_HELPER_MINOS_APP" --architecture x86_64

print -r -- "All thin architecture gate tests passed."
