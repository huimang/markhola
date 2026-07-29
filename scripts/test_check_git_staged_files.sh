#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/check_git_staged_files.sh"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/markhola-git-guard.XXXXXX")"
IMAGE_LIMIT=$((2 * 1024 * 1024))
OTHER_LIMIT=$((5 * 1024 * 1024))

trap 'rm -rf "$TEST_ROOT"' EXIT

if [[ ! -x "$CHECK_SCRIPT" ]]; then
  print -u2 -- "Missing executable staged-file guard: $CHECK_SCRIPT"
  exit 1
fi

make_file() {
  local file_path="$1"
  local size="$2"

  mkdir -p "${file_path:h}"
  dd if=/dev/zero of="$file_path" bs="$size" count=1 2>/dev/null
}

new_repo() {
  local name="$1"
  local repo="$TEST_ROOT/$name"

  mkdir -p "$repo"
  git -C "$repo" init -q
  git -C "$repo" config user.name "MarkHola Test"
  git -C "$repo" config user.email "test@markhola.local"
  print -r -- "$repo"
}

run_guard() {
  local repo="$1"
  local output_file="$2"

  (
    cd "$repo"
    "$CHECK_SCRIPT"
  ) >"$output_file" 2>&1
}

expect_success() {
  local name="$1"
  local repo="$2"
  local output="$TEST_ROOT/$name.output"

  if ! run_guard "$repo" "$output"; then
    print -u2 -- "Expected staged-file guard success: $name"
    cat "$output" >&2
    return 1
  fi

  print -r -- "PASS: $name"
}

expect_failure() {
  local name="$1"
  local repo="$2"
  local output="$TEST_ROOT/$name.output"
  shift 2

  if run_guard "$repo" "$output"; then
    print -u2 -- "Expected staged-file guard failure: $name"
    cat "$output" >&2
    return 1
  fi

  local expected_path
  for expected_path in "$@"; do
    if ! grep -Fq -- "$expected_path" "$output"; then
      print -u2 -- "Guard output omitted violation '$expected_path': $name"
      cat "$output" >&2
      return 1
    fi
  done

  print -r -- "PASS (expected failure): $name"
}

SMALL_REPO="$(new_repo ordinary-small-file)"
mkdir -p "$SMALL_REPO/docs"
print -r -- "small staged content" >"$SMALL_REPO/docs/ordinary file.txt"
git -C "$SMALL_REPO" add "docs/ordinary file.txt"
expect_success ordinary-small-file "$SMALL_REPO"

IMAGE_BOUNDARY_REPO="$(new_repo image-boundary-allowed)"
IMAGE_EXTENSIONS=(png jpg jpeg gif webp svg)
for extension in "${IMAGE_EXTENSIONS[@]}"; do
  make_file "$IMAGE_BOUNDARY_REPO/image-boundary.$extension" "$IMAGE_LIMIT"
done
git -C "$IMAGE_BOUNDARY_REPO" add .
expect_success image-boundary-allowed "$IMAGE_BOUNDARY_REPO"

OTHER_BOUNDARY_REPO="$(new_repo other-boundary-allowed)"
make_file "$OTHER_BOUNDARY_REPO/other-boundary.bin" "$OTHER_LIMIT"
git -C "$OTHER_BOUNDARY_REPO" add other-boundary.bin
expect_success other-boundary-allowed "$OTHER_BOUNDARY_REPO"

WORKTREE_LARGER_REPO="$(new_repo staged-small-worktree-large)"
print -r -- "small staged content" >"$WORKTREE_LARGER_REPO/staged-small.bin"
git -C "$WORKTREE_LARGER_REPO" add staged-small.bin
make_file "$WORKTREE_LARGER_REPO/staged-small.bin" "$((OTHER_LIMIT + 1))"
expect_success staged-small-worktree-large "$WORKTREE_LARGER_REPO"

WORKTREE_SMALLER_REPO="$(new_repo staged-large-worktree-small)"
make_file "$WORKTREE_SMALLER_REPO/staged-large.bin" "$((OTHER_LIMIT + 1))"
git -C "$WORKTREE_SMALLER_REPO" add staged-large.bin
print -r -- "small worktree content" >"$WORKTREE_SMALLER_REPO/staged-large.bin"
expect_failure staged-large-worktree-small "$WORKTREE_SMALLER_REPO" staged-large.bin

