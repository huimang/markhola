#!/bin/zsh

set -euo pipefail
export PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/markhola-release-logic.XXXXXX")"

trap 'rm -rf "$TEST_ROOT"' EXIT

fail() {
  print -u2 -- "$1"
  exit 1
}

assert_file_contains() {
  local path="$1"
  local needle="$2"

  if ! /usr/bin/grep -Fq -- "$needle" "$path"; then
    print -u2 -- "Expected to find '$needle' in $path"
    print -u2 -- "--- $path ---"
    /usr/bin/sed -n '1,200p' "$path" >&2
    exit 1
  fi
}

setup_package_repo() {
  local repo_root="$1"
  mkdir -p "$repo_root/scripts" "$repo_root/dist" "$repo_root/assets"
  cp "$ROOT_DIR/scripts/package_dmg.sh" "$repo_root/scripts/package_dmg.sh"
  cat >"$repo_root/scripts/macos_toolchain.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
MARKHOLA_RUST_TOOLCHAIN="1.95.0"
MARKHOLA_MACOS_DEPLOYMENT_TARGET="14.0"
markhola_prepare_rust_toolchain() {
  MARKHOLA_RUSTC_BIN="$MARKHOLA_MOCK_BIN/mock-rustc"
  MARKHOLA_CARGO_BIN="$MARKHOLA_MOCK_BIN/mock-cargo"
}
EOF
  cat >"$repo_root/Cargo.toml" <<'EOF'
[package]
name = "markhola"
version = "0.9.0"
edition = "2024"
EOF
  print -n -- "lockfile" >"$repo_root/Cargo.lock"
  : >"$repo_root/assets/app-icon.png"
}

setup_package_mocks() {
  local repo_root="$1"
  local mock_bin="$repo_root/mock-bin"
  local log_path="$repo_root/mock.log"

  mkdir -p "$mock_bin"

  cat >"$mock_bin/mock-rustc" <<'EOF'
#!/bin/zsh
print -r -- "rustc 1.95.0 (mock)"
EOF
  chmod +x "$mock_bin/mock-rustc"

  cat >"$mock_bin/mock-cargo" <<'EOF'
#!/bin/zsh
print -r -- "cargo 1.95.0 (mock)"
EOF
  chmod +x "$mock_bin/mock-cargo"

  cat >"$repo_root/scripts/build_app.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
print -r -- "build_app:$*" >>"$ROOT_DIR/mock.log"
app_path=""
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --target)
      shift 2
      ;;
    --app)
      app_path="$2"
      shift 2
      ;;
  esac
done
mkdir -p "$app_path/Contents/MacOS" "$app_path/Contents/Resources/help"
print -n -- "binary" >"$app_path/Contents/MacOS/MarkHola"
print -n -- "plist" >"$app_path/Contents/Info.plist"
print -n -- "help" >"$app_path/Contents/Resources/help/Documentation.md"
EOF
  chmod +x "$repo_root/scripts/build_app.sh"

  cat >"$repo_root/scripts/verify_macos_architectures.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
print -r -- "verify:$*" >>"$ROOT_DIR/mock.log"
EOF
  chmod +x "$repo_root/scripts/verify_macos_architectures.sh"

  cat >"$mock_bin/ditto" <<'EOF'
#!/bin/zsh
set -euo pipefail
src="$1"
dst="$2"
rm -rf "$dst"
cp -R "$src" "$dst"
EOF
  chmod +x "$mock_bin/ditto"

  cat >"$mock_bin/hdiutil" <<'EOF'
#!/bin/zsh
set -euo pipefail
print -r -- "hdiutil:$*" >>"$MARKHOLA_MOCK_LOG"
touch "${@: -1}"
EOF
  chmod +x "$mock_bin/hdiutil"

  cat >"$mock_bin/xattr" <<'EOF'
#!/bin/zsh
exit 0
EOF
  chmod +x "$mock_bin/xattr"

  cat >"$mock_bin/codesign" <<'EOF'
#!/bin/zsh
set -euo pipefail
print -r -- "codesign:$*" >>"$MARKHOLA_MOCK_LOG"
EOF
  chmod +x "$mock_bin/codesign"

  cat >"$mock_bin/xcrun" <<'EOF'
