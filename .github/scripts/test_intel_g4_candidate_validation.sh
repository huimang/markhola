#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS="$ROOT_DIR/.github/scripts/intel_g4_candidate_validation.sh"
FIXTURE="$ROOT_DIR/.github/fixtures/intel_g4_window_probe.swift"
WORKFLOW="$ROOT_DIR/.github/workflows/intel-g4-candidate-validation.yml"
MATRIX_GATE="$ROOT_DIR/.github/scripts/validate_intel_g4_matrix.sh"

assert_contains() {
  local path="$1"
  local needle="$2"
  grep -Fq -- "$needle" "$path" || {
    echo "Missing expected text in $path: $needle" >&2
    exit 1
  }
}

assert_contains "$WORKFLOW" "runs-on: macos-15-intel"
assert_contains "$WORKFLOW" "expected_sha256:"
assert_contains "$WORKFLOW" "MarkHola-0.9.1-intel.dmg"
assert_contains "$WORKFLOW" "RELEASE_TAG: v0.9.1"
assert_contains "$WORKFLOW" "contents: write"
assert_contains "$WORKFLOW" "persist-credentials: false"
assert_contains "$WORKFLOW" 'env:'
assert_contains "$WORKFLOW" 'ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence'
assert_contains "$WORKFLOW" 'GH_TOKEN: ${{ github.token }}'
if ruby -e '
  lines = File.readlines(ARGV[0])
  top_level = lines[0..12].join
  abort("workflow-global-contents-write") if top_level.include?("permissions:\n  contents: write")
  job_block = lines[10..24].join
  abort("missing-job-contents-write") unless job_block.include?("permissions:\n      contents: write")
  job_env = lines[18..22].join
  abort("job-env-runner-temp") if job_env.include?("ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence")
  checkout_step = lines[24..29].join
  abort("missing-persist-credentials-false") unless checkout_step.include?("persist-credentials: false")
  abort("checkout-step-gh-token") if checkout_step.include?("GH_TOKEN: ${{ github.token }}")
  validate_step = lines[30..35].join
  abort("missing-step-runner-temp") unless validate_step.include?("ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence")
  abort("job-env-gh-token") if job_env.include?("GH_TOKEN: ${{ github.token }}")
  abort("missing-step-gh-token") unless validate_step.include?("GH_TOKEN: ${{ github.token }}")
  upload_step = lines[37..42].join
  abort("upload-step-gh-token") if upload_step.include?("GH_TOKEN: ${{ github.token }}")
' "$WORKFLOW"; then
  :
else
  echo "workflow permission or token placement is invalid" >&2
  exit 1
fi
if grep -Fq 'GH_TOKEN' "$HARNESS"; then
  echo "Harness should not reference GH_TOKEN directly." >&2
  exit 1
fi
if grep -Eq '(^|[[:space:]])rg([[:space:]]|$)' "$HARNESS" "$0" "$MATRIX_GATE"; then
  echo "Intel G4 validation must not depend on ripgrep being installed on the runner." >&2
  exit 1
fi
if grep -En 'gh api .* (-X|--method) ' "$HARNESS" >/dev/null; then
  echo "Harness must not use non-GET gh api methods." >&2
  exit 1
fi
if grep -En 'gh release (create|edit|delete|upload|verify-asset)' "$HARNESS" >/dev/null; then
  echo "Harness must not perform release mutations." >&2
  exit 1
fi
if grep -En 'releases/(generate-notes|assets\?name=|[0-9]+/(assets|upload)|[0-9]+$)' "$HARNESS" >/dev/null; then
  echo "Harness references an unexpected release mutation endpoint." >&2
  exit 1
fi
if grep -En 'curl .*api.github.com|gh api .* -f |gh api .* --field |gh api .* -F |gh api .* --raw-field ' "$HARNESS" >/dev/null; then
  echo "Harness must not send API write payloads." >&2
  exit 1
