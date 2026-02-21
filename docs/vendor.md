# Vendor Repos

Cloned repos for reading source code and learning. These are gitignored and not
committed to the TermSurf repo.

## vendor/

| Repo | URL | Why |
|------|-----|-----|
| `vendor/ghostty/` | https://github.com/ghostty-org/ghostty | TermSurf GUI forks Ghostty. Reference for understanding upstream behavior, diffing changes, and planning merges. |
| `vendor/wezterm/` | https://github.com/wezterm/wezterm | Terminal emulator evaluated in ts2–ts3. Reference for terminal internals and IPC patterns. |
| `vendor/electron/` | https://github.com/electron/electron | Reference for Chromium embedding patterns, patch sets, and Content API usage. |
| `vendor/alacritty/` | https://github.com/alacritty/alacritty | Terminal emulator evaluated in ts4. Reference for Rust terminal architecture. |

## Chromium (special case)

Chromium lives at `chromium/src/` (not in `vendor/`). The repo is too large to
have two clones, so `chromium/src/` serves double duty: it is both the build
workspace for TermSurf's Chromium fork and the source code reference. When
studying Chromium internals (e.g., `WebContentsObserver`, Content API, compositor
pipeline), read from `chromium/src/` directly.

| Path | Upstream |
|------|----------|
| `chromium/src/` | https://chromium.googlesource.com/chromium/src |
