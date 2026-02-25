# Ghostty Fork

## Overview

This repo is a fork of [Ghostty](https://github.com/ghostty-org/ghostty). The
original Ghostty commit history is part of our git history — we forked, then
began modifying files in place.

The active Ghostty fork is `gui/`. All browser integration logic is in Zig,
matching Ghostty's architecture. gui/ receives upstream Ghostty merges.

Two earlier Ghostty forks (`ts1/` and `ts5/`) have been archived from the
working tree. See [early-prototypes.md](early-prototypes.md) for their history.

## Remote

| Remote     | URL                                        | Branch |
| ---------- | ------------------------------------------ | ------ |
| `upstream` | https://github.com/ghostty-org/ghostty.git | main   |

The `upstream` remote is shared across all Ghostty copies — they all came from
the same repo.

## How gui/ was created

gui/ was created the same way as ts5:

```bash
git subtree add --prefix=gui upstream main
```

It was originally named `ghost/` (after the working name "Ghost") and later
renamed to `gui/` in Issue 613.

## Merging upstream into gui/

To pull the latest upstream Ghostty changes into gui/:

```bash
git fetch upstream
git subtree pull --prefix=gui upstream main -m "Merge upstream Ghostty into gui"
```

### Resolving conflicts

gui/ has TermSurf modifications in several files (XPC integration, IOSurface
overlay, input forwarding). Upstream merges may conflict with these. Key files
likely to conflict:

- `gui/src/Surface.zig` — Browser state, input routing
- `gui/src/renderer/Metal.zig` — Overlay rendering
- `gui/macos/Sources/App/macOS/AppDelegate.swift` — Debug icon override

### After merging

Verify the build:

```bash
cd gui && zig build
```

If the build fails, common causes are:

- Zig version mismatch (check `gui/build.zig.zon` for the required version)
- New upstream dependencies or build system changes