fi
assert_contains "$HARNESS" 'gh api "repos/${GITHUB_REPOSITORY}/releases"'
assert_contains "$HARNESS" '"repos/${GITHUB_REPOSITORY}/releases/assets/${asset_id}" >"$DMG_PATH"'
assert_contains "$HARNESS" 'release = releases.find { |entry| entry["draft"] && entry["tag_name"] == ENV.fetch("RELEASE_TAG") }'
assert_contains "$HARNESS" 'asset = release.fetch("assets").find { |entry| entry["name"] == ENV.fetch("RELEASE_ASSET_NAME") }'
assert_contains "$HARNESS" 'permission_exception=validation-job-uses-contents-write-for-draft-read'
assert_contains "$HARNESS" 'token_injection=validate-step-only-gh-token'
assert_contains "$HARNESS" '[[ ! "$EXPECTED_SHA256" =~ ^[A-Fa-f0-9]{64}$ ]]'
assert_contains "$HARNESS" 'printf '\''%s\n'\'' "/var/log/markhola/markholo-${stamp}.log"'
assert_contains "$HARNESS" 'printf '\''%s\n'\'' "/tmp/markhola.log"'
assert_contains "$HARNESS" 'LSMinimumSystemVersion=$minimum_system_version'
assert_contains "$HARNESS" '[[ "$minimum_system_version" == "14.0" ]]'
assert_contains "$HARNESS" 'grep -q "pid=$app_pid" "$candidate_path"'
assert_contains "$HARNESS" 'append_ui_result "startup_log_binding" "BLOCKED"'
assert_contains "$HARNESS" 'append_ui_result "startup_log_binding" "PASS"'
assert_contains "$HARNESS" "grep -q 'Code Type:.*X86-64' \"\$UI_DIR/sample.txt\""
assert_contains "$HARNESS" "grep -Eqi 'translated|arm64|aarch64' \"\$UI_DIR/sample.txt\""
assert_contains "$HARNESS" 'append_ui_result "runtime_architecture" "BLOCKED"'
assert_contains "$HARNESS" 'append_ui_result "runtime_architecture" "PASS"'
assert_contains "$HARNESS" 'append_ui_result "executable_path_binding" "BLOCKED"'
assert_contains "$HARNESS" 'append_ui_result "lsof_binding" "BLOCKED"'
for behavior_row in \
  about_panel \
  open_edit_save \
  readonly_rendering \
  markdown_code_rendering \
  mermaid_math_image_rendering \
  pdf_html_print \
  menu_tab_window \
  theme_language_help \
  stability_exit
do
  assert_contains "$HARNESS" "append_ui_result \"$behavior_row\" \"PASS\""
  assert_contains "$HARNESS" "append_ui_result \"$behavior_row\" \"BLOCKED\""
done
assert_contains "$HARNESS" 'grep -Fqi "macos / x86_64" "$UI_DIR/about.txt"'
assert_contains "$HARNESS" 'capture_about_identity "$app_pid" "$expected_version"'
assert_contains "$HARNESS" 'click menu item "Light" of menu 1 of menu item "Theme"'
if grep -Fq '0.9.0' "$WORKFLOW" "$HARNESS"; then
  echo "Intel G4 workflow or harness retains a v0.9.0 candidate binding." >&2
  exit 1
fi
assert_contains "$HARNESS" 'set editorArea to missing value'
assert_contains "$HARNESS" 'if role of uiItem is "AXTextArea"'
assert_contains "$HARNESS" '"$candidate_executable" --smoke-export'
assert_contains "$HARNESS" '"$candidate_executable" --smoke-export-html'
assert_contains "$HARNESS" '"$candidate_executable" --smoke-print-pages'
assert_contains "$HARNESS" 'MarkHola intentionally has no Window menu'
assert_contains "$HARNESS" 'first application process whose unix id is (targetPid as integer)'
assert_contains "$HARNESS" 'CGWindow owner PID mismatch'
assert_contains "$HARNESS" '[[ "$can_run_gui" -eq 1 ]]'
assert_contains "$FIXTURE" "let windowOwnerPID: Int32?"
assert_contains "$FIXTURE" 'mode: "inspect-existing-pid"'

test_root="$(mktemp -d "${TMPDIR:-/tmp}/markhola-intel-g4-matrix.XXXXXX")"
trap 'rm -rf "$test_root"' EXIT

