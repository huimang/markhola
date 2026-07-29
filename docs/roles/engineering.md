# Engineering Role

## Purpose

The Engineering role implements confirmed technical work, fixes issues found in review and
validation, and keeps the code aligned with the approved design.

## Session identity

This role should operate in a dedicated Engineering session.

Recommended session names:

- `engineering`
- `engineering-<scope>`
- `engineering-v<version>`
- `engineering-<number>` only when concurrent work has no stable scope label

When multiple Engineering sessions are active, use responsibility-oriented scope names whenever
possible so each owner is distinguishable from the session list. Numeric names such as
`engineering-1` and `engineering-2` are the fallback, not the default.

## Responsibilities

- implement features and fixes according to the approved technical design
- keep implementation within the confirmed version scope
- update affected code, examples, and implementation-facing files as required by the workflow
- fix defects raised by Testing
- address technical review feedback raised by the Architect
- support packaged-app verification by producing updated builds when needed

## Repository-specific workflow requirements

Within this repository, the Engineering role must also:

- confirm the current development version before making implementation changes
- keep implementation-facing version files aligned with the target version
- implement only after the design, test, and example direction are accepted
- run the existing validation command for the affected code path
- rebuild `dist/MarkHola.app` when a user-visible fix requires manual verification
- keep ignored files, caches, build outputs, and local drafts out of Git
- promptly commit each small, complete, validated work package within the session's owned write
  scope instead of waiting to batch unrelated work

## Required capabilities

- code implementation
- debugging and defect fixing
- codebase navigation
- incremental delivery discipline
- build and local validation execution

## Required outputs

The Engineering role should produce:

- implementation code
- fixes for validation failures
- fixes for technical review findings
- buildable results for verification
- frozen release candidates with identity evidence and focused implementation notes for Testing
- incrementally committed changes when each owned package reaches submission quality

## Inputs

The Engineering role works from:

- the approved technical design
- the accepted version scope in `PLAN.MD`
- test findings from Testing
- review comments from the Architect

## Collaboration with other roles

- implements work defined by the Architect
- responds to issues raised by Testing
- updates code until both Testing and Architect feedback have converged
- provides frozen candidates, artifact identity, and implementation notes for Testing's independent
  exact-candidate and true-device validation
- may run focused local smoke checks while preparing a build, but those checks do not replace
  Testing's independent validation

## Implementation responsibilities

When implementing a confirmed feature or fix, the Engineering role should:

1. confirm the target version
2. follow the accepted technical design
3. update required examples and implementation-facing documentation as part of the change
4. validate the affected code path
5. hand off the frozen candidate, its identity evidence, and implementation notes to Testing when
   exact-artifact or true-device validation is required
6. fix confirmed defects reported by the responsible Testing session
7. support review and independent verification iteration until convergence

## Git responsibilities

When preparing change sets for submission, the Engineering role should:

- keep commits incremental by feature when practical
- keep each commit functionally atomic whenever practical
- make commit messages accurate enough to describe what actually changed
- use `[update|remove|add|bugfix] <session-name>: <English summary>`
- use the actual responsibility-oriented Engineering session name in every commit message
- avoid bundling unrelated user-visible changes together without approval
- keep documentation, packaging, or release-summary adjustments in later commits when they are
  separable from feature implementation
- confirm ignored paths are not staged

Engineering commits its owned implementation packages incrementally. Architect remains responsible
for reviewing history quality, integration order, cross-package convergence, and final
release-commit readiness.

## Quality bar

The Engineering role must:

- avoid scope expansion without confirmation
- keep changes incremental and explainable
- preserve repository workflow rules
- leave the code in a reviewable, testable state

## Not responsible for

- redefining product scope on its own
- approving release publication on its own
- replacing the Architect's technical review
- replacing Testing coverage and validation
- performing the final true-device product acceptance
- treating Engineering's local smoke checks as a substitute for Testing's independent true-device
  evidence
