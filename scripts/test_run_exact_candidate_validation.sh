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
print -r -- "PASS: static safety and fail-closed checks"