make_matrix() {
  local matrix_path="$1"
  printf 'check\tstatus\tdetail\n' >"$matrix_path"
  while IFS= read -r check_name; do
    printf '%s\tPASS\tfixture pass\n' "$check_name" >>"$matrix_path"
  done < <(bash "$MATRIX_GATE" --print-required)
}

complete_matrix="$test_root/complete.tsv"
complete_blockers="$test_root/complete-blockers.txt"
complete_summary="$test_root/complete-summary.txt"
make_matrix "$complete_matrix"
bash "$MATRIX_GATE" "$complete_matrix" "$complete_blockers" "$complete_summary"
assert_contains "$complete_summary" "overall=PASS"
[[ ! -s "$complete_blockers" ]] || {
  echo "Complete matrix unexpectedly produced blockers." >&2
  exit 1
}

missing_matrix="$test_root/missing.tsv"
missing_blockers="$test_root/missing-blockers.txt"
missing_summary="$test_root/missing-summary.txt"
make_matrix "$missing_matrix"
grep -v '^launch_markhola' "$missing_matrix" >"$missing_matrix.tmp"
mv "$missing_matrix.tmp" "$missing_matrix"
if bash "$MATRIX_GATE" "$missing_matrix" "$missing_blockers" "$missing_summary"; then
  echo "Matrix gate accepted a missing required row." >&2
  exit 1
fi
assert_contains "$missing_matrix" $'required_launch_markhola\tBLOCKED'
assert_contains "$missing_blockers" "Required G4 row launch_markhola"
assert_contains "$missing_summary" "overall=BLOCKED"

blocked_matrix="$test_root/blocked.tsv"
blocked_blockers="$test_root/blocked-blockers.txt"
blocked_summary="$test_root/blocked-summary.txt"
make_matrix "$blocked_matrix"
sed -i '' $'s/^runtime_architecture\tPASS/runtime_architecture\tBLOCKED/' "$blocked_matrix"
if bash "$MATRIX_GATE" "$blocked_matrix" "$blocked_blockers" "$blocked_summary"; then
  echo "Matrix gate accepted a BLOCKED required row." >&2
  exit 1
fi
assert_contains "$blocked_blockers" "Required G4 row runtime_architecture"
assert_contains "$blocked_summary" "overall=BLOCKED"

duplicate_matrix="$test_root/duplicate.tsv"
duplicate_blockers="$test_root/duplicate-blockers.txt"
duplicate_summary="$test_root/duplicate-summary.txt"
make_matrix "$duplicate_matrix"
printf 'about_panel\tPASS\tduplicate fixture row\n' >>"$duplicate_matrix"
if bash "$MATRIX_GATE" "$duplicate_matrix" "$duplicate_blockers" "$duplicate_summary"; then
  echo "Matrix gate accepted a duplicate required row." >&2
  exit 1
fi
assert_contains "$duplicate_blockers" "Required G4 row about_panel"
assert_contains "$duplicate_summary" "overall=BLOCKED"

identity_only_matrix="$test_root/identity-only.tsv"
identity_only_blockers="$test_root/identity-only-blockers.txt"
identity_only_summary="$test_root/identity-only-summary.txt"
{
  printf 'check\tstatus\tdetail\n'
  printf 'aqua_session\tPASS\tfixture pass\n'
  printf 'windowserver\tPASS\tfixture pass\n'
  printf 'visible_appkit_window\tPASS\tfixture pass\n'
  printf 'ax_tcc\tPASS\tfixture pass\n'
} >"$identity_only_matrix"
if bash "$MATRIX_GATE" "$identity_only_matrix" "$identity_only_blockers" "$identity_only_summary"; then
  echo "Matrix gate accepted generic capability rows without candidate validation." >&2
  exit 1
fi
[[ -s "$identity_only_blockers" ]] || {
  echo "Identity-only matrix did not populate blockers." >&2
  exit 1
}
assert_contains "$identity_only_summary" "overall=BLOCKED"

echo "Intel G4 workflow static checks passed."
