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

write_runner_main_stubs() {
  local stub_bin="$1"
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
  cat > "$stub_bin/diff" <<'EOF'
#!/bin/zsh
/usr/bin/diff "$@"
EOF
  cat > "$stub_bin/file" <<'EOF'
#!/bin/zsh
for arg in "$@"; do
  if [[ "$arg" = *"/Contents/MacOS/MarkHola" ]]; then
    print -r -- "$arg: Mach-O 64-bit executable"
    exit 0
  fi
done
/usr/bin/file "$@"
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
    dmg_path=""
    shift
    while [[ $# -gt 0 ]]; do
      case "$1" in
        -mountpoint) mountpoint="$2"; shift 2 ;;
        -readonly|-nobrowse) shift ;;
        *) dmg_path="$1"; shift ;;
      esac
    done
    mkdir -p "$mountpoint/MarkHola.app/Contents/MacOS" "$mountpoint/MarkHola.app/Contents/Resources"
    if [[ "$dmg_path" = *"intel"* ]]; then
      printf '#!/bin/zsh\nprint intel\n' > "$mountpoint/MarkHola.app/Contents/MacOS/MarkHola"
    else
      printf '#!/bin/zsh\nprint apple\n' > "$mountpoint/MarkHola.app/Contents/MacOS/MarkHola"
    fi
    chmod +x "$mountpoint/MarkHola.app/Contents/MacOS/MarkHola"
    printf 'plist\n' > "$mountpoint/MarkHola.app/Contents/Info.plist"
    if [[ "$dmg_path" = *"drift"* ]]; then
      printf 'drifted resource\n' > "$mountpoint/MarkHola.app/Contents/Resources/example.txt"
    else
      printf 'shared resource\n' > "$mountpoint/MarkHola.app/Contents/Resources/example.txt"
    fi
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
else
  /usr/bin/find "$@"
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
}

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

  write_runner_main_stubs "$stub_bin"

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

thin_pair_resource_parity_contract() {
  local tmpdir stub_bin evidence_ok evidence_fail apple_dmg intel_dmg intel_drift_dmg apple_sha intel_sha intel_drift_sha
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/markhola-runner-parity.XXXXXX")"
  stub_bin="$tmpdir/bin"
  evidence_ok="$tmpdir/evidence-ok"
  evidence_fail="$tmpdir/evidence-fail"
  apple_dmg="$tmpdir/apple.dmg"
  intel_dmg="$tmpdir/intel.dmg"
  intel_drift_dmg="$tmpdir/intel-drift.dmg"
  mkdir -p "$stub_bin"
  printf 'apple\n' > "$apple_dmg"
  printf 'intel\n' > "$intel_dmg"
  printf 'intel drift\n' > "$intel_drift_dmg"
  apple_sha="$(/usr/bin/shasum -a 256 "$apple_dmg" | /usr/bin/awk '{print $1}')"
  intel_sha="$(/usr/bin/shasum -a 256 "$intel_dmg" | /usr/bin/awk '{print $1}')"
  intel_drift_sha="$(/usr/bin/shasum -a 256 "$intel_drift_dmg" | /usr/bin/awk '{print $1}')"

  write_runner_main_stubs "$stub_bin"

  if ! PATH="$stub_bin:/usr/bin:/bin" "$RUNNER" \
    --apple-dmg "$apple_dmg" \
    --intel-dmg "$intel_dmg" \
    --apple-sha "$apple_sha" \
    --intel-sha "$intel_sha" \
    --evidence-dir "$evidence_ok" >/tmp/markhola-exact-parity-pass.out 2>&1; then
    print -u2 -- "thin pair pass case unexpectedly failed"
    cat /tmp/markhola-exact-parity-pass.out >&2
    return 1
  fi

  grep -Fq -- "resource_parity=PASS" "$evidence_ok/paired-manifest.txt" || {
    print -u2 -- "resource parity pass manifest missing PASS marker"
    return 1
  }
  [[ -f "$evidence_ok/apple.resources.sha256" && -f "$evidence_ok/intel.resources.sha256" && -f "$evidence_ok/resource-parity.diff" ]] || {
    print -u2 -- "pass case did not preserve resource manifests and diff"
    return 1
  }
  grep -Fq -- "Contents/Resources/example.txt" "$evidence_ok/apple.resources.sha256" || {
    print -u2 -- "resource manifest did not stay scoped to Contents/Resources"
    return 1
  }
  if grep -Fq -- "Contents/MacOS/MarkHola" "$evidence_ok/apple.resources.sha256"; then
    print -u2 -- "resource manifest incorrectly included Mach-O"
    return 1
  fi

  if PATH="$stub_bin:/usr/bin:/bin" "$RUNNER" \
    --apple-dmg "$apple_dmg" \
    --intel-dmg "$intel_drift_dmg" \
    --apple-sha "$apple_sha" \
    --intel-sha "$intel_drift_sha" \
    --evidence-dir "$evidence_fail" >/tmp/markhola-exact-parity-fail.out 2>&1; then
    print -u2 -- "resource drift case unexpectedly passed"
    cat /tmp/markhola-exact-parity-fail.out >&2
    return 1
  fi

  [[ -f "$evidence_fail/apple.resources.sha256" && -f "$evidence_fail/intel.resources.sha256" && -f "$evidence_fail/resource-parity.diff" ]] || {
    print -u2 -- "fail case did not preserve resource manifests and diff"
    return 1
  }
  grep -Fq -- "Contents/Resources/example.txt" "$evidence_fail/resource-parity.diff" || {
    print -u2 -- "resource drift diff did not capture resource mismatch"
    return 1
  }
}

thin_pair_resource_parity_contract || exit 1
print -r -- "PASS: thin pair resource parity contract"
print -r -- "PASS: static safety and fail-closed checks"
