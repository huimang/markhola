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
- use the repository IM log for explicit cross-session messaging when session-to-session handoff
  text must be persisted

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

## 2. Cross-session IM mechanism

Use the `im/` directory as the shared cross-session message area.

Rules:

- create one Markdown file per day
- the filename must use the date in `YYYYMMDD.md` format
- for 2026-07-29, the file is `im/20260729.md`
- append messages in chronological order
- prepend every new message with a unique UUID message ID
- each role session should read newly appended messages that mention its role
- treat the message ID as the canonical deduplication key for deciding whether a message has
  already been handled
- after completing a requested action, the receiving role should append a completion reply

Message format:

```text
550e8400-e29b-41d4-a716-446655440000[2026/07/29 20:34:24] product: @architect 请把当前git内的文档提交按git提交流程提交commit
550e8400-e29b-41d4-a716-446655440001[2026/07/29 20:36:34] architect: @product 已经操作完毕
```

Required fields:

- UUID message ID immediately before the timestamp
- timestamp in `[YYYY/MM/DD HH:MM:SS]`
- sender role name
- optional `@role` mention for the intended recipient
- the message body

Compatibility rule:

- historical messages without a UUID message ID may remain unchanged
- every newly appended message must include the message ID

Recommended usage:

- Product uses the IM file to dispatch role work explicitly
- Architect, Engineering, and Testing monitor new `@role` messages addressed to them
- when a message is intended for every role session, use `@all` so every role treats it as relevant
- completion, blockers, and handoff results are appended as new messages rather than editing old
  ones

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
- the reduced-frequency watcher rule does not remove the obligation to follow up on a still-blocked
  confirmation

### Next-step continuation rule

When a role has already received enough direction to identify the next concrete step, it should
continue that step immediately instead of stopping after only acknowledging receipt.

Rules:

- once the next owner, next action, and blocking condition are all clear, the receiving role should
  execute the next step directly
- do not stop at an IM reply such as "received", "noted", or "acknowledged" when the next action is
  already unambiguous
- append an IM reply only when it materially moves coordination forward, such as reporting review
  findings, requesting a missing confirmation, handing off evidence, or confirming completion
- if the next step is still unclear, say exactly what is missing and follow the confirmation
  follow-up rule rather than remaining idle
- this rule does not allow a role to bypass scope boundaries, role boundaries, or the standard
  planning-before-implementation workflow

### Message send-check rule

Before and after appending an IM message, re-check whether any newer IM message has changed what
should happen next.

Rules:

- before appending a new IM message, first read newly appended IM content so the outgoing message
  reflects the latest coordination state
- after appending a new IM message, immediately read again and decide whether any newly appended
  reply, confirmation, blocker, or handoff means the role should continue processing right away
- do not assume "message sent" means the role can wait for the next scheduled poll; if the
  send-time or post-send re-check reveals a clear next step, continue immediately
- apply this rule even when a watcher is currently in reduced-frequency mode

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

### Product heartbeat

The Product session should use a heartbeat check for new `@product` messages in the current day's
IM file.

Recommended implementation shape:

- default interval: 10 seconds
- watched file: `im/YYYYMMDD.md`
- optional local state storage for deduplication and last-seen position

The heartbeat reads only newly appended lines and prints any new `@product` messages that appear.

### Scheduled backoff rule

When a scheduled IM watcher receives no new messages for 5 consecutive minutes, reduce the polling
interval instead of polling indefinitely at the original high frequency.

Rules:

- treat 5 minutes without any newly appended IM message as an idle timeout
- when the idle timeout is reached, change the watcher interval from every 10 seconds to every 5
  minutes
- whenever a watcher reads any newly appended IM message, immediately restore the watcher interval
  to every 10 seconds and restart the 5-minute idle timer from that message-receipt time
- do not decide whether to stay in 5-minute mode by comparing the current time against an older
  message timestamp after a new message has already been received; the idle countdown must restart
  from the most recent received message
- keep the reduced-frequency watcher active until a later rule explicitly changes or removes it
- when announcing a rule that applies to every role session, send it with `@all`

## 3. Product session

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

## 4. Architect session

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

## 5. Engineering session

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

## 6. Testing session

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

## 7. Session routing quick guide

Use this routing rule when deciding where work belongs:

| If the task is mainly about... | Use this session |
| --- | --- |
| version scope, priority, HTML design direction, final acceptance, publish decision | Product |
| technical design, architecture, interfaces, terminology, commit-shape review | Architect |
| code changes, fixes, builds, implementation updates | Engineering |
| cases, validation, regression, candidate evidence, verification support | Testing |

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
| Testing and Architect have both converged, and a real-device pass is still required | Product | Final product acceptance belongs to Product. |
| Accepted work is ready to be committed into Git | Architect | Final Git submission ownership belongs to Architect. |
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
