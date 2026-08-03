#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "${0:A:h}/.." && pwd)"
RUNNER="$ROOT_DIR/scripts/run_exact_candidate_validation.sh"
[[ -x "$RUNNER" ]] || { print -u2 -- "Runner must be executable"; exit 1; }

expect_failure() {
  local name="$1"; shift
  if "$RUNNER" "$@" >/tmp/markhola-exact-test.out 2>&1; then
    print -u2 -- "Expected failure: $name"
    cat /tmp/markhola-exact-test.out >&2
    exit 1
  fi
  print -r -- "PASS: $name"
}

expect_failure missing-arguments
expect_failure relative-path --apple-dmg apple.dmg --intel-dmg /tmp/intel.dmg --apple-sha "$(printf '%064d' 0)" --intel-sha "$(printf '%064d' 0)"
expect_failure invalid-sha --apple-dmg /tmp/apple.dmg --intel-dmg /tmp/intel.dmg --apple-sha nope --intel-sha nope
expect_failure same-artifact --apple-dmg /tmp/apple.dmg --intel-dmg /tmp/apple.dmg --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})"
expect_failure old-or-mismatched-artifact --apple-dmg /tmp/does-not-exist.dmg --intel-dmg /tmp/also-missing.dmg --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})"
expect_failure missing-tool --apple-dmg /tmp/apple.dmg --intel-dmg /tmp/intel.dmg --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})" --evidence-dir /tmp/markhola-validation-missing-tool

grep -Fq -- 'manual.GUI_AX=UNSET' "$RUNNER" || { print -u2 -- "Missing manual GUI UNSET contract"; exit 1; }
grep -Fq -- 'hdiutil verify' "$RUNNER" || { print -u2 -- "Missing hdiutil verification"; exit 1; }
grep -Fq -- 'No App was launched' "$RUNNER" || { print -u2 -- "Runner must explicitly avoid GUI launch"; exit 1; }
grep -Fq -- 'refusing to overwrite evidence' "$RUNNER" || { print -u2 -- "Runner must refuse evidence overwrite"; exit 1; }
grep -Fq -- 'Release' "$RUNNER" || { print -u2 -- "Runner must document no Release mutation"; exit 1; }

path_collision_regression() {
  local tmpdir
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/markhola-runner-test.XXXXXX")"
  local dmg="$tmpdir/candidate.dmg"
  local manifest="$tmpdir/manifest.txt"
  printf 'candidate\n' > "$dmg"
  local expected actual_path saved_path
  expected="$(/usr/bin/shasum -a 256 "$dmg" | /usr/bin/awk '{print $1}')"
  saved_path="$PATH"

  MARKHOLA_TEST_SOURCE_ONLY=1 source "$RUNNER"
  MANIFEST="$manifest"
  hdiutil() { return 0; }

  actual_path="$(command -v shasum)"
  [[ "$actual_path" = /usr/bin/shasum ]] || { print -u2 -- "Expected /usr/bin/shasum, got ${actual_path:-missing}"; return 1; }
  actual_path="$(command -v awk)"
  [[ "$actual_path" = /usr/bin/awk ]] || { print -u2 -- "Expected /usr/bin/awk, got ${actual_path:-missing}"; return 1; }
  [[ "$(hash_file "$dmg")" = "$expected" ]] || { print -u2 -- "hash_file changed digest behavior"; return 1; }
  record_dmg apple "$dmg" "$expected"
  [[ "$PATH" = "$saved_path" ]] || { print -u2 -- "PATH was corrupted by sourced helpers"; return 1; }
  grep -Fq -- "apple.dmg_path=$dmg" "$manifest" || { print -u2 -- "record_dmg did not write manifest entry"; return 1; }
}

path_collision_regression || exit 1
print -r -- "PASS: path collision regression"

