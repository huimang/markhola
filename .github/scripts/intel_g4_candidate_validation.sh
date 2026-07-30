#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARTIFACT_ROOT="${ARTIFACT_ROOT:?missing ARTIFACT_ROOT}"
EXPECTED_SHA256="${EXPECTED_SHA256:?missing EXPECTED_SHA256}"
RELEASE_TAG="${RELEASE_TAG:?missing RELEASE_TAG}"
RELEASE_ASSET_NAME="${RELEASE_ASSET_NAME:?missing RELEASE_ASSET_NAME}"

PROVENANCE_DIR="$ARTIFACT_ROOT/provenance"
CANDIDATE_DIR="$ARTIFACT_ROOT/candidate"
BUNDLE_DIR="$ARTIFACT_ROOT/bundle"
UI_DIR="$ARTIFACT_ROOT/ui"
SUMMARY_FILE="$ARTIFACT_ROOT/summary.txt"
UI_MATRIX_FILE="$UI_DIR/ui-matrix.tsv"
BLOCKERS_FILE="$UI_DIR/blockers.txt"
STARTUP_LOG="$UI_DIR/startup.log"
APP_LOG_PATH_FILE="$UI_DIR/app-log-path.txt"
RUNTIME_ARCH_FILE="$UI_DIR/runtime-architecture.txt"
MATRIX_GATE="$ROOT_DIR/.github/scripts/validate_intel_g4_matrix.sh"

DMG_PATH="$CANDIDATE_DIR/$RELEASE_ASSET_NAME"
MOUNT_ROOT="$RUNNER_TEMP/intel-g4-mount"
APP_COPY="$BUNDLE_DIR/MarkHola.app"
WINDOW_PROBE_BIN="$RUNNER_TEMP/intel-g4-window-probe"

mkdir -p "$PROVENANCE_DIR" "$CANDIDATE_DIR" "$BUNDLE_DIR" "$UI_DIR"
printf 'check\tstatus\tdetail\n' >"$UI_MATRIX_FILE"
: >"$BLOCKERS_FILE"
: >"$SUMMARY_FILE"

ATTACHED=0

