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

## 0. Architect core-work triage

Before assigning or performing work, Architect classifies the package as core architecture/engine
work or supporting implementation/testing work. If the plan and acceptance are clear and the package
is not core architecture or engine work, Architect dispatches it directly to the appropriate
Engineering or Testing owner instead of implementing or testing it personally.

For this triage, core work includes architectural boundaries and module structure, OS support and
platform compatibility, technology-stack selection or migration, core framework integration, and
foundational design-pattern or lifecycle decisions. Architect leads these decisions and reviews the
resulting implementation even when another role owns the code changes.

The handoff must identify one owner, a disjoint write scope, dependencies, acceptance evidence, and
the next integration point. Architect retains responsibility for architecture boundaries, risk
review, acceptance clarity, and follow-up review. This rule does not change the planning-only
boundary: implementation starts only after explicit authorization.

For parallel implementation and test work, use one temporary role-scoped session or subagent per
feature package. Give it only the package contract, write scope, dependencies, and acceptance it
needs; do not copy unrelated project history or open-ended context into the task. After the package
has delivered its commit or evidence and the downstream owner has consumed the handoff, close and
release that temporary session or subagent. Long-lived role sessions retain only coordination,
review, and integration context.

Before closing any role session, perform a short knowledge closeout. Record only correct, verified,
and materially reusable lessons that improve future work for the same role, such as a durable
architecture constraint, a recurring validation trap, or a repository workflow rule. Do not preserve
temporary debugging narration, duplicated status, speculative conclusions, or package-specific noise.
Place accepted lessons in the smallest appropriate durable role or workflow document, or keep them in
the concise handoff. Unresolved or low-confidence observations must not be promoted to shared
knowledge.

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

While `PLAN.MD` scope is still being discussed, revised, or awaiting user confirmation, no role may
stage, commit, or push changes for that scope. Product, Architect, Engineering, and Testing must keep
their planning and review changes unstaged and uncommitted until the user confirms the scope and
Product explicitly declares it frozen. After that freeze, each role resumes the owned incremental
commit rules below.

Engineering, Testing, and Architect should commit an owned work package as soon as it is a small,
complete, validated, and independently understandable unit. Do not wait until the end of a long task
to batch unrelated packages. Product may update `PLAN.MD` for review, but must leave that planning
diff unstaged and uncommitted until the user explicitly asks for the change to be committed. Once
the user gives that explicit instruction, Product applies the same incremental rule to the confirmed
`PLAN.MD` or product-planning package. Pure coordination without tracked changes needs no commit.

After the team explicitly decides to start development for a version, and the paired design, test
plan, and example direction are confirmed, it may create one independent scope-freeze commit before
implementation begins. This commit may contain only confirmed tracked planning or version metadata
within the committing role's write scope; ignored
drafts, implementation code, tests, fixtures, build outputs, and unreconciled scope changes stay out.
The freeze commit records that implementation may start and remains separate from later feature,
test, documentation, packaging, and release commits. If Product governance still requires the
`PLAN.MD` diff to remain uncommitted, record the freeze in the handoff and wait for the user's explicit
commit instruction rather than bypassing that rule.

Use:

```text
[update|remove|add|bugfix] <session-name>: <English summary>
```

The session name must be the actual responsibility-oriented task name. Each role commits only its
owned write scope. Product does not submit design, implementation, test, or workflow documentation.
Architect reviews the resulting history, integration order, and read-only release-commit readiness
without replacing another role's independent commit. Architect does not create or modify release
tags, GitHub drafts/releases, release assets, public release state, or downloaded release readback;
Product is the single owner of those release operations.

### Structural maintainability review

Before an implementation package is accepted, the Architect must review repository structure in
addition to runtime behavior. File names and sibling clusters are signals of responsibility, not
proof of a correct boundary. The review should:

- inventory related files by responsibility, directory, naming prefix, platform boundary, and test
  ownership
- identify clusters whose files implement the same capability but are scattered or difficult to
  discover, such as several `menu*` files under an application directory
- compare cohesion, dependency direction, public interfaces, conditional compilation, test location,
  ownership, and expected future growth before recommending consolidation
- prefer a focused capability directory when it materially improves discoverability and preserves
  clear boundaries; do not move files solely to make names look uniform
- require a small refactor blueprint, impact list, and validation plan before an approved move, and
  keep the move separate from unrelated feature behavior whenever practical
- treat circular dependencies, broader visibility, duplicate abstractions, broken resource paths,
  or harder test ownership as reasons to reject or defer the consolidation

The final review records either the accepted structure, the specific consolidation package, or the
reason the current layout should remain. A directory move is not required when existing module
boundaries are cohesive and discoverable.

This review is required after each small, complete implementation package, not only during a
version-end cleanup. Limit each pass to the files and responsibility clusters affected by that
package. Record safe consolidation work as a separate follow-up package with its own owner, tests,
and integration boundary; do not accumulate unrelated structural debt for a later bulk refactor.

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
- perform technical release-readiness review and prepare an exact release manifest

Only Product may:

- create or push release tags
- create, edit, publish, unpublish, or delete GitHub drafts and releases
- upload release assets or perform downloaded-asset SHA/readback checks
- change public release state or publish the GitHub release after validation passes

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
