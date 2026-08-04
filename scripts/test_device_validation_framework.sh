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

stock="$TMP/stock-cases"; mkdir -p "$stock"
cat > "$stock/fast.sh" <<'EOF'
#!/bin/zsh
CASE_ID="fast"
CASE_TITLE="Fast case"
CASE_TAGS="smoke"
case_run() {
  print -r -- "CASE_RESULT status=PASS note=fast" > "$1/result.txt"
  print -r -- "CASE_RESULT status=PASS note=fast"
}
EOF
cat > "$stock/slow.sh" <<'EOF'
#!/bin/zsh
CASE_ID="slow"
CASE_TITLE="Slow case"
CASE_TAGS="smoke"
case_run() {
  sleep 30 &
  child=$!
  print -r -- "$child" > "$1/child.pid"
  wait "$child"
}
EOF
chmod +x "$stock/fast.sh" "$stock/slow.sh"

exact_stub="$TMP/exact-runner.sh"
cat > "$exact_stub" <<'EOF'
#!/bin/zsh
set -euo pipefail
while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir) evidence_dir="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$evidence_dir"
print -r -- "identity stub" > "$evidence_dir/paired-manifest.txt"
EOF
chmod +x "$exact_stub"

PATH="/usr/bin:/bin" DEVICE_VALIDATION_CASES_DIR="$stock" DEVICE_VALIDATION_EXACT_RUNNER="$exact_stub" \
  "$RUNNER" --apple-dmg /tmp/a --intel-dmg /tmp/i --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})" --timeout 2 --evidence-dir "$TMP/registry" > "$TMP/registry.out" 2>&1 || true
grep -Fq 'IDENTITY status=PASS' "$TMP/registry.out" || fail "identity stub should pass"
grep -Fq 'CASE id=fast status=PASS' "$TMP/registry.out" || fail "fast case should pass under stock PATH"
grep -Fq 'CASE id=slow status=FAIL reason=timeout' "$TMP/registry.out" || fail "slow case should fail with timeout"
run_dir="$(sed -n 's/^RUN end=.*summary_json=\(.*\) summary_md=.*/\1/p' "$TMP/registry.out" | head -1 | xargs dirname)"
if [[ -z "$run_dir" ]]; then
  run_dir="$(find "$TMP/registry" -mindepth 1 -maxdepth 1 -type d | head -1)"
fi
[[ -n "$run_dir" && -d "$run_dir" ]] || fail "run directory missing"
grep -Fq 'CASE_RESULT status=FAIL reason=timeout' "$run_dir/cases/slow/case.log" || fail "timeout case should emit structured FAIL"
child_pid="$(cat "$run_dir/cases/slow/child.pid")"
if ps -p "$child_pid" > /dev/null 2>&1; then fail "timed out child process should be terminated"; fi
grep -Fq 'SUMMARY status=FAIL' "$TMP/registry.out" || fail "timeout run should summarize as FAIL"

success_cases="$TMP/success-cases"; mkdir -p "$success_cases"
cat > "$success_cases/one.sh" <<'EOF'
#!/bin/zsh
CASE_ID="one"
CASE_TITLE="First success"
CASE_TAGS="smoke"
case_run() {
  print -r -- "CASE_RESULT status=PASS note=one"
}
EOF
cat > "$success_cases/two.sh" <<'EOF'
#!/bin/zsh
CASE_ID="two"
CASE_TITLE="Second success"
CASE_TAGS="smoke"
case_run() {
  print -r -- "CASE_RESULT status=PASS note=two"
}
EOF
chmod +x "$success_cases/one.sh" "$success_cases/two.sh"

PATH="/usr/bin:/bin" DEVICE_VALIDATION_CASES_DIR="$success_cases" DEVICE_VALIDATION_EXACT_RUNNER="$exact_stub" \
  "$RUNNER" --apple-dmg /tmp/a --intel-dmg /tmp/i --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})" --timeout 2 --evidence-dir "$TMP/success" > "$TMP/success.out" 2>&1 || fail "successful run should exit 0"
success_run_dir="$(sed -n 's/^RUN end=.*summary_json=\(.*\) summary_md=.*/\1/p' "$TMP/success.out" | head -1 | xargs dirname)"
[[ -n "$success_run_dir" && -d "$success_run_dir" ]] || fail "successful run directory missing"
grep -Fq 'RUN end=' "$TMP/success.out" || fail "successful run must print RUN end"
grep -Fq 'CASE id=one status=PASS' "$TMP/success.out" || fail "first success case missing"
grep -Fq 'CASE id=two status=PASS' "$TMP/success.out" || fail "second success case missing"
python3 - <<'PY' "$success_run_dir/summary.json" || exit 1
import json, sys
data = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
assert data["status"] == "PASS"
assert len(data["cases"]) == 2
assert data["cases"][0]["id"] == "one"
assert data["cases"][1]["id"] == "two"
PY
grep -Fq '# Device Validation Run ' "$success_run_dir/summary.md" || fail "summary.md header missing"
grep -Fq '`one`: **PASS**' "$success_run_dir/summary.md" || fail "summary.md first case missing"
grep -Fq '`two`: **PASS**' "$success_run_dir/summary.md" || fail "summary.md second case missing"

empty_cases="$TMP/empty-cases"; mkdir -p "$empty_cases"
PATH="/usr/bin:/bin" DEVICE_VALIDATION_CASES_DIR="$empty_cases" DEVICE_VALIDATION_EXACT_RUNNER="$exact_stub" \
  "$RUNNER" --apple-dmg /tmp/a --intel-dmg /tmp/i --apple-sha "$(printf 'a%.0s' {1..64})" --intel-sha "$(printf 'b%.0s' {1..64})" --timeout 2 --evidence-dir "$TMP/empty" > "$TMP/empty.out" 2>&1 || fail "empty run should exit 0"
empty_run_dir="$(sed -n 's/^RUN end=.*summary_json=\(.*\) summary_md=.*/\1/p' "$TMP/empty.out" | head -1 | xargs dirname)"
[[ -n "$empty_run_dir" && -d "$empty_run_dir" ]] || fail "empty run directory missing"
python3 - <<'PY' "$empty_run_dir/summary.json" || exit 1
import json, sys
data = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
assert data["status"] == "PASS"
assert data["cases"] == []
PY
grep -Fq 'SUMMARY status=PASS cases=0' "$TMP/empty.out" || fail "empty run summary missing"

print -r -- "PASS: structured parser, priority, duplicate-id, evidence and release guards"
