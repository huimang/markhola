# Robot Guidelines

## 1. Respect `PLAN.MD`

Do not directly change the plan defined in `PLAN.MD` unless the user explicitly asks for that change.

You may propose suggestions, refinements, and implementation ideas, but they must stay within the direction already defined by `PLAN.MD`.

If something seems useful but has not been explicitly confirmed by the user, do not implement it on your own.

Instead:

- raise it as a suggestion
- explain why it may help
- wait for explicit confirmation before doing it

## 2. Build Only What Is Confirmed

Only implement work that has been clearly requested, confirmed, or can be directly inferred as necessary to complete the confirmed task.

Do not expand scope just because an idea seems better, more complete, or more elegant.

## 3. Suggest Before Expanding Scope

When you identify a potentially valuable improvement that is outside the currently confirmed scope:

1. describe the suggestion briefly
2. explain the benefit
3. wait for the user to confirm before implementing it

## 4. Prefer Incremental Delivery

When implementing new work, prefer the smallest complete change that satisfies the confirmed goal.

Avoid introducing architecture, UI, or product expansion that belongs to a later version unless the user explicitly approves it.

## 5. Keep Technical Plans Aligned With Product Scope

Technical design documents should reflect the current confirmed product scope, not an ideal future product.

Future ideas can be listed as optional follow-up items, but they must be clearly separated from the confirmed implementation scope.

## 6. Use Incremental Git Commits

When code changes form a small but complete and independently understandable unit, commit them
promptly instead of waiting to batch many unrelated updates at the end of the task.

For user-visible work, each independently understandable feature should normally have its own commit.

Do not combine multiple user-visible features into one implementation commit unless the user explicitly agrees to that batching.

Product, Engineering, Testing, and Architect sessions commit only files within their owned write
scope. Product commits confirmed `PLAN.MD` and product-planning packages; pure coordination without
tracked changes needs no commit. The Architect still reviews final history quality, integration
order, and release-commit readiness without replacing another role's independent commit.

Commit messages must use this format:

```text
[update|remove|add|bugfix] <session-name>: <English summary>
```

Choose exactly one operation, use the actual responsibility-oriented session name, and summarize
the complete change clearly and directly. Examples:

```text
[bugfix] engineering-release-candidate: Validate helper deployment targets
[add] testing-unit-cases: Cover Universal helper deployment target failures
[update] architect: Align role commit ownership rules
[update] product: Reorder post-release milestones
```

## 7. Respect `.gitignore`

Do not commit files or directories that are already ignored by `.gitignore`.

Before creating a commit, confirm that ignored build outputs, packaging artifacts, caches, local notes, and other excluded paths are not staged.

If a path is meant to stay out of version control, keep it untracked unless the user explicitly asks to change the ignore rule first.

### Staged artifact size gate

Install the tracked repository hook with `scripts/install_git_hooks.sh`. The hook runs
`scripts/check_git_staged_files.sh` against staged Git blobs before every commit.

The gate must:

- reject DMG, PDF, PKG, ISO, sparse image, App bundle, and common archive artifacts
- reject staged PNG, JPG, JPEG, GIF, WebP, or SVG files larger than `2 MiB`
- reject any other staged blob larger than `5 MiB`
- inspect staged index content so partially staged files are checked correctly
- report every violation in one run

Do not bypass this gate with `--no-verify` or an environment-variable override. If a larger or
currently forbidden artifact is genuinely required, change the tracked policy explicitly and have
the exception reviewed before committing it.

## 8. Follow the Standard Delivery Flow

When implementing a new feature or a user-requested change in this repository, follow this workflow:

1. Confirm the current development version before making changes.
2. Judge the scope of the requested change before assigning the next version:
   - if the work is a bug fix or an improvement to an existing feature, increase the last version number by `+1`
   - if the work is a new feature that did not exist before, increase the middle version number by `+1`