main_integration_uses_repo_local_verify_script() {
  local tmpdir stub_bin evidence_root evidence apple_dmg intel_dmg apple_sha intel_sha verify_log
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/markhola-runner-main.XXXXXX")"
  stub_bin="$tmpdir/bin"
  evidence_root="$tmpdir/evidence-root"
  evidence="$evidence_root/session"
  apple_dmg="$tmpdir/apple.dmg"
  intel_dmg="$tmpdir/intel.dmg"
  verify_log="$tmpdir/verify.log"
  mkdir -p "$stub_bin" "$evidence_root"
  printf 'apple\n' > "$apple_dmg"
  printf 'intel\n' > "$intel_dmg"
  apple_sha="$(/usr/bin/shasum -a 256 "$apple_dmg" | /usr/bin/awk '{print $1}')"
  intel_sha="$(/usr/bin/shasum -a 256 "$intel_dmg" | /usr/bin/awk '{print $1}')"

  cat > "$stub_bin/shasum" <<'EOF'
#!/bin/zsh
/usr/bin/shasum "$@"
EOF
  cat > "$stub_bin/awk" <<'EOF'
#!/bin/zsh
/usr/bin/awk "$@"
EOF
  cat > "$stub_bin/stat" <<'EOF'
#!/bin/zsh
/usr/bin/stat "$@"
EOF
  cat > "$stub_bin/mktemp" <<'EOF'
#!/bin/zsh
/usr/bin/mktemp "$@"
EOF
  cat > "$stub_bin/sort" <<'EOF'
#!/bin/zsh
/usr/bin/sort "$@"
EOF
  cat > "$stub_bin/cmp" <<'EOF'
#!/bin/zsh
/usr/bin/cmp "$@"
EOF
  cat > "$stub_bin/tee" <<'EOF'
#!/bin/zsh
/usr/bin/tee "$@"
EOF
  cat > "$stub_bin/lipo" <<'EOF'
#!/bin/zsh
if [[ "$2" = *"/intel-copy/"* ]]; then
  print -r -- "x86_64"
else
  print -r -- "arm64"
fi
EOF
  cat > "$stub_bin/xcrun" <<'EOF'
#!/bin/zsh
print -r -- "minos 14.0"
EOF
  cat > "$stub_bin/plutil" <<'EOF'
#!/bin/zsh
case "$2" in
  LSMinimumSystemVersion) print -r -- "14.0" ;;
  CFBundleShortVersionString) print -r -- "0.9.3" ;;
  CFBundleIdentifier) print -r -- "com.markhola.app" ;;
  *) print -u2 -- "unexpected plutil extract: $2"; exit 1 ;;
esac
EOF
  cat > "$stub_bin/codesign" <<'EOF'
#!/bin/zsh
exit 0
EOF
  cat > "$stub_bin/hdiutil" <<'EOF'
#!/bin/zsh
case "$1" in
  verify) exit 0 ;;
  attach)
    mountpoint=""
    shift
    while [[ $# -gt 0 ]]; do
      case "$1" in
        -mountpoint) mountpoint="$2"; shift 2 ;;
        -readonly|-nobrowse) shift ;;
        *) shift ;;
      esac
    done
    mkdir -p "$mountpoint/MarkHola.app/Contents/MacOS" "$mountpoint/MarkHola.app/Contents"
    printf '#!/bin/zsh\nexit 0\n' > "$mountpoint/MarkHola.app/Contents/MacOS/MarkHola"
    chmod +x "$mountpoint/MarkHola.app/Contents/MacOS/MarkHola"
    printf 'plist\n' > "$mountpoint/MarkHola.app/Contents/Info.plist"
    ;;
  detach) exit 0 ;;
  *) print -u2 -- "unexpected hdiutil call: $*"; exit 1 ;;
esac
EOF
  cat > "$stub_bin/find" <<'EOF'
#!/bin/zsh
if [[ "$*" = *"-name *.app"* && "$*" = *"-print -quit"* ]]; then
  print -r -- "$1/MarkHola.app"
elif [[ "$*" = *"-name *.app"* ]]; then
  print -r -- "$1/MarkHola.app"
elif [[ "$1" = "." ]]; then
  /usr/bin/find "$@"
else
  print -u2 -- "unexpected find call: $*"
  exit 1
fi
EOF
  cat > "$stub_bin/ditto" <<'EOF'
#!/bin/zsh
src="$1"
dst="$2"
mkdir -p "$dst"
/bin/cp -R "$src"/. "$dst"/
EOF
  chmod +x "$stub_bin/"*

  local repo_root expected_verify_script original_verify output_file
  repo_root="$(cd "${RUNNER:A:h}/.." && pwd)"
  expected_verify_script="$repo_root/scripts/verify_macos_architectures.sh"
  original_verify="$tmpdir/original-verify.sh"
  output_file="/tmp/markhola-exact-main.out"
  /bin/cp "$expected_verify_script" "$original_verify"
  cat > "$expected_verify_script" <<EOF
#!/bin/zsh
print -r -- "\$0 \$*" >> "$verify_log"
exit 0
EOF
  chmod +x "$expected_verify_script"

  if ! PATH="$stub_bin:/usr/bin:/bin" "$RUNNER" \
    --apple-dmg "$apple_dmg" \
    --intel-dmg "$intel_dmg" \
    --apple-sha "$apple_sha" \
    --intel-sha "$intel_sha" \
    --evidence-dir "$evidence" >"$output_file" 2>&1; then
    /bin/cp "$original_verify" "$expected_verify_script"
    print -u2 -- "main() integration regression failed"
    cat "$output_file" >&2
    return 1
  fi

  /bin/cp "$original_verify" "$expected_verify_script"
  grep -Fq -- "$expected_verify_script --app $evidence/.work." "$verify_log" || {
    print -u2 -- "verify_macos_architectures.sh was not invoked through repo-local path"
    cat "$verify_log" >&2
    return 1
  }
}

main_integration_uses_repo_local_verify_script || exit 1
print -r -- "PASS: main integration keeps repo-local verify script path"
print -r -- "PASS: static safety and fail-closed checks"