cleanup() {
  if [[ "$ATTACHED" -eq 1 ]]; then
    hdiutil detach "$MOUNT_ROOT" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

note() {
  printf '%s\n' "$1" | tee -a "$SUMMARY_FILE"
}

fail_closed() {
  printf 'FAIL\t%s\n' "$1" | tee -a "$BLOCKERS_FILE" >&2
  note "overall=FAIL"
  exit 1
}

append_ui_result() {
  local check_name="$1"
  local status="$2"
  local detail="$3"
  printf '%s\t%s\t%s\n' "$check_name" "$status" "$detail" >>"$UI_MATRIX_FILE"
  if [[ "$status" != "PASS" ]]; then
    printf '%s\t%s\n' "$check_name" "$detail" >>"$BLOCKERS_FILE"
  fi
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail_closed "Missing required command: $1"
}

compare_directory_parity() {
  local source_dir="$1"
  local target_dir="$2"
  local label="$3"
  local diff_file="$BUNDLE_DIR/${label}-parity.diff"

  if ! diff -rq "$source_dir" "$target_dir" >"$diff_file"; then
    fail_closed "Resource parity mismatch for $label"
  fi
}

current_date_stamp() {
  date '+%Y%m%d'
}

candidate_log_paths() {
  local stamp
  stamp="$(current_date_stamp)"
  printf '%s\n' "/var/log/markhola/markholo-${stamp}.log"
  printf '%s\n' "/tmp/markhola.log"
}

validate_sha_input() {
  if [[ ! "$EXPECTED_SHA256" =~ ^[A-Fa-f0-9]{64}$ ]]; then
    fail_closed "expected_sha256 must be exactly 64 hexadecimal characters"
  fi
}

collect_provenance() {
  {
    echo "workflow_name=${GITHUB_WORKFLOW:-}"
    echo "run_id=${GITHUB_RUN_ID:-}"
    echo "run_attempt=${GITHUB_RUN_ATTEMPT:-}"
    echo "repository=${GITHUB_REPOSITORY:-}"
    echo "ref=${GITHUB_REF:-}"
    echo "sha=${GITHUB_SHA:-}"
    echo "actor=${GITHUB_ACTOR:-}"
    echo "runner_name=${RUNNER_NAME:-}"
    echo "runner_os=${RUNNER_OS:-}"
    echo "runner_arch=${RUNNER_ARCH:-}"
    echo "image_os=${ImageOS:-unknown}"
    echo "image_version=${ImageVersion:-unknown}"
    echo "release_tag=$RELEASE_TAG"
    echo "release_asset_name=$RELEASE_ASSET_NAME"
    echo "expected_sha256=$EXPECTED_SHA256"
    echo "workflow_commit=${GITHUB_SHA:-}"
    echo "permission_exception=validation-job-uses-contents-write-for-draft-read"
    echo "token_injection=validate-step-only-gh-token"
  } >"$PROVENANCE_DIR/workflow-context.txt"

  uname -a >"$PROVENANCE_DIR/uname.txt"
  sw_vers >"$PROVENANCE_DIR/sw_vers.txt"
  system_profiler SPHardwareDataType >"$PROVENANCE_DIR/hardware.txt"
  sysctl -a >"$PROVENANCE_DIR/sysctl.txt"
}

download_draft_asset() {
  local releases_json="$PROVENANCE_DIR/releases.json"
  local release_id asset_id

  gh api "repos/${GITHUB_REPOSITORY}/releases" >"$releases_json"

  release_id="$(
    ruby -rjson -e '
      releases = JSON.parse(File.read(ARGV[0]))
      release = releases.find { |entry| entry["draft"] && entry["tag_name"] == ENV.fetch("RELEASE_TAG") }
      abort("missing-draft-release") unless release
      puts release["id"]
    ' "$releases_json"
  )" || fail_closed "Unable to find draft release for tag $RELEASE_TAG"

  asset_id="$(
    ruby -rjson -e '
      releases = JSON.parse(File.read(ARGV[0]))
      release = releases.find { |entry| entry["draft"] && entry["tag_name"] == ENV.fetch("RELEASE_TAG") }
      abort("missing-draft-release") unless release
      asset = release.fetch("assets").find { |entry| entry["name"] == ENV.fetch("RELEASE_ASSET_NAME") }
      abort("missing-draft-asset") unless asset
      puts asset["id"]
    ' "$releases_json"
  )" || fail_closed "Unable to find draft asset $RELEASE_ASSET_NAME"

  {
    echo "release_id=$release_id"
    echo "asset_id=$asset_id"
  } >"$PROVENANCE_DIR/draft-release-selection.txt"

  gh api \
    -H "Accept: application/octet-stream" \
    "repos/${GITHUB_REPOSITORY}/releases/assets/${asset_id}" >"$DMG_PATH"

  local actual_sha
  actual_sha="$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')"
  {
    echo "asset_path=$DMG_PATH"
    echo "expected_sha256=$EXPECTED_SHA256"
    echo "actual_sha256=$actual_sha"
  } >"$CANDIDATE_DIR/sha256.txt"

  [[ "$actual_sha" == "$EXPECTED_SHA256" ]] || fail_closed "Draft asset SHA-256 mismatch"
  append_ui_result "candidate_sha256" "PASS" "Downloaded draft asset matches expected SHA-256"
}

verify_runner_identity() {
  local uname_m arch_name hw_machine translated arm64_flag

  uname_m="$(uname -m)"
  arch_name="$(arch)"
  hw_machine="$(sysctl -n hw.machine)"
  translated="$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)"
  arm64_flag="$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)"

  {
    echo "uname_m=$uname_m"
    echo "arch=$arch_name"
    echo "hw_machine=$hw_machine"
    echo "sysctl_proc_translated=$translated"
    echo "hw_optional_arm64=$arm64_flag"
  } >"$PROVENANCE_DIR/runner-identity.txt"

  [[ "$uname_m" == "x86_64" ]] || fail_closed "Runner uname -m must be x86_64"
  [[ "$arch_name" == "i386" || "$arch_name" == "x86_64" ]] || fail_closed "Runner arch must report x86_64-compatible Intel execution"
  [[ "$hw_machine" == "x86_64" ]] || fail_closed "Hardware view must report x86_64"
  [[ "$translated" == "0" ]] || fail_closed "sysctl.proc_translated must be 0"
  [[ "$arm64_flag" == "0" ]] || fail_closed "Runner must not expose Apple Silicon hardware mode"
  append_ui_result "runner_identity" "PASS" "Runner is native x86_64 with sysctl.proc_translated=0"
}

