# Device Validation Cases

Add one executable `*.sh` per feature or acceptance boundary. Each case declares
`CASE_ID`, `CASE_TITLE`, `CASE_TAGS`, optional `CASE_TIMEOUT`, and a `case_run`
function receiving its private evidence directory.

Cases must emit `EXPECT id=... key=... actual=...` and exactly one
`CASE_RESULT status=PASS|FAIL|BLOCKED|UNSET`. Ordinary prose is never parsed as
an outcome. Objective command errors, missing evidence, and timeouts are FAIL;
environment limitations are BLOCKED. Set `CASE_MANUAL=1` for GUI, visual,
trackpad, or true-Intel work; the runner creates an UNSET Product checklist and
never promotes subjective text to PASS. Retry with a new evidence directory so
old evidence is never overwritten. Release mutation is always NONE.
