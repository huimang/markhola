#!/bin/zsh
set -euo pipefail

ROOT_DIR="$(cd "${0:A:h}/../.." && pwd)"
CASES_DIR="${DEVICE_VALIDATION_CASES_DIR:-$ROOT_DIR/scripts/device_validation/cases}"
APPLE_DMG=""; INTEL_DMG=""; APPLE_SHA=""; INTEL_SHA=""; EVIDENCE_DIR=""
CASE_FILTER=""; TAG_FILTER=""; TIMEOUT=300
usage() { print -u2 -- "Usage: $0 --apple-dmg PATH --intel-dmg PATH --apple-sha SHA --intel-sha SHA [--evidence-dir DIR] [--case-id ID] [--tag TAG] [--timeout SECONDS]"; }
die() { print -u2 -- "ERROR: $*"; exit 2; }

run_with_timeout() {
  local timeout_seconds="$1"
  shift

  if command -v timeout >/dev/null 2>&1; then
    timeout "$timeout_seconds" "$@"
    return $?
  fi

  if ! command -v perl >/dev/null 2>&1; then
    return 125
  fi

  perl -MPOSIX=setsid -e '
    use strict;
    use warnings;

    my $timeout = shift @ARGV;
    my $pid = fork();
    defined $pid or exit 125;

    if ($pid == 0) {
      setsid() or exit 125;
      exec @ARGV or exit 125;
    }

    local $SIG{ALRM} = sub {
      kill "TERM", -$pid;
      select undef, undef, undef, 0.2;
      kill "KILL", -$pid;
      waitpid($pid, 0);
      exit 124;
    };

    alarm $timeout;
    waitpid($pid, 0);
    my $status = $?;
    alarm 0;

    if (($status & 127) != 0) {
      exit 128 + ($status & 127);
    }

    exit($status >> 8);
  ' "$timeout_seconds" "$@"
}
while [[ $# -gt 0 ]]; do
  case "$1" in
    --apple-dmg|--intel-dmg|--apple-sha|--intel-sha|--evidence-dir|--case-id|--tag|--timeout)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      case "$1" in
        --apple-dmg) APPLE_DMG="$2";; --intel-dmg) INTEL_DMG="$2";; --apple-sha) APPLE_SHA="$2";; --intel-sha) INTEL_SHA="$2";;
        --evidence-dir) EVIDENCE_DIR="$2";; --case-id) CASE_FILTER="$2";; --tag) TAG_FILTER="$2";; --timeout) TIMEOUT="$2";;
      esac
      shift 2
      ;;
    -h|--help) usage; exit 0;; *) usage; exit 2;;
  esac