#!/bin/zsh
set -euo pipefail
print -r -- "xcrun:$*" >>"$MARKHOLA_MOCK_LOG"
EOF
  chmod +x "$mock_bin/xcrun"

  print -r -- "$mock_bin|$log_path"
}

run_package_case() {
  local case_name="$1"
  shift

  local repo_root="$TEST_ROOT/$case_name"
  setup_package_repo "$repo_root"
  local mock_info
  mock_info="$(setup_package_mocks "$repo_root")"
  local mock_bin="${mock_info%%|*}"
  local log_path="${mock_info#*|}"

  (
    export PATH="$mock_bin:$PATH"
    export MARKHOLA_MOCK_LOG="$log_path"
    export MARKHOLA_MOCK_BIN="$mock_bin"
    cd "$repo_root"
    "$@" ./scripts/package_dmg.sh >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  )

  assert_file_contains "$log_path" "build_app:--target aarch64-apple-darwin --app "
  assert_file_contains "$log_path" "dist/MarkHola-apple-silicon.app"
  assert_file_contains "$log_path" "build_app:--target x86_64-apple-darwin --app "
  assert_file_contains "$log_path" "dist/MarkHola-intel.app"
  assert_file_contains "$log_path" "verify:--app "
  assert_file_contains "$log_path" "dist/MarkHola-apple-silicon.app --architecture arm64"
  assert_file_contains "$log_path" "dist/MarkHola-intel.app --architecture x86_64"
  assert_file_contains "$log_path" "hdiutil:create -volname MarkHola -srcfolder "
  assert_file_contains "$log_path" "dist/dmg-root-apple-silicon -ov -format UDZO "
  assert_file_contains "$log_path" "dist/MarkHola-0.9.0-apple-silicon.dmg"
  assert_file_contains "$log_path" "dist/dmg-root-intel -ov -format UDZO "
  assert_file_contains "$log_path" "dist/MarkHola-0.9.0-intel.dmg"

  [[ -f "$repo_root/dist/MarkHola-0.9.0-apple-silicon.dmg" ]] \
    || fail "Expected Apple Silicon DMG output for $case_name"
  [[ -f "$repo_root/dist/MarkHola-0.9.0-intel.dmg" ]] \
    || fail "Expected Intel DMG output for $case_name"
  [[ -f "$repo_root/dist/MarkHola-0.9.0-apple-silicon.manifest.txt" ]] \
    || fail "Expected Apple Silicon manifest for $case_name"
  [[ -f "$repo_root/dist/MarkHola-0.9.0-intel.manifest.txt" ]] \
    || fail "Expected Intel manifest for $case_name"

  print -r -- "$repo_root"
}

test_package_without_signing() {
  local repo_root
  repo_root="$(run_package_case package-no-sign env)"

  assert_file_contains "$repo_root/stderr.log" "DMG signing is skipped for MarkHola-0.9.0-apple-silicon.dmg"
  assert_file_contains "$repo_root/stderr.log" "DMG signing is skipped for MarkHola-0.9.0-intel.dmg"
  if /usr/bin/grep -Fq "xcrun:" "$repo_root/mock.log"; then
    fail "Unexpected xcrun invocation without notarization"
  fi
}

test_package_with_signing_and_notary() {
  local repo_root
  repo_root="$(run_package_case package-sign-notary env CODESIGN_IDENTITY='Developer ID Application: Example' NOTARY_PROFILE=markhola-notary)"

  assert_file_contains "$repo_root/mock.log" "codesign:--force --timestamp --sign Developer ID Application: Example "
  assert_file_contains "$repo_root/mock.log" "codesign:--verify --verbose=2 "
  assert_file_contains "$repo_root/mock.log" "xcrun:notarytool submit "
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola-0.9.0-apple-silicon.dmg --keychain-profile markhola-notary --wait"
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola-0.9.0-intel.dmg --keychain-profile markhola-notary --wait"
  assert_file_contains "$repo_root/mock.log" "xcrun:stapler staple "
  assert_file_contains "$repo_root/mock.log" "xcrun:stapler validate "
}

