#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_PATH="$ROOT_DIR/dist/MarkHola.app"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <markdown-file>" >&2
  echo "Example: $0 examples/mermaid.md" >&2
  exit 2
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "Missing app bundle: $APP_PATH" >&2
  echo "Run scripts/build_app.sh first." >&2
  exit 1
fi

FILE_PATH="$1"
if [[ "$FILE_PATH" != /* ]]; then
  FILE_PATH="$ROOT_DIR/$FILE_PATH"
fi

if [[ ! -f "$FILE_PATH" ]]; then
  echo "Missing Markdown file: $FILE_PATH" >&2
  exit 1
fi

echo "==> Validation app: $APP_PATH"
echo "==> Opening file: $FILE_PATH"
open -n -a "$APP_PATH" "$FILE_PATH"
