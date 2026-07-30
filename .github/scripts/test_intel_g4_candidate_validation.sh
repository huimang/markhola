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
assert_contains "$HARNESS" '[[ ! "$EXPECTED_SHA256" =~ ^[A-Fa-f0-9]{64}$ ]]'
assert_contains "$HARNESS" 'printf '\''%s\n'\'' "/var/log/markhola/markholo-${stamp}.log"'
assert_contains "$HARNESS" 'printf '\''%s\n'\'' "/tmp/markhola.log"'
assert_contains "$HARNESS" 'LSMinimumSystemVersion=$minimum_system_version'
assert_contains "$HARNESS" '[[ "$minimum_system_version" == "14.0" ]]'
assert_contains "$HARNESS" 'grep -q "pid=$app_pid" "$candidate_path"'
assert_contains "$HARNESS" 'append_ui_result "startup_log_binding" "BLOCKED"'
assert_contains "$HARNESS" 'append_ui_result "startup_log_binding" "PASS"'
assert_contains "$HARNESS" 'first application process whose unix id is (targetPid as integer)'
assert_contains "$HARNESS" 'CGWindow owner PID mismatch'
assert_contains "$FIXTURE" "let windowOwnerPID: Int32?"
assert_contains "$FIXTURE" 'mode: "inspect-existing-pid"'

echo "Intel G4 workflow static checks passed."
