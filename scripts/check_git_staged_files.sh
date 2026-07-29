#!/bin/zsh

set -eu
setopt pipe_fail

readonly IMAGE_LIMIT=$((2 * 1024 * 1024))
readonly DEFAULT_LIMIT=$((5 * 1024 * 1024))

if ! repo_root=$(git rev-parse --show-toplevel 2>/dev/null); then
  print -u2 "error: staged-file checks must run inside a Git repository."
  exit 2
fi

cd "$repo_root"

typeset -a violations
violations=()

while IFS= read -r -d $'\0' staged_path; do
  lower_path="${(L)staged_path}"

  if [[ "$lower_path" =~ '(^|/)[^/]+\.app(/|$)' ]]; then
    violations+=("$(printf '%q' "$staged_path"): application bundles (.app) must not be committed")
    continue
  fi

  case "$lower_path" in
    *.dmg|*.pdf|*.pkg|*.iso|*.sparseimage|*.zip|*.tar|*.tgz|*.gz|*.7z|*.rar)
      violations+=("$(printf '%q' "$staged_path"): this artifact type must not be committed")
      continue
      ;;
  esac

  object_type=$(git cat-file -t ":$staged_path" 2>/dev/null || true)
  [[ "$object_type" == "blob" ]] || continue

  staged_size=$(git cat-file -s ":$staged_path")

  case "$lower_path" in
    *.png|*.jpg|*.jpeg|*.gif|*.webp|*.svg)
      if (( staged_size > IMAGE_LIMIT )); then
        violations+=("$(printf '%q' "$staged_path"): staged blob is ${staged_size} bytes; image limit is ${IMAGE_LIMIT} bytes")
      fi
      ;;
    *)
      if (( staged_size > DEFAULT_LIMIT )); then
        violations+=("$(printf '%q' "$staged_path"): staged blob is ${staged_size} bytes; general limit is ${DEFAULT_LIMIT} bytes")
      fi
      ;;
  esac
done < <(git diff --cached --name-only --diff-filter=ACMR -z)

if (( ${#violations[@]} > 0 )); then
  print -u2 "Commit blocked: ${#violations[@]} staged path(s) violate repository artifact rules:"
  for violation in "${violations[@]}"; do
    print -u2 -- "  - $violation"
  done
  print -u2
  print -u2 "Rules:"
  print -u2 "  - .app paths and .dmg/.pdf/.pkg/.iso/.sparseimage/.zip/.tar/.tgz/.gz/.7z/.rar files are forbidden."
  print -u2 "  - .png/.jpg/.jpeg/.gif/.webp/.svg blobs may be at most ${IMAGE_LIMIT} bytes."
  print -u2 "  - All other blobs may be at most ${DEFAULT_LIMIT} bytes."
  print -u2 "  - Exceptions require a reviewed rule change; there is no bypass."
  exit 1
fi

exit 0
