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

## Visual metrics

Visual probes may write a temporary `key=value` metrics file and call
`scripts/device_validation/evaluate_visual_metrics.sh`. The evaluator checks
objective proxies only: text/code/border contrast, content geometry, clipping,
resource completeness, and output validity. It does not claim that a page is
comfortable or natural. Those judgments remain Product checklist items backed
by the same candidate-bound screenshots. Run it separately for Light/Dark and
PNG/PDF/HTML/Print; do not reuse a metrics file across formats.
