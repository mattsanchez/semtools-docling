#!/usr/bin/env bash
set -euo pipefail

DEFAULT_TARGETS=(
  aarch64-apple-darwin
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
)

usage() {
  cat <<EOF
Usage: $(basename "$0") [target ...]

Builds the specified Rust targets. If none are provided, builds the default set.

Default targets:
  ${DEFAULT_TARGETS[0]}
  ${DEFAULT_TARGETS[1]}
  ${DEFAULT_TARGETS[2]}
EOF
}

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
  esac
done

if [[ "$#" -gt 0 ]]; then
  TARGETS=("$@")
else
  TARGETS=("${DEFAULT_TARGETS[@]}")
fi

is_darwin=false
if [[ "$(uname -s)" == "Darwin" ]]; then
  is_darwin=true
fi

for target in "${TARGETS[@]}"; do
  echo "==> Building $target"

  if $is_darwin && [[ "$target" == *"-unknown-linux-"* ]]; then
    if cargo zigbuild -V >/dev/null 2>&1; then
      cargo zigbuild --release --locked --target "$target"
    else
      echo "cargo-zigbuild not found; attempting cargo build (requires a Linux cross toolchain)." >&2
      cargo build --release --locked --target "$target"
    fi
  else
    cargo build --release --locked --target "$target"
  fi

done
