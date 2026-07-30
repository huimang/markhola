#!/usr/bin/env bash

set -euo pipefail

readonly REQUIRED_G4_CHECKS=(
  candidate_sha256
  runner_identity
  dmg_identity
  bundle_identity
  launch_markhola
  startup_log_binding
  executable_path_binding
  lsof_binding
  runtime_architecture
  window_owner_binding
  about_panel
  open_edit_save
  readonly_rendering
  markdown_code_rendering
  mermaid_math_image_rendering
  pdf_html_print
  menu_tab_window
  theme_language_help
  stability_exit
)

if [[ "${1:-}" == "--print-required" ]]; then
  printf '%s\n' "${REQUIRED_G4_CHECKS[@]}"
  exit 0
fi

if [[ "$#" -ne 3 ]]; then
  echo "Usage: $0 MATRIX_FILE BLOCKERS_FILE SUMMARY_FILE" >&2
  exit 2
fi

matrix_file="$1"
blockers_file="$2"
summary_file="$3"

[[ -f "$matrix_file" ]] || {
  echo "Missing Intel G4 matrix: $matrix_file" >&2
  exit 2
}
touch "$blockers_file" "$summary_file"

gate_blocked=0

record_gate_blocker() {
  local check_name="$1"
  local detail="$2"
  printf '%s\tBLOCKED\t%s\n' "$check_name" "$detail" >>"$matrix_file"
  printf '%s\t%s\n' "$check_name" "$detail" >>"$blockers_file"
  gate_blocked=1
}

for check_name in "${REQUIRED_G4_CHECKS[@]}"; do
  row_counts="$(
    awk -F $'\t' -v check_name="$check_name" '
      NR > 1 && $1 == check_name {
        total += 1
        if ($2 == "PASS") {
          passed += 1
        }
      }
      END {
        printf "%d:%d", total, passed
      }
    ' "$matrix_file"
  )"

  if [[ "$row_counts" != "1:1" ]]; then
    record_gate_blocker \
      "required_${check_name}" \
      "Required G4 row $check_name must occur exactly once with PASS; observed $row_counts"
  fi
done

if awk -F $'\t' 'NR > 1 && $2 != "PASS" { found = 1 } END { exit !found }' "$matrix_file"; then
  gate_blocked=1
fi

if [[ "$gate_blocked" -ne 0 ]]; then
  if [[ ! -s "$blockers_file" ]]; then
    printf 'matrix_gate\tIntel G4 matrix contains a non-PASS result\n' >>"$blockers_file"
  fi
  printf 'overall=BLOCKED\n' >>"$summary_file"
  exit 1
fi

printf 'overall=PASS\n' >>"$summary_file"
