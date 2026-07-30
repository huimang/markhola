#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_DIR="$ROOT_DIR/dist/MarkHola.app"
EXPECTED_ARCH=""
EXPECTED_MINOS="14.0"

usage() {
  print -u2 -- "Usage: $0 [--app PATH] (--architecture arm64|x86_64 | --host)"
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --app)
      APP_DIR="$2"
      shift 2
      ;;
    --architecture)
      EXPECTED_ARCH="$2"
      shift 2
      ;;
    --host)
      EXPECTED_ARCH="host"
      shift
      ;;
    *)
      usage
      exit 1
      ;;
  esac
done

case "$EXPECTED_ARCH" in
  arm64|x86_64|host) ;;
  *)
    usage
    exit 1
    ;;
esac

EXECUTABLE="$APP_DIR/Contents/MacOS/MarkHola"
INFO_PLIST="$APP_DIR/Contents/Info.plist"

if [[ ! -x "$EXECUTABLE" ]]; then
  print -u2 -- "Missing executable: $EXECUTABLE"
  exit 1
fi

if [[ ! -f "$INFO_PLIST" ]]; then
  print -u2 -- "Missing Info.plist: $INFO_PLIST"
  exit 1
fi

actual_arches=("${(@s: :)$(lipo -archs "$EXECUTABLE")}")
if [[ "${#actual_arches}" -ne 1 ]]; then
  print -u2 -- "Expected one thin architecture in $EXECUTABLE; found ${(j: :)actual_arches}"
  exit 1
fi

ACTUAL_ARCH="${actual_arches[1]}"
if [[ "$EXPECTED_ARCH" == "host" ]]; then
  if [[ "$ACTUAL_ARCH" != "arm64" && "$ACTUAL_ARCH" != "x86_64" ]]; then
    print -u2 -- "Host build contains unsupported architecture: $ACTUAL_ARCH"
    exit 1
  fi
else
  if [[ "$ACTUAL_ARCH" != "$EXPECTED_ARCH" ]]; then
    print -u2 -- "Unexpected architecture in $EXECUTABLE: $ACTUAL_ARCH; expected $EXPECTED_ARCH"
    exit 1
  fi
fi

verify_minos() {
  local macho_path="$1"
  local architecture="$2"
  local minos
  minos="$(xcrun vtool -arch "$architecture" -show-build "$macho_path" \
    | awk '$1 == "minos" { print $2; exit }')"
  if [[ "$minos" != "$EXPECTED_MINOS" ]]; then
    print -u2 -- "Unexpected deployment target in $macho_path ($architecture): ${minos:-missing}; expected $EXPECTED_MINOS"
    exit 1
  fi
}

verify_minos "$EXECUTABLE" "$ACTUAL_ARCH"

plist_minos="$(plutil -extract LSMinimumSystemVersion raw -o - "$INFO_PLIST")"
if [[ "$plist_minos" != "$EXPECTED_MINOS" ]]; then
  print -u2 -- "Unexpected LSMinimumSystemVersion: $plist_minos"
  exit 1
fi

MACHO_COUNT=0
while IFS= read -r -d '' candidate; do
  description="$(file -b "$candidate")"
  if [[ "$description" != *"Mach-O"* ]]; then
    continue
  fi

  MACHO_COUNT=$((MACHO_COUNT + 1))
  candidate_arches=("${(@s: :)$(lipo -archs "$candidate")}")
  if [[ "${#candidate_arches}" -ne 1 || "${candidate_arches[1]}" != "$ACTUAL_ARCH" ]]; then
    print -u2 -- "Unexpected Mach-O architectures in $candidate: ${(j: :)candidate_arches}; expected only $ACTUAL_ARCH"
    exit 1
  fi
  verify_minos "$candidate" "$ACTUAL_ARCH"
done < <(find "$APP_DIR" -type f -print0)

if [[ "$MACHO_COUNT" -lt 1 ]]; then
  print -u2 -- "No Mach-O files found in $APP_DIR"
  exit 1
fi

codesign --verify --deep --strict --verbose=2 "$APP_DIR"

print -r -- "Verified $APP_DIR"
print -r -- "Architecture: $ACTUAL_ARCH"
print -r -- "Mach-O files: $MACHO_COUNT"
print -r -- "Deployment target: $EXPECTED_MINOS"
print -r -- "Signature: valid"
