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
- use direct session delivery for cross-role handoffs; historical IM logs are not an active
  transport

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

When multiple Engineering sessions run concurrently, prefer a responsibility-oriented suffix that
shows each session's work package, such as `engineering-release-candidate` or
`engineering-universal-audit`. If no stable, meaningful scope label is available, use an explicit
numeric fallback such as `engineering-1` and `engineering-2`; never leave concurrent Engineering
sessions with indistinguishable names.

## 2. Cross-session delivery mechanism

Use direct session messaging as the only active cross-session delivery path. Historical files under
the ignored `im/` directory are not an active message transport.

Rules:

- assign every cross-session message a unique UUID
- resolve the current target role session through Codex thread discovery
- send the UUID, timestamp, sender, target, and body directly to the target session
- treat the message ID as the canonical deduplication key for deciding whether a message has
  already been handled
- send `@all` directly to every mapped role session except the sender
- after completing a requested action, the receiving role should directly send a substantive
  completion, blocker, or handoff message when another role needs that result
- if direct delivery fails, refresh the target role session and retry with the same UUID
- if delivery still fails, report the delivery blocker in the sender's task instead of silently
  dropping the message

Direct delivery envelope:

```text
UUID: 550e8400-e29b-41d4-a716-446655440000
Timestamp: 2026/07/29 20:34:24
Sender: product
Target: architect
Body: 请把当前 Git 内的文档按提交流程提交。
```

Compatibility rule:

- historical IM files may remain unchanged
- do not append new coordination messages to IM

Recommended usage:

- Product and the other roles dispatch normal work with direct session messaging
- direct delivery success is the receipt signal; do not send "received" or similar replies
- decisions, blockers, completed results, and actionable handoffs may be sent directly as new UUID
  messages
- do not mirror direct messages into IM

### Scope interpretation rule

When a user or role gives short product feedback, interpret it at the smallest reasonable scope
unless the message explicitly broadens it.

Rules:

- first identify the exact target object, such as a page state, a specific prompt, a component, or
  a whole feature
- first identify the requested action, such as remove, restyle, narrow the trigger condition, or
  disable only in one scenario
- do not silently expand a local complaint into a whole-feature product decision
- before dispatching a scope-changing instruction, write one explicit scope sentence that states
  what is in scope and what is out of scope
- if a role had to choose an interpretation, prefer the smallest interpretation that still
  satisfies the request
- when the change would alter implementation behavior, Product should normally send the corrected
  scope to Architect first, not directly to Engineering, unless Architect has already aligned on
  the same boundaries and Product is only removing ambiguity

Recommended dispatch template:

- `Scope in:` the exact behavior or scenario that should change
- `Scope out:` nearby behaviors that should remain unchanged
- `Reason:` why the change is needed
- `Acceptance:` what outcome proves the interpretation is correct

### Confirmation follow-up rule

When a role is waiting for scope confirmation or clarification, it must not wait indefinitely
without follow-up.

Rules:

- if a pending confirmation blocks implementation or validation progress, the waiting role should
  send a follow-up reminder after a reasonable delay instead of remaining silent
- default reminder cadence: follow up once after 10 minutes without a confirmation reply, then keep
  following up at a low frequency while the task remains blocked
- a follow-up reminder should restate the blocked question, the blocked next step, and what kind of
  confirmation is needed
- waiting for confirmation does not count as permission to expand scope or start implementation
- a prior successful delivery does not remove the obligation to follow up on a still-blocked
  confirmation

### Next-step continuation rule

When a role has already received enough direction to identify the next concrete step, it should
continue that step immediately instead of stopping after only acknowledging receipt.

Rules:

- once the next owner, next action, and blocking condition are all clear, the receiving role should
  execute the next step directly
- do not stop at an IM reply such as "received", "noted", or "acknowledged" when the next action is
  already unambiguous
- send a reply only when it materially moves coordination forward, such as reporting review
  findings, requesting a missing confirmation, handing off evidence, or confirming completion
