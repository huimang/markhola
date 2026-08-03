#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "${0:A:h}/.." && pwd)"
RUNNER="$ROOT_DIR/scripts/device_validation/run.sh"
EVALUATOR="$ROOT_DIR/scripts/device_validation/evaluate_log.sh"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/markhola-device-framework.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
fail() { print -u2 -- "FAIL: $*"; exit 1; }
[[ -f "$RUNNER" && -f "$EVALUATOR" ]] || fail "framework scripts missing"
grep -Fq 'CASE_RESULT status=' "$RUNNER" || fail "structured result required"
grep -Fq 'release_mutation=NONE' "$RUNNER" || fail "release guard required"
log="$TMP/cases.log"
print -r -- 'CASE id=a status=PASS evidence=/tmp/a' > "$log"
print -r -- 'CASE id=b status=BLOCKED evidence=/tmp/b' >> "$log"
print -r -- 'ordinary prose says PASS but is not structured' >> "$log"
if "$EVALUATOR" "$log" > "$TMP/out"; then fail "BLOCKED must not pass"; fi
grep -Fq 'SUMMARY status=BLOCKED' "$TMP/out" || fail "priority aggregation"
dup="$TMP/duplicate"; mkdir -p "$dup"
cp "$ROOT_DIR/scripts/device_validation/cases/candidate_identity.sh" "$dup/one.sh"
cp "$ROOT_DIR/scripts/device_validation/cases/candidate_identity.sh" "$dup/two.sh"
if DEVICE_VALIDATION_CASES_DIR="$dup" "$RUNNER" --apple-dmg /tmp/a --intel-dmg /tmp/i --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})" > "$TMP/run" 2>&1; then fail "duplicate case ids"; fi
print -r -- "PASS: structured parser, priority, duplicate-id, evidence and release guards"
