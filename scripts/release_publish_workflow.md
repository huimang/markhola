# Release Publish Workflow

Use this workflow for every MarkHola release candidate that will be uploaded to GitHub.

The key rule is:

1. finish implementation and version updates
2. build the exact DMG candidate
3. validate that candidate inside a macOS sandbox
4. only publish the GitHub release after sandbox validation passes

## 1. Prepare the release candidate

Make sure the target version is already aligned in:

- `Cargo.toml`
- `PLAN.MD`
- `README.md`
- `assets/help/Documentation.md`
- `assets/help/Documentation.zh-CN.md` when the release supports Simplified Chinese
- any release notes or user-facing example files affected by the release

The bundled Help document is a required release artifact, not an optional documentation follow-up.
Before packaging:

- set `Current version` in every bundled Help language file to the exact target `v<version>`
- update every language file's feature and menu descriptions for each user-visible release change
- remove descriptions of menu items or behavior that no longer exist from every language file

Do not build or publish the release candidate while any bundled Help version or visible behavior
description is stale or while a supported language's Help file is missing.

Run the automated regression flow first:

```bash
./scripts/release_regression.sh --with-package
```

This should leave you with an architecture-specific pair:

```bash
dist/MarkHola-<version>-apple-silicon.dmg
dist/MarkHola-<version>-intel.dmg
```

For `v0.9.0`, verify both thin Apps independently:

```bash
./scripts/verify_macos_architectures.sh \
  --app dist/MarkHola-apple-silicon.app \
  --architecture arm64
./scripts/verify_macos_architectures.sh \
  --app dist/MarkHola-intel.app \
  --architecture x86_64
```

Each check must prove that the main executable and every bundled Mach-O contain only the named
architecture, every Mach-O uses the macOS 14.0 deployment target, `LSMinimumSystemVersion` is 14.0,
and the final assembled App signature remains valid. Both Apps must come from one commit and have
identical user-visible resources.

## 2. Run pre-publish sandbox validation

Before Product creates or publishes the GitHub release, validate both packaged Apps in their
accepted macOS environments.

Each validation target must be the exact architecture-specific DMG file that Product will upload,
not a separately rebuilt artifact.

Recommended sandbox validation flow:

1. Mount the architecture-specific `dist/MarkHola-<version>-<architecture>.dmg`
2. Copy `MarkHola.app` from the mounted volume into a sandbox-local path
3. Before launching, check whether other local `MarkHola.app` copies already exist, especially `/Applications/MarkHola.app`
4. Stop or isolate other running `MarkHola` processes so LaunchServices does not route validation to an older installed copy
5. Launch that copied app
6. Confirm the running process path matches the copied candidate app, not `/Applications/MarkHola.app` or another local bundle
7. Capture startup-log evidence that the expected version and release-specific initialization ran in that candidate app
8. Verify the app can open a Markdown file through `File > Open`
9. Switch to writable mode
10. Edit the document and add representative Markdown syntax
11. Save the file and confirm the file changed on disk
12. Switch back to readonly mode and verify rendered output
13. If the release includes `[toc]`, verify the generated table of contents updates after save

Capture the actual running architecture for each thin candidate:

- Apple Silicon native launch must report `aarch64`
- Intel validation must run on a physical Intel Mac or true x86_64 macOS 14+ virtual/fully emulated
  guest and report `x86_64`
- virtual/fully emulated Intel validation must prove `sysctl.proc_translated=0`
- Rosetta, arm64 Tart, and static architecture evidence are supplemental and cannot pass Intel G4

Hard rule:

- Do not trust UI validation alone when multiple MarkHola installs share the same bundle id.
- Always keep at least one process-path or startup-log proof that the tested app is the copied candidate artifact from the target DMG.

Minimum required manual coverage:

- open a Markdown file successfully
- edit and save successfully
- open `Help > Documentation` in each supported interface language and confirm it shows the target
  version and current release behavior
- verify the release's new feature works
- verify one or more existing core features still work

For `v0.7.5`-style releases, the sandbox verification must include:

- `[toc]` rendering
- multi-section heading navigation in the generated TOC
- normal Markdown editing and saving

If sandbox validation fails, do not upload or publish the DMG.

If the UI behavior and the logs disagree, assume the wrong app copy may have been activated first, then re-run validation against a confirmed candidate process path.

## 3. Product creates the GitHub release draft

Only Product performs these actions, and only after both architecture checks pass:

1. create the Git tag `v<version>` on the final release commit
2. draft the GitHub release
3. upload both already-validated architecture-specific DMG files
4. fill the release title and notes

The release notes should summarize the items listed under the matching version in `PLAN.MD`.

## 4. Publish the GitHub release

Publish the release only after confirming all of the following:

- both uploaded DMGs are the same validated artifacts
- the release title matches `MarkHola-<version>`
- the release notes match the target version scope
- every packaged `Help > Documentation` language matches the target version and user-visible release
  scope
- the Git tag points at the intended final release commit
- the GitHub Release contains exactly `MarkHola-<version>-apple-silicon.dmg` and
  `MarkHola-<version>-intel.dmg`
- each downloaded release asset has the same SHA-256 as its frozen validated candidate

## 5. Keep evidence

For each release, keep a short verification record with:

- both DMG paths
- both copied validation App paths
- the running process path used during validation
- the tested version
- the sandbox validation result
- the key behaviors verified
- the complete Mach-O list, thin architecture, and deployment target for each App
- the actual Apple Silicon and qualified Intel/x86_64 process architectures
- whether Developer ID signing, DMG signing, notarization, staple, and validate ran or were skipped
- the GitHub release URL after publish

Store this record at `drafts/release-validation-v<version>.md`. Release validation records are local
working evidence and must not be staged or committed.