- if the next step is still unclear, say exactly what is missing and follow the confirmation
  follow-up rule rather than remaining idle
- this rule does not allow a role to bypass scope boundaries, role boundaries, or the standard
  planning-before-implementation workflow

### Message send-check rule

Before and after direct delivery, check whether any newly delivered session message has changed
what should happen next.

Rules:

- before sending, process all currently delivered messages relevant to the decision so the outgoing
  message reflects the latest coordination state
- after sending, continue immediately when the current state already makes the next action clear
- do not assume "message sent" means the role can wait for the next scheduled poll; if the
  send-time or post-send check reveals a clear next step, continue immediately
- when direct delivery fails, refresh the target thread and retry with the same UUID; report a
  blocker if the target still cannot be reached

### Consolidated blocker-report rule

When a role is performing a bounded release-preparation, submission-readiness, or similar
repository-wide check, it should prefer one consolidated blocker report over multiple fragmented
messages.

Rules:

- before reporting a newly found blocker, first finish the current bounded check scope whenever
  practical, so nearby blockers can be reported together
- if multiple blockers belong to the same immediate decision or handoff, send them in one message
  instead of one-by-one
- use a fragmented sequence only when an urgent blocker must be escalated immediately or when the
  later issue could not reasonably be discovered within the same check pass
- once a consolidated report is sent, continue following the message send-check rule and only send
  another blocker message when a truly new blocker is discovered after the prior check scope ends

### Direct delivery

Product, Architect, Engineering, and Testing send normal messages directly to one another. They do
not run IM polling heartbeats.

Recommended implementation shape:

- discover the current role sessions with Codex thread tools
- deliver with direct session messaging and no scheduled wait
- include UUID, sender, target, timestamp, and message body
- deduplicate retries with the same UUID
- treat successful direct delivery as receipt
- do not run an IM Proxy, per-role watcher, or scheduled IM poll
- if a role session is replaced, refresh it through thread discovery before the next send

## 3. Product session

### Recommended names

- `product`
- `product-<scope>`
- `product-v<version>`

### Primary goal

Own version scope, product priorities, HTML design direction for important scenarios, final product
and release acceptance decisions, cross-role session orchestration, and continuous critical-path
progress.

### Typical inputs

- user requests
- roadmap state
- existing `PLAN.MD`
- technical constraints from Architect
- risk and validation feedback from Testing
- implementation and delivery status from Engineering
- current owner, next action, dependencies, blockers, acceptance exit, and last substantive progress
  for every active package

### Expected outputs

- version placement decisions
- scope confirmation
- priority decisions
- role dispatch and handoff decisions
- maintained current-stage and critical-path state
- HTML design mockups or prototypes when needed
- product acceptance decisions
- release go/no-go decisions

### Allowed actions

- update `PLAN.MD` when planning changes are confirmed
- promptly commit each small complete Product-owned planning package using
  `[update|remove|add|bugfix] <product-session-name>: <English summary>`
- skip commits for pure coordination when no tracked product-planning file changed
- decide which role session should act next
- dispatch work to Architect, Engineering, and Testing sessions
- immediately drive a clear next owner and action without waiting for a user reminder
- follow up on blocking confirmations after 10 minutes by default
- investigate other work that exceeds a reasonable expected duration, then re-route, parallelize, or
  escalate when that materially unblocks the critical path
- ensure every completed result or handoff is consumed by its downstream owner
- dispatch final history review, integration, or release-commit work to the Architect session
- define user-experience expectations
- define important-scenario HTML design direction
- review Testing's true-device evidence and decide final product acceptance
- decide whether a release should publish

### Not allowed as the primary responsibility

- writing implementation code
- authoring detailed technical design
- replacing formal test coverage design
- performing the final technical code review
- committing design, implementation, test, or workflow files owned by another role
- implementing, testing, or technically reviewing work under the project-manager responsibility

Product reports substantive status, risks, decisions, and user-action blockers. Healthy routine
progress remains low-noise.

### Handoff targets

