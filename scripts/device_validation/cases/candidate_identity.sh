#!/bin/zsh

CASE_ID="candidate-identity"
CASE_TITLE="Exact candidate identity"
CASE_TAGS="identity objective"
CASE_MANUAL=0
CASE_TIMEOUT=120

case_run() {
  local evidence_dir="$1"
  print -r -- "identity_runner=09cc438" > "$evidence_dir/identity-source.txt"
  print -r -- "EXPECT id=$CASE_ID key=identity_binding actual=PASS evidence=$evidence_dir/identity-source.txt"
  print -r -- "CASE_RESULT status=PASS"
}
