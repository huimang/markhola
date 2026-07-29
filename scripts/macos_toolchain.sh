#!/bin/zsh

set -euo pipefail

MARKHOLA_RUST_TOOLCHAIN="1.95.0"
MARKHOLA_MACOS_DEPLOYMENT_TARGET="14.0"
MARKHOLA_MACOS_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
)

markhola_find_rustup() {
  if [[ -n "${RUSTUP_BIN:-}" && -x "${RUSTUP_BIN}" ]]; then
    print -r -- "${RUSTUP_BIN}"
    return
  fi

  if command -v rustup >/dev/null 2>&1; then
    command -v rustup
    return
  fi

  if [[ -x "/opt/homebrew/opt/rustup/bin/rustup" ]]; then
    print -r -- "/opt/homebrew/opt/rustup/bin/rustup"
    return
  fi

  if command -v brew >/dev/null 2>&1; then
    local brew_rustup
    brew_rustup="$(brew --prefix rustup 2>/dev/null || true)"
    if [[ -n "$brew_rustup" && -x "$brew_rustup/bin/rustup" ]]; then
      print -r -- "$brew_rustup/bin/rustup"
      return
    fi
  fi

  print -u2 -- "Missing rustup. Install it before building MarkHola."
  return 1
}

markhola_prepare_rust_toolchain() {
  MARKHOLA_RUSTUP_BIN="$(markhola_find_rustup)"

  if ! "$MARKHOLA_RUSTUP_BIN" toolchain list \
    | grep -Eq "^${MARKHOLA_RUST_TOOLCHAIN}(-|[[:space:]])"; then
    print -u2 -- "Missing Rust toolchain ${MARKHOLA_RUST_TOOLCHAIN}."
    print -u2 -- "Install it with:"
    print -u2 -- "  $MARKHOLA_RUSTUP_BIN toolchain install ${MARKHOLA_RUST_TOOLCHAIN} --profile minimal --target ${MARKHOLA_MACOS_TARGETS[1]} --target ${MARKHOLA_MACOS_TARGETS[2]}"
    return 1
  fi

  local installed_targets
  installed_targets="$("$MARKHOLA_RUSTUP_BIN" target list \
    --installed \
    --toolchain "$MARKHOLA_RUST_TOOLCHAIN")"

  local target
  for target in "${MARKHOLA_MACOS_TARGETS[@]}"; do
    if ! grep -qx "$target" <<<"$installed_targets"; then
      print -u2 -- "Missing Rust target ${target} for ${MARKHOLA_RUST_TOOLCHAIN}."
      print -u2 -- "Install it with:"
      print -u2 -- "  $MARKHOLA_RUSTUP_BIN target add ${target} --toolchain ${MARKHOLA_RUST_TOOLCHAIN}"
      return 1
    fi
  done

  MARKHOLA_CARGO_BIN="$("$MARKHOLA_RUSTUP_BIN" which cargo \
    --toolchain "$MARKHOLA_RUST_TOOLCHAIN")"
  MARKHOLA_RUSTC_BIN="$("$MARKHOLA_RUSTUP_BIN" which rustc \
    --toolchain "$MARKHOLA_RUST_TOOLCHAIN")"
}

markhola_cargo() {
  MACOSX_DEPLOYMENT_TARGET="$MARKHOLA_MACOS_DEPLOYMENT_TARGET" \
    RUSTC="$MARKHOLA_RUSTC_BIN" \
    "$MARKHOLA_CARGO_BIN" "$@"
}
