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

When code changes are ready, prefer committing them incrementally by feature instead of batching many unrelated updates into one commit.

For user-visible work, each independently understandable feature should normally have its own commit.

Do not combine multiple user-visible features into one implementation commit unless the user explicitly agrees to that batching.

Commit messages should:

- be written in English
- be a single sentence
- summarize the change clearly and directly

## 7. Respect `.gitignore`

Do not commit files or directories that are already ignored by `.gitignore`.

Before creating a commit, confirm that ignored build outputs, packaging artifacts, caches, local notes, and other excluded paths are not staged.

If a path is meant to stay out of version control, keep it untracked unless the user explicitly asks to change the ignore rule first.

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
15. When the user explicitly asks to commit a version, split the implementation history by small feature first, and keep each user-visible feature in its own English one-sentence commit whenever practical.
16. After the feature commits, place documentation, packaging, validation, or release-summary adjustments in later commits instead of folding them back into the feature commits unless they are inseparable from a specific feature.
17. Before uploading or publishing a GitHub release, run sandbox validation against the exact DMG candidate that will be released. The sandbox flow must at minimum verify opening a Markdown file, editing it, saving it, and checking the target release feature in the packaged app.
18. Only publish the GitHub release after the sandbox validation passes. If validation fails, do not upload or publish the DMG.
19. Use the final release commit for that version as the tag target, and create a Git tag named `v<version>`.
20. During release validation, do not assume the active `MarkHola` window belongs to the latest build when multiple local app copies exist. First verify the actual process path or startup log of the running app.
21. If `/Applications/MarkHola.app` or another local copy may conflict with the candidate artifact, isolate or stop the conflicting app before validating the DMG candidate.
22. When the UI result conflicts with runtime logs, treat that as a release-validation blocker and resolve the target-artifact mismatch before continuing.
23. Keep a short validation record for each release that includes the candidate DMG path, copied validation app path, tested process path, and the key runtime evidence used to prove the correct artifact was validated.

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

## 11. Keep `docs/` Documents in Chinese

Write new documents under `docs/` in Chinese so the user can review technical and test directions quickly.

When updating an existing `docs/` document for active development, keep the updated content in Chinese. File names and technical identifiers may remain in English when that preserves stable references or matches the implementation.

Do not rewrite unrelated historical documents only to change their language unless the user explicitly requests it.
