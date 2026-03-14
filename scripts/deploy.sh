#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

COMPONENT=""

for arg in "$@"; do
  case "$arg" in
    -*)
      echo "Unknown flag: $arg"
      echo "Usage: $0 <component>"
      echo "Components: website"
      exit 1
      ;;
    *)
      if [ -z "$COMPONENT" ]; then
        COMPONENT="$arg"
      else
        echo "Error: multiple components specified"
        exit 1
      fi
      ;;
  esac
done

if [ -z "$COMPONENT" ]; then
  echo "Usage: $0 <component>"
  echo "Components: website"
  exit 1
fi

deploy_website() {
  echo "==> Building blog and commit data..."
  cd "$REPO_DIR/website"
  bun run build:data

  echo "==> Deploying website to Fly.io..."
  cd "$REPO_DIR"
  fly deploy --config website/fly.toml --dockerfile website/Dockerfile
}

case "$COMPONENT" in
  website) deploy_website ;;
  *)
    echo "Unknown component: $COMPONENT"
    echo "Components: website"
    exit 1
    ;;
esac
