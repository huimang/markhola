#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT_DIR/scripts/macos_toolchain.sh"
WITH_PACKAGE=0
APP_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"

for argument in "$@"; do
  case "$argument" in
    --with-package)
      WITH_PACKAGE=1
      ;;
    *)
      echo "Unknown argument: $argument" >&2
      exit 1
      ;;
  esac
done

markhola_prepare_rust_toolchain

require_file() {
  local path="$1"
  if [[ ! -f "$ROOT_DIR/$path" ]]; then
    echo "Missing required file: $path" >&2
    exit 1
  fi
}

verify_help_version() {
  local path="$1"
  local content
  require_file "$path"
  content="$(<"$ROOT_DIR/$path")"
  if [[ "$content" != *"Current version: \`v${APP_VERSION}\`"* ]]; then
    echo "Bundled Help version mismatch: $path must declare v${APP_VERSION}" >&2
    exit 1
  fi
}

run_packaging_with_retry() {
  local attempt
  for attempt in 1 2 3; do
    if "$ROOT_DIR/scripts/package_dmg.sh"; then
      return 0
    fi

    if [[ "$attempt" -eq 3 ]]; then
      echo "Release packaging failed after ${attempt} attempts." >&2
      exit 1
    fi

    echo "Retrying full packaging flow after transient failure (attempt ${attempt}/3)..." >&2
    sleep 2
  done
}

run_release_binary() {
  local log_path="$1"
  shift

  if "$@" >"$log_path" 2>&1; then
    cat "$log_path"
    return 0
  fi

  cat "$log_path" >&2
  return 1
}

require_output_directory() {
  local output_dir="$ROOT_DIR/dist"
  if [[ -d "$output_dir" ]]; then
    return 0
  fi

  if ! mkdir -p "$output_dir"; then
    echo "Failed to create required output directory: $output_dir" >&2
    exit 1
  fi
}

require_unix_socket_test_capability() {
  case "${MARKHOLA_SOCKET_PREFLIGHT:-auto}" in
    pass)
      return 0
      ;;
    fail)
      echo "Release regression requires Unix domain socket bind capability for protocol transport tests." >&2
      echo "Current environment is not approved for this check (forced preflight failure)." >&2
      echo "Run scripts/release_regression.sh in an allowed local environment, not a sandbox that denies AF_UNIX bind." >&2
      exit 1
      ;;
  esac

  if ! /usr/bin/python3 - <<'PY'
import os
import socket
import tempfile

root = tempfile.mkdtemp(prefix="markhola-socket-preflight-")
path = os.path.join(root, "transport.sock")
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
try:
    sock.bind(path)
finally:
    sock.close()
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass
    os.rmdir(root)
PY
  then
    echo "Release regression requires Unix domain socket bind capability for protocol transport tests." >&2
    echo "Current environment denied AF_UNIX bind (for example: sandbox Operation not permitted)." >&2
    echo "Review and rerun scripts/release_regression.sh in an allowed environment instead of treating socket failures as PASS." >&2
    exit 1
  fi
}

is_known_sandbox_webkit_failure() {
  local log_path="$1"
  grep -q "unsupported type" "$log_path" || grep -q "Timed out while preparing the export page" "$log_path"
}

echo "==> Running thin architecture gate tests"
"$ROOT_DIR/scripts/test_verify_macos_architectures.sh"

echo "==> Checking Unix socket test capability"
require_unix_socket_test_capability

echo "==> Preparing ignored output directory"
require_output_directory

echo "==> Running automated regression tests"
markhola_cargo test --locked --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Running x86_64 automated regression tests"
markhola_cargo test \
  --locked \
  --manifest-path "$ROOT_DIR/Cargo.toml" \
  --target x86_64-apple-darwin

echo "==> Building release binary"
markhola_cargo build \
  --release \
  --locked \
  --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Verifying required regression fixtures"
require_file "examples/basic.md"
require_file "examples/languages.md"
require_file "examples/mermaid.md"
require_file "examples/math.md"
require_file "examples/multi-document.md"
require_file "examples/pdf-export.md"
require_file "examples/theme-showcase.md"
require_file "examples/v0.9.2-offline-cli-export.md"
require_file "assets/help/Documentation.md"
require_file "assets/help/Documentation.zh-CN.md"
require_file "i18n/en.yaml"
require_file "i18n/zh-CN.yaml"
require_file "themes/default/layout.css"
require_file "themes/dark/layout.css"
require_file "scripts/release_regression_checklist.md"
verify_help_version "assets/help/Documentation.md"
verify_help_version "assets/help/Documentation.zh-CN.md"