verify_dmg_and_copy_app() {
  hdiutil verify "$DMG_PATH" >"$CANDIDATE_DIR/hdiutil-verify.txt" 2>&1
  hdiutil imageinfo "$DMG_PATH" >"$CANDIDATE_DIR/hdiutil-imageinfo.txt"

  if ! grep -Eq '^Format:[[:space:]]+UDZO$' "$CANDIDATE_DIR/hdiutil-imageinfo.txt"; then
    fail_closed "DMG format must be UDZO"
  fi

  rm -rf "$MOUNT_ROOT" "$APP_COPY"
  mkdir -p "$MOUNT_ROOT"
  hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_ROOT" -nobrowse -readonly >"$CANDIDATE_DIR/hdiutil-attach.txt"
  ATTACHED=1

  find "$MOUNT_ROOT" -print | LC_ALL=C sort >"$CANDIDATE_DIR/mounted-tree.txt"

  local mounted_app
  mounted_app="$(find "$MOUNT_ROOT" -maxdepth 2 -type d -name 'MarkHola.app' | head -n1)"
  [[ -n "$mounted_app" ]] || fail_closed "Unable to locate MarkHola.app inside mounted DMG"

  ditto "$mounted_app" "$APP_COPY"
  [[ -d "$APP_COPY/Contents/MacOS" ]] || fail_closed "Copied app bundle is incomplete"
  append_ui_result "dmg_identity" "PASS" "Exact UDZO DMG verified, mounted read-only, and copied"
}

verify_bundle_manifest() {
  find "$APP_COPY" -print | LC_ALL=C sort >"$BUNDLE_DIR/app-tree.txt"
  while IFS= read -r path; do
    [[ -f "$path" ]] || continue
    shasum -a 256 "$path"
  done < <(find "$APP_COPY" -type f | LC_ALL=C sort) >"$BUNDLE_DIR/app-file-sha256.txt"
}

verify_bundle_machos() {
  local report="$BUNDLE_DIR/macho-verification.txt"
  : >"$report"

  while IFS= read -r candidate; do
    local description arches minos rel
    description="$(file -b "$candidate")"
    [[ "$description" == *"Mach-O"* ]] || continue

    rel="${candidate#$APP_COPY/}"
    arches="$(lipo -archs "$candidate")"
    minos="$(xcrun vtool -arch x86_64 -show-build "$candidate" | awk '$1 == "minos" { print $2; exit }')"

    {
      echo "path=$rel"
      echo "arches=$arches"
      echo "minos=${minos:-missing}"
      echo
    } >>"$report"

    [[ "$arches" == "x86_64" ]] || fail_closed "Mach-O $rel is not x86_64-only"
    [[ "$minos" == "14.0" ]] || fail_closed "Mach-O $rel does not use minos=14.0"
  done < <(find "$APP_COPY" -type f | LC_ALL=C sort)
}

