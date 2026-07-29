# Development Workflow

This document defines how MarkHola work should move between planning, implementation, and release
tasks.

Related role documents:

- `docs/roles/product.md`
- `docs/roles/architect.md`
- `docs/roles/engineering.md`
- `docs/roles/testing.md`
- `docs/role-sessions.md`
- `docs/role-collaboration-flow.md`

Repository workflow ownership is further decomposed in the role documents. When a rule concerns
version planning, technical design, implementation, validation, Git submission, or release
publication, apply the role-specific requirements from those documents rather than treating the
workflow as role-neutral.

When role-based collaboration is used, each primary role should run in its own dedicated session as
defined in `docs/role-collaboration-flow.md`.

Product continuously tracks the active phase and critical path. A completed package is not treated
as closed until its downstream owner has consumed the handoff and the relevant acceptance exit is
either passed or explicitly blocked.

## 1. Task types

Every task must be treated as one of these modes before work begins:

- planning-only
- implementation
- release/publish

The active task type controls what changes are allowed.

## 2. Planning-only tasks

Use planning-only tasks for roadmap work, version placement, scope discussion, sequencing,
tradeoffs, acceptance criteria, and implementation preparation.

### Allowed work

- discuss product scope and release placement
- compare implementation options
- identify risks, dependencies, and validation needs
- update `PLAN.MD` when the user explicitly asks for a plan change
- update repository workflow/process documentation when the user explicitly asks to formalize rules

### Not allowed

- changing implementation code
- changing development versions
- running implementation builds or release packaging
- performing release publication work
- creating implementation commits

### Important boundary rule

Once a task is established as planning-only, later shorthand such as “start”, “continue”, or
“begin implementation” does not automatically change the task into implementation. The task must
first be explicitly reclassified by the user, or implementation must move into a separate
implementation task.

## 3. Implementation tasks

Use implementation tasks when the user clearly asks to code, implement, fix, or deliver a
confirmed change.

### Required prerequisites

Before coding:

1. confirm the target version
2. confirm the feature belongs in the accepted `PLAN.MD` scope
3. prepare the required design and test documentation
4. confirm any required example update direction

### Allowed work

- modify implementation code
- update implementation-facing version markers
- run targeted validation
- rebuild application artifacts needed for local QA
- prepare commits that reflect the confirmed implementation scope

### Not allowed

- expanding scope beyond the confirmed request
- skipping the documented delivery flow
- pulling future-version work into the current version without user approval

### Incremental commit rule

Product, Engineering, Testing, and Architect should commit an owned work package as soon as it is a
small, complete, validated, and independently understandable unit. Do not wait until the end of a
long task to batch unrelated packages. Product applies this rule to confirmed `PLAN.MD` and product
planning changes; pure coordination without tracked changes needs no commit.

Use:

```text
[update|remove|add|bugfix] <session-name>: <English summary>
```

The session name must be the actual responsibility-oriented task name. Each role commits only its
owned write scope. Product does not submit design, implementation, test, or workflow documentation.
Architect reviews the resulting history, integration order, and final release-commit readiness
without replacing another role's independent commit.

### Repository storage guard

Every local checkout should install the tracked Git hook with:

```bash
scripts/install_git_hooks.sh
```

Before each commit, `scripts/check_git_staged_files.sh` inspects the staged Git blobs. It rejects
release artifacts and common archives, images larger than `2 MiB`, and other files larger than
`5 MiB`. The index is authoritative so partially staged content is checked rather than the current
working-tree file.

Do not bypass the hook. A legitimate exception requires an explicit tracked policy change and
Architect review. Ignored build output, release candidates, validation PDFs, screenshots, and local
evidence remain outside Git.

## 4. Release/publish tasks

Use release/publish tasks when the user asks to package, validate, tag, upload, or publish a
version.

### Required prerequisites

- implementation for the target version is complete
- release validation is run against the exact candidate artifact
- the version, documentation, and tag target are aligned

### Allowed work

- build release artifacts
- run release validation
- create tags
- push release-related changes
- publish the GitHub release after validation passes

## 5. Task switching rules

Task switching must be explicit enough to avoid accidental coding inside a planning conversation.

### Planning-only → implementation

Do not switch merely because the user says:

- start
- continue
- begin
- start implementation

Switch only when the user also makes it clear that the current task is no longer planning-only, or
asks for implementation in a separate task.

### Implementation → release/publish

Switch only after implementation is complete and the user asks for packaging, validation, tagging,
or publishing.

## 6. Required mode declaration

Before acting, state the current mode in a short user-facing line:

- `Current task: planning-only.`
- `Current task: implementation.`
- `Current task: release/publish.`

This declaration is not cosmetic. It is a guardrail that forces the task type to be checked before
work begins.

## 7. Repository-state guardrail

If the next step would change repository implementation state, and the current task is still
planning-only, stop and do not perform that step.

Examples of repository implementation state changes include:

- editing source code
- editing implementation-facing versions
- building app artifacts for implementation validation
- staging implementation files for commit
