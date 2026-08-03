#!/bin/zsh

set -euo pipefail

usage() {
  print -u2 -- "Usage: $0 --apple-dmg PATH --intel-dmg PATH --apple-sha SHA256 --intel-sha SHA256 [--evidence-dir DIR]"
}

die() { print -u2 -- "ERROR: $*"; exit 1; }
need_command() { command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"; }

for command_name in shasum stat hdiutil ditto find file lipo xcrun plutil codesign awk sort cmp mktemp; do
  need_command "$command_name"
done

APPLE_DMG=""
INTEL_DMG=""
APPLE_SHA=""
INTEL_SHA=""
EVIDENCE_DIR=""
WORK_DIR=""
APPLE_MOUNT=""
INTEL_MOUNT=""
LOG=""
MANIFEST=""
ROOT_DIR=""
VERIFY_ARCH_SCRIPT=""

hash_file() { shasum -a 256 "$1" | awk '{print $1}'; }

record_dmg() {
  local label="$1" dmg_path="$2" expected="$3"
  local actual size
  actual="$(hash_file "$dmg_path")"
  size="$(stat -f '%z' "$dmg_path")"
  [[ "$actual" = "$expected" ]] || die "$label DMG SHA mismatch (expected $expected, got $actual)"
  print -r -- "$label.dmg_path=$dmg_path" >> "$MANIFEST"
  print -r -- "$label.dmg_sha256=$actual" >> "$MANIFEST"
  print -r -- "$label.dmg_size=$size" >> "$MANIFEST"
  hdiutil verify "$dmg_path" || die "$label DMG hdiutil verify failed"
}

mount_and_copy() {
  local label="$1" dmg_path="$2" mount_point="$3" copy_path="$4"
  hdiutil attach -readonly -nobrowse -mountpoint "$mount_point" "$dmg_path" || die "$label DMG read-only mount failed"
  local app_count
  app_count="$(find "$mount_point" -maxdepth 2 -type d -name '*.app' | wc -l | awk '{print $1}')"
  [[ "$app_count" = 1 ]] || die "$label DMG must contain exactly one App bundle"
  local source_app
  source_app="$(find "$mount_point" -maxdepth 2 -type d -name '*.app' -print -quit)"
  ditto "$source_app" "$copy_path" || die "$label App copy failed"
  [[ -x "$copy_path/Contents/MacOS/MarkHola" ]] || die "$label copied App executable missing"
  print -r -- "$label.copied_app=$copy_path" >> "$MANIFEST"
}

verify_app() {
  local label="$1" app="$2" expected_arch="$3"
  local executable="$app/Contents/MacOS/MarkHola" plist="$app/Contents/Info.plist"
  [[ -f "$plist" ]] || die "$label Info.plist missing"
  local arches minos version bundle executable_sha
  arches="$(lipo -archs "$executable")"
  [[ "$arches" = "$expected_arch" ]] || die "$label executable is not thin $expected_arch"
  minos="$(xcrun vtool -arch "$expected_arch" -show-build "$executable" | awk '$1 == "minos" {print $2; exit}')"
  [[ "$minos" = "14.0" ]] || die "$label executable minos is '${minos:-missing}', expected 14.0"
  [[ "$(plutil -extract LSMinimumSystemVersion raw -o - "$plist")" = "14.0" ]] || die "$label LSMinimumSystemVersion is not 14.0"
  version="$(plutil -extract CFBundleShortVersionString raw -o - "$plist")"
  bundle="$(plutil -extract CFBundleIdentifier raw -o - "$plist")"
  [[ "$version" = 0.9.* ]] || die "$label unexpected bundle version: $version"
  [[ "$bundle" = com.markhola.app ]] || die "$label unexpected bundle identifier: $bundle"
  "$VERIFY_ARCH_SCRIPT" --app "$app" --architecture "$expected_arch" || die "$label bundle architecture/signature gate failed"
  executable_sha="$(hash_file "$executable")"
  print -r -- "$label.architecture=$expected_arch" >> "$MANIFEST"
  print -r -- "$label.minos=$minos" >> "$MANIFEST"
  print -r -- "$label.bundle_version=$version" >> "$MANIFEST"
  print -r -- "$label.bundle_id=$bundle" >> "$MANIFEST"
  print -r -- "$label.executable_sha256=$executable_sha" >> "$MANIFEST"
}

write_resource_manifest() {
  local app="$1" output="$2"
  (cd "$app" && find . -type f -print | sort | while IFS= read -r relative; do
    print -r -- "$(hash_file "$app/$relative")  $relative"
  done) > "$output"
}

ROOT_DIR="$(cd "${0:A:h}/.." && pwd)"
VERIFY_ARCH_SCRIPT="$ROOT_DIR/scripts/verify_macos_architectures.sh"
main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --apple-dmg) [[ $# -ge 2 ]] || { usage; exit 2; }; APPLE_DMG="$2"; shift 2 ;;
      --intel-dmg) [[ $# -ge 2 ]] || { usage; exit 2; }; INTEL_DMG="$2"; shift 2 ;;
      --apple-sha) [[ $# -ge 2 ]] || { usage; exit 2; }; APPLE_SHA="$2"; shift 2 ;;
      --intel-sha) [[ $# -ge 2 ]] || { usage; exit 2; }; INTEL_SHA="$2"; shift 2 ;;
      --evidence-dir) [[ $# -ge 2 ]] || { usage; exit 2; }; EVIDENCE_DIR="$2"; shift 2 ;;
      -h|--help) usage; exit 0 ;;
      *) usage; exit 2 ;;
    esac
  done

  [[ "$APPLE_DMG" = /* && "$INTEL_DMG" = /* ]] || die "DMG paths must be absolute"
  [[ -f "$APPLE_DMG" && -f "$INTEL_DMG" ]] || die "Both DMG paths must be existing files"
  [[ "$APPLE_DMG" != "$INTEL_DMG" ]] || die "Apple and Intel DMG paths must differ"
  [[ "$APPLE_SHA" =~ '^[a-fA-F0-9]{64}$' && "$INTEL_SHA" =~ '^[a-fA-F0-9]{64}$' ]] || die "Expected SHA values must be 64 hexadecimal characters"

  if [[ -z "$EVIDENCE_DIR" ]]; then
    EVIDENCE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/markhola-exact-validation.XXXXXX")"
  else
    [[ "$EVIDENCE_DIR" = /* ]] || die "Evidence directory must be absolute"
    [[ ! -e "$EVIDENCE_DIR" ]] || die "Evidence directory already exists; refusing to overwrite evidence"
    mkdir -p "$EVIDENCE_DIR"
  fi

  WORK_DIR="$(mktemp -d "${EVIDENCE_DIR}/.work.XXXXXX")"
  trap 'hdiutil detach "$APPLE_MOUNT" >/dev/null 2>&1 || true; hdiutil detach "$INTEL_MOUNT" >/dev/null 2>&1 || true; rm -rf "$WORK_DIR"' EXIT
  APPLE_MOUNT="$WORK_DIR/apple-mount"
  INTEL_MOUNT="$WORK_DIR/intel-mount"
  mkdir -p "$APPLE_MOUNT" "$INTEL_MOUNT"
  LOG="$EVIDENCE_DIR/validation.log"
  MANIFEST="$EVIDENCE_DIR/paired-manifest.txt"
  exec > >(tee "$LOG") 2>&1

  record_dmg apple "$APPLE_DMG" "$APPLE_SHA"
  record_dmg intel "$INTEL_DMG" "$INTEL_SHA"
  mount_and_copy apple "$APPLE_DMG" "$APPLE_MOUNT" "$WORK_DIR/apple-copy/MarkHola.app"
  mount_and_copy intel "$INTEL_DMG" "$INTEL_MOUNT" "$WORK_DIR/intel-copy/MarkHola.app"
  verify_app apple "$WORK_DIR/apple-copy/MarkHola.app" arm64
  verify_app intel "$WORK_DIR/intel-copy/MarkHola.app" x86_64
  write_resource_manifest "$WORK_DIR/apple-copy/MarkHola.app" "$WORK_DIR/apple.resources.sha256"
  write_resource_manifest "$WORK_DIR/intel-copy/MarkHola.app" "$WORK_DIR/intel.resources.sha256"
  cmp -s "$WORK_DIR/apple.resources.sha256" "$WORK_DIR/intel.resources.sha256" || die "Paired resource manifests differ"
  print -r -- "resource_parity=PASS" >> "$MANIFEST"
  print -r -- "manual.GUI_AX=UNSET" >> "$MANIFEST"
  print -r -- "manual.visual_rendering=UNSET" >> "$MANIFEST"
  print -r -- "manual.trackpad=UNSET" >> "$MANIFEST"
  print -r -- "manual.true_intel_hardware=UNSET" >> "$MANIFEST"
  print -r -- "manual.cli_protocol_smoke=UNSET" >> "$MANIFEST"
  print -r -- "manual.note=No App was launched; no GUI or release PASS was inferred" >> "$MANIFEST"
  print -r -- "release_mutation=NONE" >> "$MANIFEST"
  cat > "$EVIDENCE_DIR/product-manual-checklist.md" <<'EOF'
# Product Manual Checklist

This runner intentionally performs no GUI automation and launches no App.

- [ ] GUI/AX behavior: UNSET
- [ ] Visual rendering and theme review: UNSET
- [ ] Trackpad/mouse interaction: UNSET
- [ ] CLI/protocol smoke: UNSET (headless hook not run by this runner)
- [ ] Physical Intel hardware: UNSET

The objective candidate manifest is not a release decision. Product must bind any
manual result to the copied App path, process identity, and exact candidate SHA.
Release/tag/asset mutation: NONE.
EOF
  cp "$WORK_DIR/apple.resources.sha256" "$EVIDENCE_DIR/apple.resources.sha256"
  cp "$WORK_DIR/intel.resources.sha256" "$EVIDENCE_DIR/intel.resources.sha256"
  print -r -- "Validation complete: objective candidate checks PASS; manual items remain UNSET."
}

if [[ "${MARKHOLA_TEST_SOURCE_ONLY:-0}" != "1" ]]; then
  main "$@"
fi
