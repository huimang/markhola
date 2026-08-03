#!/bin/zsh
set -euo pipefail
[[ $# -eq 1 && -f "$1" ]] || { print -u2 -- "Usage: $0 LOG"; exit 2; }
typeset -A statuses
while IFS= read -r line; do
  if [[ "$line" =~ '^CASE id=([^ ]+) status=(PASS|FAIL|BLOCKED|UNSET)' ]]; then
    statuses["${match[1]}"]="${match[2]}"
  fi
done < "$1"
overall="PASS"
for id case_status in ${(kv)statuses}; do
  case "$case_status" in
    FAIL) overall=FAIL; break;;
    BLOCKED) [[ "$overall" = PASS ]] && overall=BLOCKED;;
    UNSET) [[ "$overall" = PASS ]] && overall=UNSET;;
  esac
done
print -r -- "SUMMARY status=$overall cases=${#statuses} release_mutation=NONE"
[[ "$overall" = PASS ]] || exit 1
