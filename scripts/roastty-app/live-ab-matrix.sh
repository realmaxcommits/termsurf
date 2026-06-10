#!/usr/bin/env bash
# Issue 802 / Exp 45 — run live A/B recipes and emit one JSON Lines summary per recipe.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HARNESS="$ROOT/scripts/roastty-app/live-ab-smoke.sh"

max_mismatch_ratio="1"
max_mean_channel_delta="255"
selected_recipes=()

usage() {
  cat >&2 <<USAGE
usage: $0 [--recipe NAME ...] [--max-mismatch-ratio N] [--max-mean-channel-delta N]

Defaults to every recipe reported by live-ab-smoke.sh --list-recipes.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --recipe)
      selected_recipes+=("${2:?missing value for --recipe}")
      shift 2
      ;;
    --max-mismatch-ratio)
      max_mismatch_ratio="${2:?missing value for --max-mismatch-ratio}"
      shift 2
      ;;
    --max-mean-channel-delta)
      max_mean_channel_delta="${2:?missing value for --max-mean-channel-delta}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [ "${#selected_recipes[@]}" -eq 0 ]; then
  while IFS= read -r recipe_name; do
    selected_recipes+=("$recipe_name")
  done < <("$HARNESS" --list-recipes)
fi

any_failed=0
for recipe in "${selected_recipes[@]}"; do
  echo "matrix: running recipe=$recipe" >&2
  child_status=0
  child_json="$("$HARNESS" \
    --recipe "$recipe" \
    --max-mismatch-ratio "$max_mismatch_ratio" \
    --max-mean-channel-delta "$max_mean_channel_delta")" || child_status=$?

  if [ "$child_status" -eq 0 ]; then
    status="PASS"
  else
    status="FAIL"
    any_failed=1
  fi

  python3 - "$recipe" "$status" "$child_status" "$child_json" <<'PY'
import json
import sys

recipe, status, child_status, child_json = sys.argv[1:]
try:
    summary = json.loads(child_json)
except json.JSONDecodeError:
    summary = {"error": "invalid_child_json", "raw": child_json}

print(json.dumps({
    "recipe": recipe,
    "status": status,
    "child_exit_status": int(child_status),
    "summary": summary,
}, sort_keys=True, separators=(",", ":")))
PY
done

exit "$any_failed"