3. If the user directly requests a feature, record or update that feature in `PLAN.MD` under the target version before implementation.
4. Write the technical design document and the matching test document for that feature together before coding.
5. If the user adjusts either the design document or the test document after they were written, re-read the latest accepted versions before proceeding, and treat that re-read as a required step in the flow.
6. After the user accepts the design and test direction, refine both documents together so the implementation scope and validation scope stay aligned.
7. For user-visible features, also add or update a matching example Markdown file under `examples/` so users can verify the new capability after installing the new version.
8. Start development only after the user confirms the design document, test document, and example direction is OK.
9. Implement the feature in the smallest complete scope that satisfies the confirmed design.
10. Run the existing validation command for the affected code path.
11. Update `README.md` so the documented version and user-visible features match the target release.
12. Generate the DMG package after the implementation is complete, and use the `UDZO` compressed image format for release packaging unless the user explicitly asks for a different format.
13. (macOS dev workflow) When iterating locally on macOS, keep `dist/MarkHola.app` up to date so manual QA always runs the latest code:
    - one-off build: `scripts/build_app.sh`
    - auto-build on file changes: `scripts/dev_watch_app.sh`
14. When a fix is user-visible and needs manual verification (e.g. drag/drop, menus, webview navigation/rendering), include rebuilding `dist/MarkHola.app` via `scripts/build_app.sh` as a required step before declaring the work done.
15. Commit each small complete work package promptly using
    `[update|remove|add|bugfix] <session-name>: <English summary>`; when the user explicitly asks to
    commit a version, verify that each user-visible feature remains independently understandable.
16. After the feature commits, place documentation, packaging, validation, or release-summary adjustments in later commits instead of folding them back into the feature commits unless they are inseparable from a specific feature.
17. Before uploading or publishing a GitHub release, run sandbox validation against the exact DMG candidate that will be released. The sandbox flow must at minimum verify opening a Markdown file, editing it, saving it, and checking the target release feature in the packaged app.
18. Only publish the GitHub release after the sandbox validation passes. If validation fails, do not upload or publish the DMG.
19. Use the final release commit for that version as the tag target, and create a Git tag named `v<version>`.
20. During release validation, do not assume the active `MarkHola` window belongs to the latest build when multiple local app copies exist. First verify the actual process path or startup log of the running app.
21. If `/Applications/MarkHola.app` or another local copy may conflict with the candidate artifact, isolate or stop the conflicting app before validating the DMG candidate.
22. When the UI result conflicts with runtime logs, treat that as a release-validation blocker and resolve the target-artifact mismatch before continuing.
23. Keep a short validation record for each release under `drafts/` that includes the candidate DMG path, copied validation app path, tested process path, and the key runtime evidence used to prove the correct artifact was validated. Release validation records are local drafts and must not be committed.

For version tracking:

- always keep the current development version aligned across implementation-related files
- do not add feature work without making sure the target version is clear in the plan
- keep `PLAN.MD`, `README.md`, technical design notes, tests, example files, implementation status, and release tag consistent with the same target version

## 9. Rust Refactor Workflow

When refactoring large Rust files, follow this workflow and constraints:

1. Start with a Blueprint before coding:
   - list the current pain points
   - propose the target module/file structure
   - estimate the resulting file sizes
   - stop for confirmation before extraction work starts
2. After confirmation, extract incrementally:
   - move one logical submodule at a time
   - keep `use` imports explicit in every file
   - prefer `pub(crate)` or narrower visibility over broad `pub`
3. Finish with Integration:
   - reduce the host file to `mod` declarations, `pub use`, and minimal orchestration
   - avoid leaving extracted logic behind in the host file

Rust-specific constraints:

- keep any single Rust file, including tests, around 200 lines when practical
- prefer zero-cost abstractions such as generics, traits, and small inlineable helpers
- handle ownership explicitly when moving code; document or encode when values are `Copy`, `Clone`, borrowed, or moved

When choosing patterns for large Rust files:

- use composition when a struct owns too many fields or responsibilities
- use state pattern or typestate when branches mostly encode state transitions
- use strategy pattern when multiple business branches are parallel algorithms
- use builders or declarative macros when initialization or template code is repetitive

## 10. Verify Visual Layout With Real Rendering

DOM assertions, CSS string checks, and automated tests can prove that structure exists, but they cannot prove that the rendered geometry matches the intended layout.

When a user-visible change involves CSS Grid, Flexbox, hidden elements, positioning, scroll regions, or a combination of native controls and WebView content:

1. Run the automated tests and rebuild `dist/MarkHola.app`.
2. Launch that exact build instead of another installed copy.
3. When validation requires a specific Markdown file, open it directly with `scripts/qa_open_app.sh examples/<file>.md`; do not spend validation time navigating the native file picker.
4. Confirm the running process path belongs to that exact app bundle before trusting the UI result.
5. Use the accessibility tree to verify control presence, visibility, and text.
6. Use a real screenshot to verify position, dimensions, centering, stacking, and duplicated information.
7. Cover at least the representative states that change parent-child layout, such as no document and an opened document.
8. If the screenshot differs from the code-level expectation, treat the rendered result as authoritative and inspect parent sizing, Grid auto-placement, and CSS overrides.

Keep visual verification bounded to the smallest set of states directly related to the change. For a local CSS or behavior change, validate only the affected component, its relevant interaction or theme state, and any immediately coupled layout. Do not repeat unrelated file, feature, theme, or release checks unless the change can reasonably affect them. Full manual regression remains a release-validation activity, not the default for every local change.

## 10. Keep Prototypes Proportional

Before starting work, distinguish between:

- a technical evaluation or visual prototype
- an implementation intended for the product
- a production or release deliverable

For a technical evaluation or visual prototype:

- build the smallest artifact that makes the key idea directly inspectable
- prefer a local, dependency-free implementation when it is sufficient
- stop once the user can evaluate the requested behavior or appearance
- do not add frameworks, hosting, deployment, packaging, release work, or production hardening unless they are required for the evaluation or explicitly requested
- do not let incidental tooling concerns expand an otherwise simple prototype; simplify the approach first when possible

Before adding another step, ask whether it materially improves the user's ability to make the current decision. If it does not, leave it as an optional follow-up instead of implementing it.

## 11. Separate Repository Documentation From Development Drafts

Keep permanent repository documentation under `docs/` and commit it. Write new tracked repository
documentation in English by default, including long-term design principles, development guidance,
and code comments.

Use Chinese for Help documents that are specifically intended for Chinese-speaking end users. A
localized Chinese Help document may coexist with an English source or another localized Help version.

Keep version-scoped technical designs, test plans, release validation records, exploratory notes,
and other working documents under `drafts/`. The entire `drafts/` directory must remain ignored by
Git and must not be staged or committed.

`PLAN.MD` is the exception: keep the canonical product and version plan at the repository root,
track it in Git, and commit it with relevant planning updates. Never move `PLAN.MD` into `drafts/`.

Drafts may use the language that is most efficient for the current collaboration because they are
not repository documentation. Do not move a draft into `docs/` merely to preserve it; promote only
content that has become a permanent English repository reference or a localized Help document.

Do not rewrite unrelated historical documents only to change their language unless the user
explicitly requests it.

## 12. Follow the Visual Design Guidelines

Before changing theme colors, syntax highlighting, code blocks, Markdown reading surfaces, or other
color or visual-experience styles, read and follow `docs/visual-design-guidelines.md`.

For affected foreground/background pairs:

- calculate contrast instead of judging color values by appearance alone
- keep 14 px code text around `6:1–8:1`
- keep line numbers and secondary text around `4.5:1–5.5:1`
- keep required non-text separators and state indicators at least `3:1`
- keep language badges and other small text at least `4.5:1`
- validate both Default and Dark with real rendering

Meeting a minimum contrast threshold does not make higher contrast automatically better. Avoid
unnecessary luminance jumps, near-white text on large dark surfaces, pure-black code backgrounds,
and excessive saturation or simultaneous color/weight emphasis. If visual comfort conflicts with
minimum accessibility, preserve the accessibility minimum and consider a separate increased-contrast
variant rather than weakening the default below the minimum.

## 13. Separate Planning Tasks From Implementation Tasks

Keep `AGENTS.md` short and use it as an index for workflow rules.

For detailed task-boundary, task-switching, and repository-state guardrail rules, follow
`docs/development-workflow.md`.

For role responsibilities and cross-role execution order, follow:

- `docs/roles/product.md`
- `docs/roles/architect.md`
- `docs/roles/engineering.md`
- `docs/roles/testing.md`
- `docs/role-collaboration-flow.md`

Minimum requirements here:

- determine whether the current task is planning-only, implementation, or release/publish before
  acting
- if the task is planning-only, do not change implementation state
- do not let a brief phrase such as “start”, “continue”, or “begin implementation” silently
  override an already established planning-only task boundary
- in implementation tasks, follow the standard delivery flow in this document together with
  `docs/development-workflow.md`
