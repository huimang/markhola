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
  cat >"$repo_root/Cargo.toml" <<'EOF'
[package]
name = "markhola"
version = "0.9.0"
edition = "2024"
EOF
  : >"$repo_root/assets/app-icon.png"
}

setup_package_mocks() {
  local repo_root="$1"
  local mock_bin="$repo_root/mock-bin"
  local log_path="$repo_root/mock.log"

  mkdir -p "$mock_bin"

  cat >"$repo_root/scripts/build_app.sh" <<'EOF'
#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
print -r -- "build_app:$*" >>"$ROOT_DIR/mock.log"
mkdir -p "$ROOT_DIR/dist/MarkHola.app/Contents/MacOS"
print -n -- "binary" >"$ROOT_DIR/dist/MarkHola.app/Contents/MacOS/MarkHola"
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
    cd "$repo_root"
    "$@" ./scripts/package_dmg.sh >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  )

  assert_file_contains "$log_path" "build_app:--universal"
  assert_file_contains "$log_path" "verify:--app "
  assert_file_contains "$log_path" "dist/MarkHola.app --universal"
  assert_file_contains "$log_path" "hdiutil:create -volname MarkHola -srcfolder "
  assert_file_contains "$log_path" "dist/dmg-root -ov -format UDZO "
  assert_file_contains "$log_path" "dist/MarkHola-0.9.0.dmg"

  if [[ ! -f "$repo_root/dist/MarkHola-0.9.0.dmg" ]]; then
    fail "Expected DMG output for $case_name"
  fi

  print -r -- "$repo_root"
}

test_package_without_signing() {
  local repo_root
  repo_root="$(run_package_case package-no-sign env)"

  assert_file_contains "$repo_root/stderr.log" "Warning: CODESIGN_IDENTITY is not set; DMG signing is skipped."
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
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola-0.9.0.dmg --keychain-profile markhola-notary --wait"
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
mkdir -p "$ROOT_DIR/dist/MarkHola.app/Contents/Resources/themes/default"
mkdir -p "$ROOT_DIR/dist/MarkHola.app/Contents/Resources/themes/dark"
mkdir -p "$ROOT_DIR/dist/MarkHola.app/Contents/Resources/help"
print -n -- "css" >"$ROOT_DIR/dist/MarkHola.app/Contents/Resources/themes/default/layout.css"
print -n -- "css" >"$ROOT_DIR/dist/MarkHola.app/Contents/Resources/themes/dark/layout.css"
cp "$ROOT_DIR/assets/help/Documentation.md" "$ROOT_DIR/dist/MarkHola.app/Contents/Resources/help/Documentation.md"
cp "$ROOT_DIR/assets/help/Documentation.zh-CN.md" "$ROOT_DIR/dist/MarkHola.app/Contents/Resources/help/Documentation.zh-CN.md"
print -n -- "dmg" >"$ROOT_DIR/dist/MarkHola-0.9.0.dmg"
EOF
  chmod +x "$repo_root/scripts/package_dmg.sh"

  print -r -- "# example" >"$repo_root/examples/basic.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/languages.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/mermaid.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/math.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/multi-document.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/pdf-export.md"
  cp "$repo_root/examples/basic.md" "$repo_root/examples/theme-showcase.md"
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

  (
    export MARKHOLA_MOCK_LOG="$repo_root/mock.log"
    cd "$repo_root"
    ./scripts/release_regression.sh --with-package >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  )

  assert_file_contains "$repo_root/mock.log" "test_verify:"
  assert_file_contains "$repo_root/mock.log" "markhola_cargo:test --locked --manifest-path "
  assert_file_contains "$repo_root/mock.log" "Cargo.toml --target x86_64-apple-darwin"
  assert_file_contains "$repo_root/mock.log" "package_attempt:1"
  assert_file_contains "$repo_root/mock.log" "package_attempt:2"
  assert_file_contains "$repo_root/mock.log" "package_attempt:3"
  assert_file_contains "$repo_root/mock.log" "verify_release:--app "
  assert_file_contains "$repo_root/mock.log" "dist/MarkHola.app --universal"
  assert_file_contains "$repo_root/stderr.log" "Retrying full packaging flow after transient failure (attempt 1/3)..."
  assert_file_contains "$repo_root/stderr.log" "Retrying full packaging flow after transient failure (attempt 2/3)..."
}

test_release_regression_rejects_version_mismatch() {
  local repo_root="$TEST_ROOT/release-version-mismatch"
  setup_regression_repo "$repo_root"
  print -r -- "Current version: \`v0.8.9\`" >"$repo_root/assets/help/Documentation.md"

  if (
    export MARKHOLA_MOCK_LOG="$repo_root/mock.log"
    cd "$repo_root"
    ./scripts/release_regression.sh >"$repo_root/stdout.log" 2>"$repo_root/stderr.log"
  ); then
    fail "Expected release_regression.sh to reject help-version mismatch"
  fi

  assert_file_contains "$repo_root/stderr.log" "Bundled Help version mismatch: assets/help/Documentation.md must declare v0.9.0"
}

test_package_without_signing
test_package_with_signing_and_notary
test_release_regression_retries_packaging
test_release_regression_rejects_version_mismatch

print -r -- "All release packaging logic tests passed."
