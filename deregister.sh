#!/usr/bin/env bash
set -euo pipefail

# Deregister XPC gateways from launchd.
#
# Run this when switching between TermSurf builds (release repo build vs.
# installed /Applications build). Both register the same gateway name
# (com.termsurf.xpc-gateway), so launchd binds the gateway binary to
# whichever app registered last. If you were running the release build and
# then switch to the installed build (or vice versa), the gateway will still
# point to the old app's bundle. Deregister first, then launch the new app.
#
# The debug gateway (com.termsurf.debug.xpc-gateway) has its own name and
# never conflicts, but is included here for completeness.

echo "==> Deregistering XPC gateways..."
launchctl bootout "gui/$(id -u)/com.termsurf.xpc-gateway" 2>/dev/null && \
  echo "  Removed com.termsurf.xpc-gateway" || \
  echo "  com.termsurf.xpc-gateway not registered"
launchctl bootout "gui/$(id -u)/com.termsurf.debug.xpc-gateway" 2>/dev/null && \
  echo "  Removed com.termsurf.debug.xpc-gateway" || \
  echo "  com.termsurf.debug.xpc-gateway not registered"

echo ""
echo "Done. Launch the app you want to use — it will re-register its gateway."
