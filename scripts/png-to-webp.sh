#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <input.png> <output.webp>"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

# Resolve paths before cd
INPUT="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUTPUT_DIR="$(cd "$(dirname "$2")" && pwd)"
OUTPUT="$OUTPUT_DIR/$(basename "$2")"

cd "$REPO_DIR/website"
bun run scripts/png-to-webp.ts "$INPUT" "$OUTPUT"
