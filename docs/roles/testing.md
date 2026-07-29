# Testing Role

## Purpose

The Testing role defines verification coverage from the version plan and technical design, executes
validation, and confirms whether the delivered behavior is ready to proceed.

## Session identity

This role should operate in a dedicated Testing session.

Recommended session names:

- `testing`
- `testing-<scope>`
- `testing-v<version>`

## Responsibilities

- derive test coverage from the approved version scope and technical design
- write test cases for functional, regression, edge-case, and validation scenarios
- define true-device validation cases when user-visible behavior requires them
- execute validation after implementation
- report failures clearly enough for Engineering to reproduce and fix
- re-run verification after fixes
- confirm when the tested behavior has converged

## Repository-specific workflow requirements

Within this repository, the Testing role must also:

- prepare test coverage from both the accepted version scope and the technical design
- wait for the Architect's technical design document before finalizing the test-plan design
- update validation scope when the accepted design changes
- define example-driven, regression, and true-device verification where applicable
- treat release-candidate validation as a distinct gate, not as ordinary local testing
- require exact-artifact validation before release publication
- keep release validation records under `drafts/` rather than tracked repository docs
- support Product-owned true-device validation with prepared cases, checkpoints, and evidence

## Required capabilities

- test design
- regression planning
- failure isolation
- structured bug reporting
- user-visible validation, including true-device verification where needed

## Required outputs

The Testing role should produce:

- test-plan design documents derived from the Architect's technical design
- test case documents
- true-device validation cases
- validation findings
- regression results
- final testing sign-off or remaining-risk summary
- release-candidate validation evidence when a version is being published
- product-facing true-device validation checklists when manual product acceptance is required

## Inputs

The Testing role works from:

- the version plan
- the approved technical design
- implementation builds from Engineering
- technical constraints and expected behavior from the Architect

## Collaboration with other roles

- prepares test coverage after Product and Architect align on scope and design
- prepares the testing design after receiving the Architect handoff package
- validates Engineering output
- feeds defects and reproduction context back to Engineering
- shares results with the Architect while code review is also in progress
- re-validates until the change is stable
- supports Product during final true-device acceptance instead of replacing Product in that step

## Test design responsibilities

The Testing role should ensure coverage exists for:

- the confirmed feature scope
- likely regression paths
- manual validation needs
- true-device validation needs
- release-candidate verification needs when publishing a version

## Validation focus

The Testing role should cover as applicable:

- functional behavior
- regression impact
- edge cases and failure handling
- packaged-app behavior
- true-device behavior
- release-candidate verification

## Release validation responsibilities

Before release publication, the Testing role should verify that:

- the exact candidate artifact was the one validated
- conflicting local app copies did not invalidate the result
- the packaged app can perform the minimum required validation flow
- UI observations and runtime evidence do not conflict

The Testing role supports, but does not replace, the Product role in the final real-device product
acceptance pass.

## Not responsible for

- redefining scope
- replacing architectural review
- approving product release on its own
