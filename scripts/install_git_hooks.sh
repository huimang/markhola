#!/bin/zsh

set -eu

if ! repo_root=$(git rev-parse --show-toplevel 2>/dev/null); then
  print -u2 "error: Git hooks can only be installed from inside a Git repository."
  exit 1
fi

cd "$repo_root"
git config --local core.hooksPath .githooks

print "Configured repository-local Git hooks from .githooks."
