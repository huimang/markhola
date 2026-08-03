#!/bin/zsh
set -euo pipefail
ROOT_DIR="$(cd "${0:A:h}/.." && pwd)"
RUNNER="$ROOT_DIR/scripts/device_validation/run.sh"
EVALUATOR="$ROOT_DIR/scripts/device_validation/evaluate_log.sh"
METRICS_EVALUATOR="$ROOT_DIR/scripts/device_validation/evaluate_visual_metrics.sh"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/markhola-device-framework.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
fail() { print -u2 -- "FAIL: $*"; exit 1; }
[[ -f "$RUNNER" && -f "$EVALUATOR" && -f "$METRICS_EVALUATOR" ]] || fail "framework scripts missing"
grep -Fq 'CASE_RESULT status=' "$RUNNER" || fail "structured result required"
grep -Fq 'release_mutation=NONE' "$RUNNER" || fail "release guard required"
metrics="$TMP/metrics.properties"
cat > "$metrics" <<'EOF'
theme=dark
format=pdf
body_text_contrast=7.1
code_text_contrast=6.2
border_contrast=3.4
content_width=960
line_height=1.5
no_edge_clipping=true
resources_complete=true
output_valid=true
EOF
"$METRICS_EVALUATOR" "$metrics" > "$TMP/metrics.out" || fail "valid visual metrics rejected"
grep -Fq 'VISUAL_METRICS status=PASS' "$TMP/metrics.out" || fail "visual metrics pass missing"
sed 's/body_text_contrast=7.1/body_text_contrast=2.1/' "$metrics" > "$TMP/bad-metrics"
if "$METRICS_EVALUATOR" "$TMP/bad-metrics" > "$TMP/bad.out"; then fail "low contrast accepted"; fi
grep -Fq 'VISUAL_METRICS status=FAIL' "$TMP/bad.out" || fail "visual metric failure missing"
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
