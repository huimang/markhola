#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_DIR="$ROOT_DIR/dist/MarkHola.app"
MODE=""
EXPECTED_MINOS="14.0"

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --app)
      APP_DIR="$2"
      shift 2
      ;;
    --host)
      MODE="host"
      shift
      ;;
    --universal)
      MODE="universal"
      shift
      ;;
    *)
      print -u2 -- "Unknown argument: $1"
      exit 1
      ;;
  esac
done

if [[ -z "$MODE" ]]; then
  print -u2 -- "Specify --host or --universal."
  exit 1
fi

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

if [[ "$MODE" == "universal" ]]; then
  lipo "$EXECUTABLE" -verify_arch arm64 x86_64
  if [[ "${#actual_arches}" -ne 2 ]]; then
    print -u2 -- "Unexpected Universal architectures: ${(j: :)actual_arches}"
    exit 1
  fi
elif [[ "${#actual_arches}" -ne 1 \
  || ("${actual_arches[1]}" != "arm64" && "${actual_arches[1]}" != "x86_64") ]]; then
  print -u2 -- "Host build must contain exactly one supported architecture."
  exit 1
fi

for architecture in "${actual_arches[@]}"; do
  minos="$(xcrun vtool -arch "$architecture" -show-build "$EXECUTABLE" \
    | awk '$1 == "minos" { print $2; exit }')"
  if [[ "$minos" != "$EXPECTED_MINOS" ]]; then
    print -u2 -- "Unexpected deployment target for ${architecture}: ${minos:-missing}; expected ${EXPECTED_MINOS}"
    exit 1
  fi
done

plist_minos="$(plutil -extract LSMinimumSystemVersion raw -o - "$INFO_PLIST")"
if [[ "$plist_minos" != "$EXPECTED_MINOS" ]]; then
  print -u2 -- "Unexpected LSMinimumSystemVersion: $plist_minos"
  exit 1
fi

while IFS= read -r -d '' candidate; do
  if [[ "$candidate" == "$EXECUTABLE" ]]; then
    continue
  fi

  description="$(file -b "$candidate")"
  if [[ "$description" != *"Mach-O"* ]]; then
    continue
  fi

  helper_arches=("${(@s: :)$(lipo -archs "$candidate")}")
  if [[ "${(j: :)${(on)helper_arches}}" != "${(j: :)${(on)actual_arches}}" ]]; then
    print -u2 -- "Unexpected helper architectures in $candidate: ${(j: :)helper_arches}"
    exit 1
  fi

  for architecture in "${helper_arches[@]}"; do
    helper_minos="$(xcrun vtool -arch "$architecture" -show-build "$candidate" \
      | awk '$1 == "minos" { print $2; exit }')"
    if [[ "$helper_minos" != "$EXPECTED_MINOS" ]]; then
      print -u2 -- "Unexpected helper deployment target in ${candidate} (${architecture}): ${helper_minos:-missing}; expected ${EXPECTED_MINOS}"
      exit 1
    fi
  done
done < <(find "$APP_DIR" -type f -print0)

codesign --verify --deep --strict --verbose=2 "$APP_DIR"

print -r -- "Verified $APP_DIR"
print -r -- "Architectures: ${(j: :)actual_arches}"
print -r -- "Deployment target: $EXPECTED_MINOS"
print -r -- "Signature: valid"
