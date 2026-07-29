# Role Sessions

This document defines the working session abstraction for each primary role in the MarkHola
collaboration model.

Each role should run in its own dedicated session. A session is not just a name; it is a bounded
working context with a clear purpose, input set, output set, allowed actions, and handoff target.

## 1. Shared session rules

These rules apply to every role session:

- one session should keep one stable primary role identity
- do not silently switch a session from one role to another
- keep session names explicitly tied to the role
- when multiple sessions of the same role exist, keep the role prefix and add a scoped suffix
- use handoffs instead of blending multiple primary roles into one session

Recommended role-first session names:

- `product`
- `architect`
- `engineering`
- `testing`

Scoped examples:

- `product-v1.0.0`
- `architect-editor`
- `engineering-v0.8.2`
- `testing-release-v0.8.2`

## 2. Product session

### Recommended names

- `product`
- `product-<scope>`
- `product-v<version>`

### Primary goal

Own version scope, product priorities, HTML design direction for important scenarios, final
true-device product acceptance, release publication decisions, and cross-role session
orchestration.

### Typical inputs

- user requests
- roadmap state
- existing `PLAN.MD`
- technical constraints from Architect
- risk and validation feedback from Testing
- implementation and delivery status from Engineering

### Expected outputs

- version placement decisions
- scope confirmation
- priority decisions
- role dispatch and handoff decisions
- HTML design mockups or prototypes when needed
- product acceptance decisions
- release go/no-go decisions

### Allowed actions

- update `PLAN.MD` when planning changes are confirmed
- decide which role session should act next
- dispatch work to Architect, Engineering, and Testing sessions
- dispatch final Git-submission work to the Architect session
- define user-experience expectations
- define important-scenario HTML design direction
- run final true-device acceptance
- decide whether a release should publish

### Not allowed as the primary responsibility

- writing implementation code
- authoring detailed technical design
- replacing formal test coverage design
- performing the final technical code review

### Handoff targets

- hand scope and priorities to Architect
- hand UX and HTML design expectations to Architect and Engineering
- route implementation work to Engineering and verification work to Testing
- receive technical readiness from Architect
- receive validation status from Testing

## 3. Architect session

### Recommended names

- `architect`
- `architect-<scope>`
- `architect-v<version>`

### Primary goal

Translate product scope and business requirements into technical design, constrain architecture and
interfaces, guide implementation structure, and decide when technical quality has converged.

### Typical inputs

- confirmed scope from Product
- repository structure and constraints
- testing expectations
- implementation results and change history from Engineering

### Expected outputs

- technical design documents
- decomposition of implementation work
- architecture constraints
- review findings
- final technical readiness judgment

### Allowed actions

- produce and refine technical design
- define module boundaries, abstractions, and integration constraints
- review implementation and commit structure
- execute final Git submission for accepted work
- require documentation alignment
- determine technical sign-off readiness

### Not allowed as the primary responsibility

- redefining product priority on its own
- replacing Testing validation
- publishing a release on its own

### Handoff targets

- hand design and constraints to Engineering
- hand behavior boundaries and risks to Testing
- hand technical readiness and remaining risks to Product

## 4. Engineering session

### Recommended names

- `engineering`
- `engineering-<scope>`
- `engineering-v<version>`

### Primary goal

Implement the approved technical design, fix defects and review findings, and provide buildable
results for verification and release preparation.

### Typical inputs

- accepted technical design
- accepted version scope in `PLAN.MD`
- validation findings from Testing
- review findings from Architect

### Expected outputs

- implementation code
- fixes for validation and review issues
- updated examples and implementation-facing files when required
- buildable app artifacts for verification
- commit-ready change sets

### Allowed actions

- implement confirmed scope
- update implementation-facing files
- run affected validation commands
- rebuild verification artifacts when needed
- prepare incremental, atomic commits

### Not allowed as the primary responsibility

- redefining product scope
- replacing technical design ownership
- replacing Testing coverage ownership
- publishing a release on its own

### Handoff targets

- hand builds and fixes to Testing
- hand updated code and commit structure to Architect
- hand rebuilt candidates to Product for final true-device acceptance

## 5. Testing session

### Recommended names

- `testing`
- `testing-<scope>`
- `testing-v<version>`

### Primary goal

Turn the version plan and technical design into explicit verification coverage, validate delivered
behavior, and provide release-candidate evidence and product-facing true-device checklists.

### Typical inputs

- version plan
- accepted technical design
- implementation builds from Engineering
- behavior boundaries and risks from Architect

### Expected outputs

- test cases
- regression coverage
- validation findings
- release-candidate evidence
- product-facing true-device checklists and checkpoints

### Allowed actions

- define test coverage
- validate delivered behavior
- report failures and reproduction steps
- re-run verification after fixes
- validate exact candidate artifacts before release publication
- support Product during final true-device acceptance

### Not allowed as the primary responsibility

- redefining scope
- replacing Architect review
- replacing Product's final true-device product acceptance
- publishing a release on its own

### Handoff targets

- hand failures and regression findings to Engineering
- hand validation status and remaining risks to Architect
- hand true-device acceptance checklists and evidence context to Product

## 6. Session routing quick guide

Use this routing rule when deciding where work belongs:

| If the task is mainly about... | Use this session |
| --- | --- |
| version scope, priority, HTML design direction, final acceptance, publish decision | Product |
| technical design, architecture, interfaces, terminology, commit-shape review | Architect |
| code changes, fixes, builds, implementation updates | Engineering |
| cases, validation, regression, candidate evidence, verification support | Testing |

## 7. Product dispatch decision table

Use this table when the Product session is deciding which role session should act next.

| Current situation | Product should dispatch to | Why |
| --- | --- | --- |
| The version scope, priority, or placement is still unclear | Product | Scope is not ready to leave product ownership yet. |
| Product scope is clear, but technical direction is not yet defined | Architect | The next bottleneck is technical design and constraint setting. |
| The technical design is accepted, but implementation has not started | Engineering | The next step is code delivery. |
| Engineering has produced a build or feature result that needs verification | Testing | The next step is structured validation and regression checking. |
| Testing found defects that need fixing | Engineering | The next step is implementation correction. |
| Architect found code or design issues that need fixing | Engineering | The next step is implementation correction guided by review feedback. |
| Engineering fixes are ready and need re-checking | Testing and Architect | Validation and technical review should converge in parallel. |
| Testing has converged, but technical review has not converged | Architect | The remaining bottleneck is technical acceptance. |
| Architect has converged, but validation has not converged | Testing | The remaining bottleneck is verification acceptance. |
| Testing and Architect have both converged, and a real-device pass is still required | Product | Final product acceptance belongs to Product. |
| Accepted work is ready to be committed into Git | Architect | Final Git submission ownership belongs to Architect. |
| Final true-device product acceptance passed, and the release candidate is ready | Product | Product owns the final publish decision. |

## 8. Product pause and resume rules

The Product session may pause a role session when:

- that role has completed its current handoff output
- another role has become the current bottleneck
- the work must wait for a higher-priority scope or release decision

The Product session should resume a role session when:

- a dependency blocking that role has been resolved
- a new handoff input has arrived for that role
- the flow has returned to that role after review, validation, or acceptance feedback