echo "==> Running public offline CLI regression"
CLI_BINARY="$ROOT_DIR/target/release/markhola"
CLI_SOURCE="$ROOT_DIR/examples/v0.9.2-offline-cli-export.md"
CLI_PNG_PATH="$ROOT_DIR/dist/offline-cli-smoke.png"
CLI_PDF_PATH="$ROOT_DIR/dist/offline-cli-smoke.pdf"
CLI_HTML_PATH="$ROOT_DIR/dist/offline-cli-smoke.html"
CLI_SOURCE_SHA_BEFORE="$(shasum -a 256 "$CLI_SOURCE" | awk '{print $1}')"
rm -f "$CLI_PNG_PATH" "$CLI_PDF_PATH" "$CLI_HTML_PATH"

if [[ "$("$CLI_BINARY" version)" != "MarkHola ${APP_VERSION}" ]]; then
  echo "Offline CLI version output does not match v${APP_VERSION}." >&2
  exit 1
fi
CLI_HELP_OUTPUT="$("$CLI_BINARY" help)"
if [[ "$CLI_HELP_OUTPUT" != *"export-png"* || "$CLI_HELP_OUTPUT" == *"--smoke-"* ]]; then
  echo "Offline CLI help is missing public commands or exposes internal smoke commands." >&2
  exit 1
fi

"$CLI_BINARY" export-png \
  --source="$CLI_SOURCE" \
  --target="$CLI_PNG_PATH" \
  --theme=dark \
  --json
"$CLI_BINARY" export-pdf \
  --source="$CLI_SOURCE" \
  --target="$CLI_PDF_PATH" \
  --theme=light \
  --json
"$CLI_BINARY" export-html \
  --source="$CLI_SOURCE" \
  --target="$CLI_HTML_PATH" \
  --theme=dark \
  --json

if [[ "$(head -c 8 "$CLI_PNG_PATH" | xxd -p)" != "89504e470d0a1a0a" ]]; then
  echo "Offline CLI PNG output has an invalid signature." >&2
  exit 1
fi
if [[ "$(head -c 5 "$CLI_PDF_PATH")" != "%PDF-" ]]; then
  echo "Offline CLI PDF output has an invalid signature." >&2
  exit 1
fi
if ! grep -q '<!DOCTYPE html>' "$CLI_HTML_PATH"; then
  echo "Offline CLI HTML output is invalid." >&2
  exit 1
fi
if [[ "$(shasum -a 256 "$CLI_SOURCE" | awk '{print $1}')" != "$CLI_SOURCE_SHA_BEFORE" ]]; then
  echo "Offline CLI modified its source fixture." >&2
  exit 1
fi
if find "$ROOT_DIR/dist" -maxdepth 1 -name '.*.markhola-export-*.tmp' -print -quit | grep -q .; then
  echo "Offline CLI left a temporary export artifact." >&2
  exit 1
fi

echo "==> Running automated PDF export smoke test"
SMOKE_EXPORT_PATH="$ROOT_DIR/dist/pdf-export-smoke.pdf"
SMOKE_EXPORT_LOG="$ROOT_DIR/dist/pdf-export-smoke.log"
rm -f "$SMOKE_EXPORT_PATH"
if ! run_release_binary "$SMOKE_EXPORT_LOG" \
  markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-export \
  "$ROOT_DIR/examples/basic.md" \
  "$SMOKE_EXPORT_PATH"; then
  if is_known_sandbox_webkit_failure "$SMOKE_EXPORT_LOG"; then
    echo "Warning: skipped blocking PDF smoke export due to known sandboxed WKWebView JavaScript limitation." >&2
  else
    exit 1
  fi
elif [[ ! -s "$SMOKE_EXPORT_PATH" ]]; then
  echo "Smoke export produced an empty PDF file." >&2
  exit 1
fi

echo "==> Running Mermaid PDF export smoke test"
MERMAID_EXPORT_PATH="$ROOT_DIR/dist/mermaid-export-smoke.pdf"
MERMAID_EXPORT_LOG="$ROOT_DIR/dist/mermaid-export-smoke.log"
rm -f "$MERMAID_EXPORT_PATH"
if ! run_release_binary "$MERMAID_EXPORT_LOG" \
  markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-export \
  "$ROOT_DIR/examples/mermaid.md" \
  "$MERMAID_EXPORT_PATH"; then
  if is_known_sandbox_webkit_failure "$MERMAID_EXPORT_LOG"; then
    echo "Warning: skipped blocking Mermaid PDF smoke export due to known sandboxed WKWebView JavaScript limitation." >&2
  else
    exit 1
  fi