verify_version_signature_and_resources() {
  local info_plist="$APP_COPY/Contents/Info.plist"
  local expected_version short_version bundle_version bundle_exec minimum_system_version

  [[ -f "$info_plist" ]] || fail_closed "Missing Info.plist"
  expected_version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
  short_version="$(plutil -extract CFBundleShortVersionString raw -o - "$info_plist")"
  bundle_version="$(plutil -extract CFBundleVersion raw -o - "$info_plist")"
  bundle_exec="$(plutil -extract CFBundleExecutable raw -o - "$info_plist")"
  minimum_system_version="$(plutil -extract LSMinimumSystemVersion raw -o - "$info_plist")"

  {
    echo "expected_version=$expected_version"
    echo "CFBundleShortVersionString=$short_version"
    echo "CFBundleVersion=$bundle_version"
    echo "CFBundleExecutable=$bundle_exec"
    echo "LSMinimumSystemVersion=$minimum_system_version"
  } >"$BUNDLE_DIR/version.txt"

  [[ "$short_version" == "$expected_version" ]] || fail_closed "CFBundleShortVersionString mismatch"
  [[ "$bundle_version" == "$expected_version" ]] || fail_closed "CFBundleVersion mismatch"
  [[ "$bundle_exec" == "MarkHola" ]] || fail_closed "CFBundleExecutable mismatch"
  [[ "$minimum_system_version" == "14.0" ]] || fail_closed "LSMinimumSystemVersion mismatch"

  codesign --verify --deep --strict --verbose=2 "$APP_COPY" >"$BUNDLE_DIR/codesign-verify.txt" 2>&1

  compare_directory_parity "$ROOT_DIR/assets/help" "$APP_COPY/Contents/Resources/help" "help"
  compare_directory_parity "$ROOT_DIR/themes" "$APP_COPY/Contents/Resources/themes" "themes"
  cmp -s "$ROOT_DIR/assets/logo.png" "$APP_COPY/Contents/Resources/logo.png" || fail_closed "logo.png resource parity mismatch"
  [[ -f "$APP_COPY/Contents/Resources/MarkHola.icns" ]] || fail_closed "Missing generated MarkHola.icns"
  append_ui_result "bundle_identity" "PASS" "Bundle version, signature, architecture, minos, and resources passed"
}

compile_window_probe() {
  swiftc \
    "$ROOT_DIR/.github/fixtures/intel_g4_window_probe.swift" \
    -o "$WINDOW_PROBE_BIN"
}

check_gui_capabilities() {
  local gui_domain="gui/$(id -u)"
  local can_run_gui=1

  if launchctl print "$gui_domain" >"$UI_DIR/aqua-session.txt" 2>&1; then
    append_ui_result "aqua_session" "PASS" "$gui_domain is reachable"
  else
    append_ui_result "aqua_session" "BLOCKED" "No Aqua launchctl session"
    can_run_gui=0
  fi

  if pgrep -x WindowServer >"$UI_DIR/windowserver-pid.txt"; then
    append_ui_result "windowserver" "PASS" "WindowServer process is present"
  else
    append_ui_result "windowserver" "BLOCKED" "WindowServer process is missing"
    can_run_gui=0
  fi

  if "$WINDOW_PROBE_BIN" >"$UI_DIR/window-probe.json" 2>"$UI_DIR/window-probe.stderr"; then
    local visible_window ax_trusted owner_name owner_pid probe_pid
    visible_window="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts payload["visibleWindow"]' \
        "$UI_DIR/window-probe.json"
    )"
    ax_trusted="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts payload["axTrusted"]' \
        "$UI_DIR/window-probe.json"
    )"
    owner_name="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts(payload["windowOwner"].to_s)' \
        "$UI_DIR/window-probe.json"
    )"
    owner_pid="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts(payload["windowOwnerPID"].to_s)' \
        "$UI_DIR/window-probe.json"
    )"
    probe_pid="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts payload["pid"]' \
        "$UI_DIR/window-probe.json"
    )"

    if [[ "$visible_window" == "true" && "$owner_pid" == "$probe_pid" ]]; then
      append_ui_result "visible_appkit_window" "PASS" "Probe window owner=${owner_name:-unknown} owner_pid=$owner_pid"
    else
      append_ui_result "visible_appkit_window" "BLOCKED" "Probe window visibility/owner PID mismatch owner_pid=${owner_pid:-missing} probe_pid=${probe_pid:-missing}"
      can_run_gui=0
    fi

    if [[ "$ax_trusted" == "true" ]]; then
      append_ui_result "ax_tcc" "PASS" "Accessibility trust is available"
    else
      append_ui_result "ax_tcc" "BLOCKED" "Accessibility trust is unavailable"
      can_run_gui=0
    fi
  else
    append_ui_result "window_probe" "BLOCKED" "Swift AppKit window probe failed"
    can_run_gui=0
  fi

  [[ "$can_run_gui" -eq 1 ]]
}