setup_regression_repo() {
  local repo_root="$1"

  mkdir -p \
    "$repo_root/scripts" \
    "$repo_root/examples" \
    "$repo_root/assets/help" \
    "$repo_root/i18n" \
    "$repo_root/themes/default" \
    "$repo_root/themes/dark" \
    "$repo_root/target/release" \
    "$repo_root/dist"

  cp "$ROOT_DIR/scripts/release_regression.sh" "$repo_root/scripts/release_regression.sh"

  cat >"$repo_root/Cargo.toml" <<'EOF'
[package]
name = "markhola"
version = "0.9.0"
edition = "2024"
EOF

  cat >"$repo_root/scripts/macos_toolchain.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
markhola_prepare_rust_toolchain() {
  :
}
markhola_cargo() {
  print -r -- "markhola_cargo:$*" >>"$MARKHOLA_MOCK_LOG"

  case " $* " in
    *" run "*"--smoke-export-html "*)
      local output_path="${@: -1}"
      print -r -- "<html>ok</html>" >"$output_path"
      ;;
    *" run "*"--smoke-export "*)
      local output_path="${@: -1}"
      print -r -- "%PDF-1.7" >"$output_path"
      ;;
    *" run "*"--smoke-print-pages "*)
      # Match the current accepted Mermaid print layout baseline.
      print -r -- "pages=7"
      ;;
    *" run "*"--smoke-print-prepare "*)
      :
      ;;
    *" test "*|*" build "*)
      :
      ;;
    *)
      print -u2 -- "Unexpected markhola_cargo invocation: $*"
      return 1
      ;;
  esac
}
EOF

  cat >"$repo_root/scripts/test_verify_macos_architectures.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
print -r -- "test_verify:$*" >>"$MARKHOLA_MOCK_LOG"
EOF
  chmod +x "$repo_root/scripts/test_verify_macos_architectures.sh"

  cat >"$repo_root/scripts/verify_macos_architectures.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
print -r -- "verify_release:$*" >>"$MARKHOLA_MOCK_LOG"
EOF
  chmod +x "$repo_root/scripts/verify_macos_architectures.sh"

  cat >"$repo_root/scripts/package_dmg.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
attempt_file="$ROOT_DIR/package-attempt.txt"
attempt=0
if [[ -f "$attempt_file" ]]; then
  attempt="$(<"$attempt_file")"
fi
attempt=$((attempt + 1))
print -r -- "$attempt" >"$attempt_file"
print -r -- "package_attempt:$attempt" >>"$MARKHOLA_MOCK_LOG"
if [[ "$attempt" -lt 3 ]]; then
  exit 1
fi
for label in apple-silicon intel; do
  app="$ROOT_DIR/dist/MarkHola-${label}.app"
  mkdir -p "$app/Contents/Resources/themes/default"
  mkdir -p "$app/Contents/Resources/themes/dark"
  mkdir -p "$app/Contents/Resources/help"
  print -n -- "css" >"$app/Contents/Resources/themes/default/layout.css"
  print -n -- "css" >"$app/Contents/Resources/themes/dark/layout.css"
  cp "$ROOT_DIR/assets/help/Documentation.md" "$app/Contents/Resources/help/Documentation.md"
  cp "$ROOT_DIR/assets/help/Documentation.zh-CN.md" "$app/Contents/Resources/help/Documentation.zh-CN.md"
  print -n -- "dmg" >"$ROOT_DIR/dist/MarkHola-0.9.0-${label}.dmg"
  print -n -- "manifest" >"$ROOT_DIR/dist/MarkHola-0.9.0-${label}.manifest.txt"
done
EOF
  chmod +x "$repo_root/scripts/package_dmg.sh"

  cat >"$repo_root/target/release/markhola" <<'EOF'
#!/bin/zsh
set -euo pipefail

command_name="${1:-}"
shift || true

case "$command_name" in
  version)
    print -r -- "MarkHola 0.9.0"
    ;;
  help)
    print -r -- "Usage: markhola <export-png|export-pdf|export-html|version|help>"
    ;;
  export-png|export-pdf|export-html)
    target_path=""
    for argument in "$@"; do
      case "$argument" in
        --target=*)
          target_path="${argument#--target=}"
          ;;
      esac
    done
    [[ -n "$target_path" ]] || exit 2
    case "$command_name" in
      export-png)
        printf '\211PNG\r\n\032\n' >"$target_path"
        ;;
      export-pdf)
        print -n -- "%PDF-1.7" >"$target_path"
        ;;
      export-html)
        print -n -- "<!DOCTYPE html><html></html>" >"$target_path"
        ;;
    esac
    print -r -- '{"schema_version":1,"status":"completed"}'
    ;;
  *)
    exit 2
    ;;
