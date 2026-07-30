#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS="$ROOT_DIR/.github/scripts/intel_g4_candidate_validation.sh"
FIXTURE="$ROOT_DIR/.github/fixtures/intel_g4_window_probe.swift"
WORKFLOW="$ROOT_DIR/.github/workflows/intel-g4-candidate-validation.yml"

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
assert_contains "$WORKFLOW" "contents: write"
assert_contains "$WORKFLOW" 'env:'
assert_contains "$WORKFLOW" 'ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence'
assert_contains "$WORKFLOW" 'GH_TOKEN: ${{ github.token }}'
if ruby -e '
  lines = File.readlines(ARGV[0])
  job_env = lines[18..22].join
  abort("job-env-runner-temp") if job_env.include?("ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence")
  validate_step = lines[26..31].join
  abort("missing-step-runner-temp") unless validate_step.include?("ARTIFACT_ROOT: ${{ runner.temp }}/intel-g4-evidence")
  abort("job-env-gh-token") if job_env.include?("GH_TOKEN: ${{ github.token }}")
  abort("missing-step-gh-token") unless validate_step.include?("GH_TOKEN: ${{ github.token }}")
  upload_step = lines[33..38].join
  abort("upload-step-gh-token") if upload_step.include?("GH_TOKEN: ${{ github.token }}")
' "$WORKFLOW"; then
  :
else
  echo "runner.temp or GH_TOKEN placement in workflow is invalid" >&2
  exit 1
fi
if grep -Fq 'GH_TOKEN' "$HARNESS"; then
  echo "Harness should not reference GH_TOKEN directly." >&2
  exit 1
fi
if rg -n 'gh api .* (-X|--method) ' "$HARNESS" >/dev/null; then
  echo "Harness must not use non-GET gh api methods." >&2
  exit 1
fi
if rg -n 'gh release (create|edit|delete|upload|verify-asset)' "$HARNESS" >/dev/null; then
  echo "Harness must not perform release mutations." >&2
  exit 1
fi
if rg -n 'releases/(generate-notes|assets\?name=|[0-9]+/(assets|upload)|[0-9]+$)' "$HARNESS" >/dev/null; then
  echo "Harness references an unexpected release mutation endpoint." >&2
  exit 1
fi
if rg -n 'curl .*api.github.com|gh api .* -f |gh api .* --field |gh api .* -F |gh api .* --raw-field ' "$HARNESS" >/dev/null; then
  echo "Harness must not send API write payloads." >&2
  exit 1
fi
assert_contains "$HARNESS" 'gh api "repos/${GITHUB_REPOSITORY}/releases"'
assert_contains "$HARNESS" '"repos/${GITHUB_REPOSITORY}/releases/assets/${asset_id}" >"$DMG_PATH"'
assert_contains "$HARNESS" 'release = releases.find { |entry| entry["draft"] && entry["tag_name"] == ENV.fetch("RELEASE_TAG") }'
assert_contains "$HARNESS" 'asset = release.fetch("assets").find { |entry| entry["name"] == ENV.fetch("RELEASE_ASSET_NAME") }'
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
assert_contains "$HARNESS" 'first application process whose unix id is (targetPid as integer)'
assert_contains "$HARNESS" 'CGWindow owner PID mismatch'
assert_contains "$FIXTURE" "let windowOwnerPID: Int32?"
assert_contains "$FIXTURE" 'mode: "inspect-existing-pid"'

echo "Intel G4 workflow static checks passed."