capture_blocked_ui_placeholders() {
  printf 'BLOCKED: GUI capability gate failed before launching MarkHola.\n' >"$UI_DIR/startup.txt"
  printf 'BLOCKED: GUI capability gate failed before capturing startup log.\n' >"$STARTUP_LOG"
  printf 'BLOCKED: GUI capability gate failed before locating candidate app log path.\n' >"$APP_LOG_PATH_FILE"
  printf 'BLOCKED: GUI capability gate failed before PID capture.\n' >"$UI_DIR/process.txt"
  printf 'BLOCKED: GUI capability gate failed before lsof capture.\n' >"$UI_DIR/lsof.txt"
  printf 'BLOCKED: GUI capability gate failed before sample capture.\n' >"$UI_DIR/sample.txt"
  printf 'BLOCKED: GUI capability gate failed before runtime architecture verification.\n' >"$RUNTIME_ARCH_FILE"
  printf 'BLOCKED: GUI capability gate failed before window-owner capture.\n' >"$UI_DIR/window-owner.txt"
  printf 'BLOCKED: GUI capability gate failed before About binding.\n' >"$UI_DIR/about.txt"
}

run_gui_matrix() {
  local sample_doc="$ROOT_DIR/examples/basic.md"
  local app_pid=""
  local app_log_path=""
  local startup_ready=0
  local candidate_path

  printf 'launch_command=%q %q\n' "$APP_COPY/Contents/MacOS/MarkHola" "$sample_doc" >"$UI_DIR/startup.txt"
  "$APP_COPY/Contents/MacOS/MarkHola" "$sample_doc" >"$STARTUP_LOG" 2>&1 &
  app_pid="$!"
  echo "spawned_pid=$app_pid" >>"$UI_DIR/startup.txt"
  {
    echo "candidate_log_paths:"
    candidate_log_paths
  } >"$APP_LOG_PATH_FILE"

  for _ in {1..30}; do
    if ps -p "$app_pid" >/dev/null 2>&1; then
      while IFS= read -r candidate_path; do
        if [[ -r "$candidate_path" ]] \
          && grep -q "pid=$app_pid" "$candidate_path" 2>/dev/null \
          && grep -q "stage=" "$candidate_path" 2>/dev/null; then
          app_log_path="$candidate_path"
          startup_ready=1
          cp "$candidate_path" "$STARTUP_LOG"
          break 2
        fi
      done < <(candidate_log_paths)
    else
      app_pid=""
      break
    fi
    sleep 1
  done

  if [[ -z "$app_pid" ]]; then
    append_ui_result "launch_markhola" "BLOCKED" "No exact MarkHola PID was observed after direct launch"
    capture_blocked_ui_placeholders
    return 1
  fi

  if [[ "$startup_ready" -ne 1 || -z "$app_log_path" ]]; then
    {
      echo "app_pid=$app_pid"
      echo "status=BLOCKED"
      echo "reason=unable to locate readable candidate startup log containing pid and stage fields"
      echo "stdout_stderr_capture=$STARTUP_LOG"
    } >"$UI_DIR/startup.txt"
    printf 'BLOCKED: no readable candidate app log was captured for pid=%s\n' "$app_pid" >"$STARTUP_LOG"
    append_ui_result "startup_log_binding" "BLOCKED" "Unable to capture real candidate startup log for pid=$app_pid"
    return 1
  fi

  append_ui_result "launch_markhola" "PASS" "pid=$app_pid"
  append_ui_result "startup_log_binding" "PASS" "startup log captured from $app_log_path for pid=$app_pid"
  ps -p "$app_pid" -o pid=,ppid=,state=,etime=,command= >"$UI_DIR/process.txt"
  lsof -p "$app_pid" >"$UI_DIR/lsof.txt" 2>&1 || true
  sample "$app_pid" 2 1 -file "$UI_DIR/sample.txt" >/dev/null 2>&1 || printf 'BLOCKED: sample command failed for pid=%s\n' "$app_pid" >"$UI_DIR/sample.txt"

  if grep -Fq "$APP_COPY/Contents/MacOS/MarkHola" "$UI_DIR/process.txt"; then
    append_ui_result "executable_path_binding" "PASS" "Process command is bound to the copied candidate executable"
  else
    append_ui_result "executable_path_binding" "BLOCKED" "Process command does not prove the copied candidate executable path"
  fi

  if grep -Fq "$APP_COPY/Contents/MacOS/MarkHola" "$UI_DIR/lsof.txt"; then
    append_ui_result "lsof_binding" "PASS" "lsof binds the candidate executable to pid=$app_pid"
  else
    append_ui_result "lsof_binding" "BLOCKED" "lsof does not bind the candidate executable to pid=$app_pid"
  fi

  if grep -q 'Code Type:.*X86-64' "$UI_DIR/sample.txt" \
    && ! grep -Eqi 'translated|arm64|aarch64' "$UI_DIR/sample.txt"; then
    printf 'PASS: exact pid %s sample output confirms Code Type X86-64 with no translated/arm64 markers.\n' "$app_pid" >"$RUNTIME_ARCH_FILE"
    append_ui_result "runtime_architecture" "PASS" "sample output confirms x86_64 runtime for pid=$app_pid"
  else
    cat "$UI_DIR/sample.txt" >"$RUNTIME_ARCH_FILE"
    append_ui_result "runtime_architecture" "BLOCKED" "sample output did not prove x86_64 non-translated runtime for pid=$app_pid"
  fi

  {
    echo "sample_document=$sample_doc"
    echo "app_pid=$app_pid"
    echo "app_binary=$APP_COPY/Contents/MacOS/MarkHola"
    echo "candidate_app_log=$app_log_path"
    echo "startup_log=$STARTUP_LOG"
  } >"$UI_DIR/startup.txt"

  if "$WINDOW_PROBE_BIN" --inspect-pid "$app_pid" >"$UI_DIR/window-owner.json" 2>"$UI_DIR/window-owner.stderr"; then
    cat "$UI_DIR/window-owner.json" >"$UI_DIR/window-owner.txt"
    local owner_pid
    owner_pid="$(
      ruby -rjson -e 'payload = JSON.parse(File.read(ARGV[0])); puts(payload["windowOwnerPID"].to_s)' \
        "$UI_DIR/window-owner.json"
    )"
    if [[ "$owner_pid" == "$app_pid" ]]; then
      append_ui_result "window_owner_binding" "PASS" "Captured window-owner JSON for pid=$app_pid"
    else
      append_ui_result "window_owner_binding" "BLOCKED" "CGWindow owner PID mismatch owner_pid=${owner_pid:-missing} app_pid=$app_pid"
    fi
  else
    printf 'BLOCKED: unable to inspect CGWindow owner for pid=%s\n' "$app_pid" >"$UI_DIR/window-owner.txt"
    append_ui_result "window_owner_binding" "BLOCKED" "Unable to inspect visible window ownership"
  fi

  if osascript - "$app_pid" >"$UI_DIR/about.txt" 2>"$UI_DIR/about.stderr" <<'APPLESCRIPT'