IMAGE_OVERSIZE_REPO="$(new_repo image-over-limit)"
IMAGE_OVERSIZE_PATHS=()
for extension in "${IMAGE_EXTENSIONS[@]}"; do
  artifact_path="oversized image.$extension"
  IMAGE_OVERSIZE_PATHS+=("$artifact_path")
  make_file "$IMAGE_OVERSIZE_REPO/$artifact_path" "$((IMAGE_LIMIT + 1))"
done
git -C "$IMAGE_OVERSIZE_REPO" add .
expect_failure image-over-limit "$IMAGE_OVERSIZE_REPO" "${IMAGE_OVERSIZE_PATHS[@]}"

OTHER_OVERSIZE_REPO="$(new_repo other-over-limit)"
make_file "$OTHER_OVERSIZE_REPO/oversized other.bin" "$((OTHER_LIMIT + 1))"
git -C "$OTHER_OVERSIZE_REPO" add "oversized other.bin"
expect_failure other-over-limit "$OTHER_OVERSIZE_REPO" "oversized other.bin"

BANNED_REPO="$(new_repo banned-artifacts-and-app)"
BANNED_EXTENSIONS=(dmg pdf pkg iso sparseimage zip tar tgz gz 7z rar)
BANNED_PATHS=()
for extension in "${BANNED_EXTENSIONS[@]}"; do
  artifact_path="release artifact.$extension"
  BANNED_PATHS+=("$artifact_path")
  print -r -- "artifact" >"$BANNED_REPO/$artifact_path"
done
APP_PATH="Build Output/MarkHola.app/Contents/MacOS/MarkHola"
BANNED_PATHS+=("$APP_PATH")
mkdir -p "$BANNED_REPO/${APP_PATH:h}"
print -r -- "bundle executable" >"$BANNED_REPO/$APP_PATH"
git -C "$BANNED_REPO" add .
expect_failure banned-artifacts-and-app "$BANNED_REPO" "${BANNED_PATHS[@]}"

UPPERCASE_REPO="$(new_repo uppercase-extensions)"
UPPERCASE_DMG_PATH="uppercase artifact.DMG"
UPPERCASE_PNG_PATH="uppercase oversized.PNG"
print -r -- "artifact" >"$UPPERCASE_REPO/$UPPERCASE_DMG_PATH"
make_file "$UPPERCASE_REPO/$UPPERCASE_PNG_PATH" "$((IMAGE_LIMIT + 1))"
git -C "$UPPERCASE_REPO" add .
expect_failure uppercase-extensions "$UPPERCASE_REPO" \
  "$UPPERCASE_DMG_PATH" "$UPPERCASE_PNG_PATH"

RENAME_REPO="$(new_repo renamed-target-paths)"
print -r -- "safe source" >"$RENAME_REPO/safe-artifact.txt"
print -r -- "safe source" >"$RENAME_REPO/safe-bundle.txt"
git -C "$RENAME_REPO" add .
git -C "$RENAME_REPO" commit -qm "Add safe source files"
RENAMED_DMG_PATH="renamed artifact.dmg"
RENAMED_APP_PATH="Renamed Output.app/Contents/data"
git -C "$RENAME_REPO" mv safe-artifact.txt "$RENAMED_DMG_PATH"
mkdir -p "$RENAME_REPO/${RENAMED_APP_PATH:h}"
git -C "$RENAME_REPO" mv safe-bundle.txt "$RENAMED_APP_PATH"
expect_failure renamed-target-paths "$RENAME_REPO" \
  "$RENAMED_DMG_PATH" "$RENAMED_APP_PATH"

DELETION_REPO="$(new_repo staged-deletions-allowed)"
make_file "$DELETION_REPO/legacy-large.bin" "$((OTHER_LIMIT + 1))"
print -r -- "legacy artifact" >"$DELETION_REPO/legacy.dmg"
mkdir -p "$DELETION_REPO/Legacy.app/Contents"
print -r -- "legacy bundle" >"$DELETION_REPO/Legacy.app/Contents/data"
git -C "$DELETION_REPO" add .
git -C "$DELETION_REPO" commit -qm "Add legacy files"
git -C "$DELETION_REPO" rm -qr legacy-large.bin legacy.dmg Legacy.app
expect_success staged-deletions-allowed "$DELETION_REPO"

print -r -- "All staged-file guard tests passed."