- hand scope and priorities to Architect
- hand UX and HTML design expectations to Architect and Engineering
- route implementation work to Engineering and verification work to Testing
- receive technical readiness from Architect
- receive validation status from Testing

## 4. Architect session

### Recommended names

- `architect`
- `architect-<scope>`
- `architect-v<version>`

### Primary goal

Translate product scope and business requirements into technical design, constrain architecture and
interfaces, guide implementation structure, and decide when technical quality has converged.
Architect performs targeted risk-based verification but does not duplicate Testing's full automated
or true-device validation matrix.

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
- commit small complete Architect-owned packages using the repository message format
- execute final integration and Git submission for accepted work
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

## 5. Engineering session

### Recommended names

- `engineering`
- `engineering-<scope>`
- `engineering-v<version>`
- `engineering-<number>` only when a meaningful scope suffix is not available

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
- incrementally committed, submission-ready change sets

### Allowed actions

- implement confirmed scope
- update implementation-facing files
- run affected validation commands
- rebuild verification artifacts when needed
- promptly create incremental, atomic commits using
  `[update|remove|add|bugfix] <engineering-session-name>: <English summary>`

### Not allowed as the primary responsibility

- redefining product scope
- replacing technical design ownership
- replacing Testing coverage ownership
- publishing a release on its own

### Handoff targets

- hand builds and fixes to Testing
- hand updated code and commit structure to Architect
- hand rebuilt and frozen candidates to Testing for exact-artifact and true-device validation

## 6. Testing session

### Recommended names

- `testing`
- `testing-<scope>`
- `testing-v<version>`

### Primary goal

Turn the version plan and technical design into explicit verification coverage and automated test
code, validate delivered behavior, execute true-device checks, and provide release-candidate
evidence for Product's final go/no-go decision.

### Typical inputs

- version plan
- accepted technical design
- implementation builds from Engineering
- behavior boundaries and risks from Architect

### Expected outputs

- test cases
- automated unit, integration, or script-level test code
- regression coverage
- validation findings
- release-candidate evidence
- true-device results, evidence, and remaining-risk summaries

### Allowed actions

- define test coverage
- implement test code derived from accepted test cases
- promptly commit complete test-code packages using
  `[update|remove|add|bugfix] <testing-session-name>: <English summary>`
- validate delivered behavior
- report failures and reproduction steps directly to the responsible Engineering session
- re-run verification after fixes
- validate exact candidate artifacts before release publication
- execute true-device validation and provide evidence to Product

### Not allowed as the primary responsibility

- redefining scope
- replacing Architect review
- implementing production-code fixes
- replacing Product's final product and release decision
- publishing a release on its own

### Handoff targets

- hand failures and regression findings to Engineering
- hand validation status and remaining risks to Architect
- hand true-device results, evidence, and remaining risks to Product

## 7. Session routing quick guide

Use this routing rule when deciding where work belongs:

| If the task is mainly about... | Use this session |
| --- | --- |
| version scope, priority, HTML design direction, final acceptance, publish decision | Product |
| technical design, architecture, interfaces, terminology, commit-shape review | Architect |
| code changes, fixes, builds, implementation updates | Engineering |
| test code, cases, validation, regression, candidate evidence, true-device execution | Testing |

## 8. Product dispatch decision table

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
| Testing and Architect have both converged, and a real-device pass is still required | Testing | Testing executes the real-device pass and produces evidence for Product. |
| A role-owned package is small, complete, validated, and ready to commit | The package owner | Each role promptly commits only its own write scope. |
| All accepted packages need final history or release-commit review | Architect | Architect owns final grouping, format, integration order, and release readiness. |
| Final true-device product acceptance passed, and the release candidate is ready | Product | Product owns the final publish decision. |

## 9. Product pause and resume rules

The Product session may pause a role session when:

- that role has completed its current handoff output
- another role has become the current bottleneck
- the work must wait for a higher-priority scope or release decision

The Product session should resume a role session when:

- a dependency blocking that role has been resolved
- a new handoff input has arrived for that role
- the flow has returned to that role after review, validation, or acceptance feedback