on run argv
  set targetPid to item 1 of argv
tell application "System Events"
  set targetProcess to first application process whose unix id is (targetPid as integer)
  tell targetProcess
    click menu item "About MarkHola" of menu 1 of menu bar item "MarkHola" of menu bar 1
    delay 1
    get {unix id, name of every window}
  end tell
end tell
end run
APPLESCRIPT
  then
    append_ui_result "about_panel" "PASS" "About MarkHola menu action succeeded for pid=$app_pid"
  else
    append_ui_result "about_panel" "BLOCKED" "Unable to drive About MarkHola through System Events"
  fi

  if osascript - "$app_pid" >"$UI_DIR/menu-tab.txt" 2>"$UI_DIR/menu-tab.stderr" <<'APPLESCRIPT'
on run argv
  set targetPid to item 1 of argv
tell application "System Events"
  set targetProcess to first application process whose unix id is (targetPid as integer)
  tell targetProcess
    get name of every menu item of menu 1 of menu bar item "Tab" of menu bar 1
  end tell
end tell
end run
APPLESCRIPT
  then
    append_ui_result "tab_menu" "PASS" "Tab menu items were enumerated"
  else
    append_ui_result "tab_menu" "BLOCKED" "Unable to enumerate Tab menu items"
  fi

  if osascript - "$app_pid" >"$UI_DIR/menu-window.txt" 2>"$UI_DIR/menu-window.stderr" <<'APPLESCRIPT'