done
[[ "$APPLE_DMG" = /* && "$INTEL_DMG" = /* ]] || die "candidate DMG paths must be absolute"
[[ "$APPLE_SHA" =~ '^[a-fA-F0-9]{64}$' && "$INTEL_SHA" =~ '^[a-fA-F0-9]{64}$' ]] || die "candidate SHA values must be 64 hexadecimal characters"
[[ "$TIMEOUT" =~ '^[1-9][0-9]*$' ]] || die "timeout must be a positive integer"
[[ -d "$CASES_DIR" ]] || die "case directory does not exist: $CASES_DIR"
if [[ -z "$EVIDENCE_DIR" ]]; then
  EVIDENCE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/markhola-device-validation.XXXXXX")"
else
  [[ "$EVIDENCE_DIR" = /* && ! -e "$EVIDENCE_DIR" ]] || die "evidence directory must be new and absolute"
  mkdir -p "$EVIDENCE_DIR"
fi
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-$$"; RUN_DIR="$EVIDENCE_DIR/$RUN_ID"; mkdir -p "$RUN_DIR/cases"
LOG="$RUN_DIR/validation.log"; exec > >(tee "$LOG") 2>&1
print -r -- "RUN start=$RUN_ID candidate_apple_sha=$APPLE_SHA candidate_intel_sha=$INTEL_SHA release_mutation=NONE"
EXACT_RUNNER="${DEVICE_VALIDATION_EXACT_RUNNER:-$ROOT_DIR/scripts/run_exact_candidate_validation.sh}"; IDENTITY_DIR="$RUN_DIR/candidate-identity"
[[ -x "$EXACT_RUNNER" ]] || { print -r -- "IDENTITY status=FAIL reason=missing_exact_runner"; exit 1; }
if "$EXACT_RUNNER" --apple-dmg "$APPLE_DMG" --intel-dmg "$INTEL_DMG" --apple-sha "$APPLE_SHA" --intel-sha "$INTEL_SHA" --evidence-dir "$IDENTITY_DIR"; then
  print -r -- "IDENTITY status=PASS evidence=$IDENTITY_DIR"
else
  print -r -- "IDENTITY status=FAIL evidence=$IDENTITY_DIR"; print -r -- "SUMMARY status=FAIL reason=exact_candidate_identity"; exit 1
fi
typeset -a case_files results; case_files=("$CASES_DIR"/*.sh(N)); typeset -A seen
for case_file in $case_files; do
  unset CASE_ID CASE_TITLE CASE_TAGS CASE_MANUAL CASE_TIMEOUT
  source "$case_file"
  [[ -n "${CASE_ID:-}" ]] || die "case missing CASE_ID: $case_file"
  [[ -z "${seen[$CASE_ID]:-}" ]] || die "duplicate case id: $CASE_ID"
  seen[$CASE_ID]="$case_file"
  [[ -z "$CASE_FILTER" || "$CASE_ID" = "$CASE_FILTER" ]] || continue
  [[ -z "$TAG_FILTER" || " ${CASE_TAGS:-} " == *" $TAG_FILTER "* ]] || continue
  case_dir="$RUN_DIR/cases/$CASE_ID"; mkdir -p "$case_dir"; case_log="$case_dir/case.log"
  print -r -- "CASE id=$CASE_ID status=RUNNING evidence=$case_dir"; case_status="PASS"
  if [[ "${CASE_MANUAL:-0}" = 1 ]]; then
    case_status="UNSET"; print -r -- "CASE id=$CASE_ID status=UNSET reason=manual_only evidence=$case_dir"; print -r -- "EXPECT id=$CASE_ID key=manual actual=UNSET evidence=$case_dir/product-checklist.md"; print -r -- "manual_only=true" > "$case_dir/product-checklist.md"
  else
    if run_with_timeout "${CASE_TIMEOUT:-$TIMEOUT}" zsh -c 'source "$1"; case_run "$2"' zsh "$case_file" "$case_dir" > >(tee "$case_log") 2>&1; then
      case_exit=0
    else
      case_exit=$?
    fi
    if [[ $case_exit -eq 124 ]]; then
      case_status="FAIL"
      print -r -- "CASE_RESULT status=FAIL reason=timeout" >> "$case_log"
      print -r -- "CASE id=$CASE_ID status=FAIL reason=timeout evidence=$case_log"
    elif [[ $case_exit -eq 125 ]]; then
      case_status="BLOCKED"
      print -r -- "CASE_RESULT status=BLOCKED reason=timeout_infrastructure" >> "$case_log"
      print -r -- "CASE id=$CASE_ID status=BLOCKED reason=timeout_infrastructure evidence=$case_log"
    elif [[ $case_exit -ne 0 ]]; then
      case_status="FAIL"
      print -r -- "CASE id=$CASE_ID status=FAIL reason=command_failure evidence=$case_log"
    elif ! grep -Eq '^CASE_RESULT status=(PASS|FAIL|BLOCKED|UNSET)( |$)' "$case_log"; then
      case_status="FAIL"; print -r -- "CASE id=$CASE_ID status=FAIL reason=missing_structured_result evidence=$case_log"
    else
      case_status="$(grep -E '^CASE_RESULT status=' "$case_log" | tail -1 | sed 's/^CASE_RESULT status=//' | awk '{print $1}')"; print -r -- "CASE id=$CASE_ID status=$case_status evidence=$case_log"
    fi
  fi
  results+=("$CASE_ID|$case_status|$case_dir")
done
priority() { case "$1" in FAIL) print 4;; BLOCKED) print 3;; UNSET) print 2;; PASS) print 1;; *) print 5;; esac; }
overall="PASS"
for result in $results; do case_status="${result#*|}"; case_status="${case_status%%|*}"; (( $(priority "$case_status") > $(priority "$overall") )) && overall="$case_status"; done
print -r -- "SUMMARY status=$overall cases=${#results[@]} release_mutation=NONE"
SUMMARY_JSON="$RUN_DIR/summary.json"; SUMMARY_MD="$RUN_DIR/summary.md"
{ print '{'; print "  \"schema_version\": 1,"; print "  \"run_id\": \"$RUN_ID\","; print "  \"status\": \"$overall\","; print '  \"release_mutation\": \"NONE\",'; print '  \"cases\": ['; integer i=0
  for result in $results; do IFS='|' read id case_status dir <<< "$result"; ((++i)); [[ $i -gt 1 ]] && print ','; print "    {\"id\":\"$id\",\"status\":\"$case_status\",\"evidence\":\"$dir\"}"; done
  print '  ]'; print '}'
} > "$SUMMARY_JSON"
{
  print -r -- "# Device Validation Run $RUN_ID"
  print
  print -r -- "- Status: **$overall**"
  print -r -- "- Release mutation: **NONE**"
  print -r -- "- Candidate SHA: Apple $APPLE_SHA; Intel $INTEL_SHA"
  print
  print -r -- "## Cases"
  for result in $results; do
    IFS='|' read id case_status dir <<< "$result"
    printf '%s\n' "- \`$id\`: **$case_status** ($dir)"
  done
} > "$SUMMARY_MD"
print -r -- "RUN end=$RUN_ID status=$overall summary_json=$SUMMARY_JSON summary_md=$SUMMARY_MD"; [[ "$overall" = PASS ]] || exit 1