esac
EOF
  chmod +x "$repo_root/target/release/markhola"

  print -r -- "# example" >"$repo_root/examples/basic.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/languages.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/mermaid.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/math.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/multi-document.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/pdf-export.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/theme-showcase.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/v0.9.2-offline-cli-export.md"
  print -r -- "Current version: \`v0.9.0\`" >"$repo_root/assets/help/Documentation.md"
  print -r -- "Current version: \`v0.9.0\`" >"$repo_root/assets/help/Documentation.zh-CN.md"
  print -n -- "en" >"$repo_root/i18n/en.yaml"
  print -n -- "zh" >"$repo_root/i18n/zh-CN.yaml"
  print -n -- "css" >"$repo_root/themes/default/layout.css"
  print -n -- "css" >"$repo_root/themes/dark/layout.css"
  print -r -- "manual" >"$repo_root/scripts/release_regression_checklist.md"
}

test_release_regression_retries_packaging() {
  local repo_root="$TEST_ROOT/release-retry"
  setup_regression_repo "$repo_root"
  rm -rf "$repo_root/dist"

  (
    export MARKHOLA_MOCK_LOG="$repo_root/mock.log"
    export MARKHOLA_SOCKET_PREFLIGHT=pass
    cd "$repo_root"
    ./scripts/release_regression.sh --with-package >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  )

  assert_file_contains "$repo_root/mock.log" "test_verify:"
  assert_file_contains "$repo_root/mock.log" "markhola_cargo:test --locked --manifest-path "
  assert_file_contains "$repo_root/mock.log" "Cargo.toml --target x86_64-apple-darwin"
  [[ -d "$repo_root/dist" ]] || fail "Expected release_regression.sh to create dist for release-retry"
  assert_file_contains "$repo_root/mock.log" "package_attempt:1"
  assert_file_contains "$repo_root/mock.log" "package_attempt:2"
  assert_file_contains "$repo_root/mock.log" "package_attempt:3"
  assert_file_contains "$repo_root/mock.log" "verify_release:--app "
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola-apple-silicon.app --architecture arm64"
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola-intel.app --architecture x86_64"
  assert_file_contains "$repo_root/stderr.log" "Retrying full packaging flow after transient failure (attempt 1/3)..."
  assert_file_contains "$repo_root/stderr.log" "Retrying full packaging flow after transient failure (attempt 2/3)..."
}

test_release_regression_rejects_version_mismatch() {
  local repo_root="$TEST_ROOT/release-version-mismatch"
  setup_regression_repo "$repo_root"
  print -r -- "Current version: \`v0.8.9\`" >"$repo_root/assets/help/Documentation.md"

  if (
    export MARKHOLA_MOCK_LOG="$repo_root/mock.log"
    export MARKHOLA_SOCKET_PREFLIGHT=pass
    cd "$repo_root"
    ./scripts/release_regression.sh >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  ); then
    fail "Expected release_regression.sh to reject help-version mismatch"
  fi

  assert_file_contains "$repo_root/stderr.log" "Bundled Help version mismatch: assets/help/Documentation.md must declare v0.9.0"
}

test_release_regression_fails_closed_without_socket_capability() {
  local repo_root="$TEST_ROOT/release-socket-preflight"
  setup_regression_repo "$repo_root"
  rm -rf "$repo_root/dist"

  if (
    export MARKHOLA_MOCK_LOG="$repo_root/mock.log"
    export MARKHOLA_SOCKET_PREFLIGHT=fail
    cd "$repo_root"
    ./scripts/release_regression.sh >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  ); then
    fail "Expected release_regression.sh to fail closed without Unix socket capability"
  fi

  assert_file_contains "$repo_root/stderr.log" "Release regression requires Unix domain socket bind capability for protocol transport tests."
  assert_file_contains "$repo_root/stderr.log" "Run scripts/release_regression.sh in an allowed local environment, not a sandbox that denies AF_UNIX bind."
}

test_package_without_signing
test_package_with_signing_and_notary
test_release_regression_retries_packaging
test_release_regression_rejects_version_mismatch
test_release_regression_fails_closed_without_socket_capability

print -r -- "All release packaging logic tests passed."