on run argv
  set targetPid to item 1 of argv
tell application "System Events"
  set targetProcess to first application process whose unix id is (targetPid as integer)
  tell targetProcess
    get name of every menu item of menu 1 of menu bar item "Window" of menu bar 1
  end tell
end tell
end run
APPLESCRIPT
  then
    append_ui_result "window_menu" "PASS" "Window menu items were enumerated"
  else
    append_ui_result "window_menu" "BLOCKED" "Unable to enumerate Window menu items"
  fi

  if osascript - "$app_pid" >"$UI_DIR/menu-export.txt" 2>"$UI_DIR/menu-export.stderr" <<'APPLESCRIPT'
on run argv
  set targetPid to item 1 of argv
tell application "System Events"
  set targetProcess to first application process whose unix id is (targetPid as integer)
  tell targetProcess
    get name of every menu item of menu 1 of menu item "Export" of menu 1 of menu bar item "File" of menu bar 1
  end tell
end tell
end run
APPLESCRIPT
  then
    append_ui_result "export_menu" "PASS" "Export submenu items were enumerated"
  else
    append_ui_result "export_menu" "BLOCKED" "Unable to enumerate Export submenu items"
  fi

  if osascript - "$app_pid" >"$UI_DIR/menu-print.txt" 2>"$UI_DIR/menu-print.stderr" <<'APPLESCRIPT'
on run argv
  set targetPid to item 1 of argv
tell application "System Events"
  set targetProcess to first application process whose unix id is (targetPid as integer)
  tell targetProcess
    exists menu item "Print" of menu 1 of menu bar item "File" of menu bar 1
  end tell
end tell
end run
APPLESCRIPT
  then
    append_ui_result "print_menu" "PASS" "Print menu item probe completed"
  else
    append_ui_result "print_menu" "BLOCKED" "Unable to probe Print menu item"
  fi
}

finalize_result() {
  bash "$MATRIX_GATE" "$UI_MATRIX_FILE" "$BLOCKERS_FILE" "$SUMMARY_FILE"
}

write_step_summary() {
  {
    echo "## Intel G4 Candidate Validation"
    echo
    cat "$SUMMARY_FILE"
    echo
    echo '```tsv'
    cat "$UI_MATRIX_FILE"
    echo '```'
    if [[ -s "$BLOCKERS_FILE" ]]; then
      echo
      echo "### Residuals"
      cat "$BLOCKERS_FILE"
    fi
  } >>"$GITHUB_STEP_SUMMARY"
}

run_static_self_test() {
  local test_script="$ROOT_DIR/.github/scripts/test_intel_g4_candidate_validation.sh"
  if [[ -f "$test_script" ]]; then
    bash "$test_script"
  fi
}

require_command gh
require_command shasum
require_command hdiutil
require_command lipo
require_command xcrun
require_command codesign
require_command plutil
require_command diff
require_command cmp
require_command swiftc
require_command osascript
require_command sample
require_command lsof
require_command grep
require_command awk

run_static_self_test
validate_sha_input
collect_provenance
download_draft_asset
verify_runner_identity
verify_dmg_and_copy_app
verify_bundle_manifest
verify_bundle_machos
verify_version_signature_and_resources
compile_window_probe

if check_gui_capabilities; then
  run_gui_matrix || true
else
  capture_blocked_ui_placeholders
fi

final_status=0
finalize_result || final_status=$?
write_step_summary
exit "$final_status"
