#!/bin/zsh
set -euo pipefail
[[ $# -eq 1 && -f "$1" ]] || { print -u2 -- "Usage: $0 METRICS_FILE"; exit 2; }
typeset -A value
while IFS='=' read -r key metric; do
  [[ -z "$key" || "$key" == \#* ]] && continue
  [[ "$key" =~ '^[a-z][a-z0-9_]*$' ]] || { print -u2 -- "invalid metric key: $key"; exit 2; }
  value[$key]="$metric"
done < "$1"
failures=()
require() { if [[ -z "${value[$1]:-}" ]]; then failures+=("missing:$1"); fi; return 0; }
number_at_least() {
  local key="$1" minimum="$2" actual="${value[$1]:-}"
  require "$key"
  [[ -n "$actual" && "$actual" =~ '^[0-9]+([.][0-9]+)?$' ]] || { failures+=("invalid:$key"); return; }
  if (( actual + 0 < minimum )); then failures+=("below:$key:$actual<$minimum"); fi
  return 0
}
equals() { if [[ "${value[$1]:-}" != "$2" ]]; then failures+=("expected:$1=$2"); fi; return 0; }
number_at_least body_text_contrast 4.5
number_at_least code_text_contrast 6
number_at_least border_contrast 3
number_at_least content_width 1
number_at_least line_height 1
equals no_edge_clipping true
equals resources_complete true
equals output_valid true
require theme
require format
if (( ${#failures} > 0 )); then
  print -r -- "VISUAL_METRICS status=FAIL reasons=${(j:,:)failures}"; exit 1
fi
print -r -- "VISUAL_METRICS status=PASS theme=${value[theme]} format=${value[format]}"
for key in body_text_contrast code_text_contrast border_contrast content_width no_edge_clipping resources_complete; do
  print -r -- "EXPECT key=$key actual=${value[$key]}"
done