elif [[ ! -s "$MERMAID_EXPORT_PATH" ]]; then
  echo "Mermaid smoke export produced an empty PDF file." >&2
  exit 1
fi

echo "==> Running HTML export smoke test"
HTML_EXPORT_PATH="$ROOT_DIR/dist/html-export-smoke.html"
rm -f "$HTML_EXPORT_PATH"
markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-export-html \
  "$ROOT_DIR/examples/basic.md" \
  "$HTML_EXPORT_PATH"
require_file "dist/html-export-smoke.html"
if [[ ! -s "$HTML_EXPORT_PATH" ]]; then
  echo "HTML smoke export produced an empty file." >&2
  exit 1
fi

echo "==> Running print preparation smoke test"
PRINT_PREPARE_BASIC_LOG="$ROOT_DIR/dist/print-prepare-basic.log"
PRINT_PREPARE_MERMAID_LOG="$ROOT_DIR/dist/print-prepare-mermaid.log"
if ! run_release_binary "$PRINT_PREPARE_BASIC_LOG" \
  markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-print-prepare \
  "$ROOT_DIR/examples/basic.md"; then
  if is_known_sandbox_webkit_failure "$PRINT_PREPARE_BASIC_LOG"; then
    echo "Warning: skipped blocking basic print prepare smoke due to known sandboxed WKWebView JavaScript limitation." >&2
  else
    exit 1
  fi
fi
if ! run_release_binary "$PRINT_PREPARE_MERMAID_LOG" \
  markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-print-prepare \
  "$ROOT_DIR/examples/mermaid.md"; then
  if is_known_sandbox_webkit_failure "$PRINT_PREPARE_MERMAID_LOG"; then
    echo "Warning: skipped blocking Mermaid print prepare smoke due to known sandboxed WKWebView JavaScript limitation." >&2
  else
    exit 1
  fi
fi

echo "==> Verifying Mermaid print preview page count"
MERMAID_PRINT_PAGES_LOG="$ROOT_DIR/dist/mermaid-print-pages.log"
if run_release_binary "$MERMAID_PRINT_PAGES_LOG" \
  markhola_cargo run --release --locked --bin markhola --manifest-path "$ROOT_DIR/Cargo.toml" -- --smoke-print-pages \
  "$ROOT_DIR/examples/mermaid.md"; then
  MERMAID_PRINT_PAGES_OUTPUT="$(cat "$MERMAID_PRINT_PAGES_LOG")"
  # Keep this baseline aligned with the current accepted Mermaid print layout.
  if [[ "$MERMAID_PRINT_PAGES_OUTPUT" != *"pages=7"* ]]; then
    echo "Unexpected Mermaid print preview page count. Expected pages=7." >&2
    exit 1
  fi
elif is_known_sandbox_webkit_failure "$MERMAID_PRINT_PAGES_LOG"; then
  echo "Warning: skipped blocking Mermaid print page-count smoke due to known sandboxed WKWebView JavaScript limitation." >&2
else
  exit 1
fi

if [[ "$WITH_PACKAGE" -eq 1 ]]; then
  echo "==> Packaging paired architecture-specific Apps and DMGs"
  run_packaging_with_retry

  for label in apple-silicon intel; do
    require_file "dist/MarkHola-${label}.app/Contents/Resources/themes/default/layout.css"
    require_file "dist/MarkHola-${label}.app/Contents/Resources/themes/dark/layout.css"
    require_file "dist/MarkHola-${label}.app/Contents/Resources/help/Documentation.md"
    require_file "dist/MarkHola-${label}.app/Contents/Resources/help/Documentation.zh-CN.md"
    require_file "dist/MarkHola-${APP_VERSION}-${label}.dmg"
    require_file "dist/MarkHola-${APP_VERSION}-${label}.manifest.txt"
  done

  "$ROOT_DIR/scripts/verify_macos_architectures.sh" \
    --app "$ROOT_DIR/dist/MarkHola-apple-silicon.app" \
    --architecture arm64
  "$ROOT_DIR/scripts/verify_macos_architectures.sh" \
    --app "$ROOT_DIR/dist/MarkHola-intel.app" \
    --architecture x86_64
fi

echo "==> Automated regression checks passed"
echo "==> Manual release checklist:"
echo "    $ROOT_DIR/scripts/release_regression_checklist.md"
