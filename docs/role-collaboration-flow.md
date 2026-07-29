# Role Collaboration Flow

This document defines how the Product, Architect, Engineering, and Testing roles collaborate on a
MarkHola version change.

## 1. Roles

- Product — owns version scope, cross-role orchestration, and release publication
- Architect — owns technical design, technical review, and final code-quality convergence
- Engineering — owns implementation and fixes
- Testing — owns test coverage, validation, and re-validation

## 2. Session model

Each role should operate in its own dedicated session. Do not mix multiple primary roles into the
same session when the intent is role-based collaboration.

Required session-to-role mapping:

- one Product session
- one Architect session
- one Engineering session
- one Testing session

Session names must stay explicitly associated with the role. Use role-first naming such as:

- `product`
- `architect`
- `engineering`
- `testing`

When multiple sessions of the same role are needed, keep the role prefix and add a scoped suffix,
for example:

- `architect-v1.0.0`
- `engineering-editor`
- `testing-release-v0.8.2`

The role identity of a session should remain stable throughout the work. Do not silently switch a
session from one primary role to another.

The Product session is the default coordinator for the role sessions. Product decides which role
session should act next and when work should be handed off across roles. Use the dispatch rules in
`docs/role-sessions.md` when deciding the next session.

Role details live in:

- `docs/roles/product.md`
- `docs/roles/architect.md`
- `docs/roles/engineering.md`
- `docs/roles/testing.md`
- `docs/role-sessions.md`

## 3. Sequence overview

```mermaid
sequenceDiagram
  participant Product
  participant Architect
  participant Testing
  participant Engineering

  Product->>Architect: Dispatch scope, release intent, and next-step ownership
  Architect->>Product: Align scope boundaries and technical direction
  Architect->>Testing: Share technical design and risk boundaries
  Architect->>Engineering: Share design, task breakdown, and constraints
  Testing->>Testing: Prepare test cases, regression cases, and true-device cases
  Engineering->>Engineering: Implement the confirmed scope

  par Validation and review
    Testing->>Engineering: Report validation findings and reproduction details
    Engineering->>Testing: Submit fixes for re-validation
  and Technical review
    Architect->>Engineering: Report code review and architecture findings
    Engineering->>Architect: Submit fixes for re-review
  end

  loop Until both validation and technical quality converge
    Testing->>Engineering: Re-run verification and report remaining issues
    Architect->>Engineering: Re-review code and report remaining issues
    Engineering->>Testing: Deliver updated fixes
    Engineering->>Architect: Deliver updated fixes
  end

  Testing->>Product: Provide true-device checklist and candidate validation context
  Engineering->>Product: Deliver rebuilt app or release candidate
  Product->>Product: Run final true-device validation for important scenarios and UX
  Product->>Engineering: Report product-acceptance findings when issues remain
  Testing->>Architect: Confirm test status and remaining risks
  Architect->>Product: Confirm technical convergence and release readiness
  Product->>Product: Approve and publish the version
```

## 4. Standard flow

### Step 1 — Product plans the version

The Product role defines the version goal, priority, and scope in `PLAN.MD`, then hands the
version task to the Architect.

### Step 2 — Product and Architect align on the solution direction

The Architect reviews the requested scope with Product, identifies technical constraints, and
prepares the technical design.

### Step 3 — Testing prepares validation coverage

Testing uses the version plan and the technical design to prepare:

- test cases
- regression coverage
- true-device validation cases
- release-relevant verification cases when applicable

Product also provides important-scenario UX expectations and key-scenario HTML design direction
when the change needs explicit HTML prototype or brand guidance.

### Step 4 — Engineering implements the work

Engineering starts implementation according to the approved technical design and keeps the work
within the confirmed version scope.

### Step 5 — Architect reviews the implementation direction

As implementation becomes available, the Architect reviews code quality and gives improvement
feedback on:

- architecture layering
- module structure
- design patterns
- extensibility
- performance
- compatibility

### Step 6 — Testing validates the delivered behavior

Once the implementation is functionally ready, Testing executes the planned validation and reports
issues with enough detail for reproduction.

### Step 7 — Engineering fixes issues and resubmits

Engineering addresses both:

- defects reported by Testing
- technical review findings from the Architect

Then Engineering resubmits the updated result.

### Step 8 — Testing and Architect converge in parallel

Testing re-runs verification while the Architect re-reviews the updated code. This loop continues
until both of these are true:

- the tested behavior is acceptable
- the technical structure is acceptable

### Step 9 — True-device validation

When required by the change, Product operates the final true-device validation on the packaged or
rebuilt application. Testing prepares the cases and evidence checkpoints, and Engineering provides
the rebuilt app or release candidate. Any findings return to Engineering for fixes, then back
through Testing and Architect review again.

### Step 10 — Architect prepares final technical acceptance

After Testing and review findings have converged, the Architect confirms the code is ready for
submission quality and release-candidate preparation.

### Step 11 — Product publishes the version

After technical and validation acceptance is complete, the Product role makes the final release
decision and publishes the new software version.

## 5. Iteration rule

The flow is intentionally iterative. After implementation starts, these three roles may cycle
multiple times:

- Engineering
- Testing
- Architect

The cycle ends only when both validation quality and technical quality have converged.

## 6. Handoff summary

### Product → Architect

Handoff items:

- target version
- confirmed scope
- priority and release intent

### Architect → Testing

Handoff items:

- technical design
- expected behavior boundaries
- identified risks and special cases

### Architect → Engineering

Handoff items:

- technical design
- task breakdown
- implementation constraints

### Testing → Engineering

Handoff items:

- validation failures
- reproduction details
- regression findings

### Architect → Engineering

Follow-up items:

- code review findings
- architecture improvements
- extensibility or performance concerns

### Architect → Product

Final status:

- technical convergence result
- remaining risk summary, if any
- submission readiness confirmation

## 7. Exit criteria by stage

### Planning exits when

- version scope is clear
- technical design direction is accepted
- testing coverage direction is defined

### Implementation exits when

- Engineering has completed the intended scope
- Architect review findings are addressed or explicitly accepted
- Testing findings are addressed or explicitly accepted

### Release preparation exits when

- true-device validation is complete where required
- code quality is accepted by the Architect
- Product decides to publish

## 8. Workflow ownership by role

The repository contains several workflow rules that belong to different roles. Use this ownership
split when deciding who should drive each part of the process.

### Product owns

- version planning
- cross-role session orchestration
- feature placement into `PLAN.MD`
- scope confirmation
- important-scenario user experience and HTML design direction
- final true-device product acceptance
- final release publication decision

### Architect owns

- technical design
- implementation task decomposition
- architecture and maintainability review
- final technical convergence before submission quality

### Engineering owns

- feature implementation
- incremental code changes
- local validation of affected code paths
- commit preparation for implementation work

### Testing owns

- test-case preparation
- regression planning
- true-device validation design and support
- release-candidate validation evidence

## 9. Special repository flows

### Feature implementation flow

- Product confirms version scope
- Architect defines technical design
- Testing defines test coverage
- Engineering implements
- Testing validates
- Architect reviews
- Engineering fixes
- Testing and Architect re-check until convergence

### Git submission flow

- Engineering prepares submission-ready change sets
- Architect verifies the code has reached submission quality
- Product dispatches the Git-submission step to Architect
- Architect performs the final Git commit with accurate grouping and commit messaging

### Release flow

- Engineering prepares the release candidate build
- Testing validates the exact candidate artifact and prepares true-device validation support
- Architect confirms technical convergence and release readiness
- Product runs final true-device acceptance
- Product makes the final publish decision
