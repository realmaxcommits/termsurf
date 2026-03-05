#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
CHROMIUM_OUT="$REPO_DIR/chromium/src/out/Default"
CHROMIUM_PROTOC="$CHROMIUM_OUT/protoc"

if [ -x "$CHROMIUM_PROTOC" ]; then
  export PROTOC="$CHROMIUM_PROTOC"
fi

cd "$REPO_DIR/roamium"
cargo build "$@"

# Determine target dir based on --release flag.
if [[ " $* " == *" --release "* ]]; then
  SRC="$REPO_DIR/roamium/target/release/roamium"
else
  SRC="$REPO_DIR/roamium/target/debug/roamium"
fi

cp "$SRC" "$CHROMIUM_OUT/roamium"
echo "Copied roamium to $CHROMIUM_OUT/roamium"
