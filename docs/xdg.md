# XDG Directories

TermSurf follows the
[XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
for storing user data.

## Directories

| Variable          | Default          | TermSurf path              | Contents                                          |
| ----------------- | ---------------- | -------------------------- | ------------------------------------------------- |
| `XDG_CONFIG_HOME` | `~/.config`      | `~/.config/termsurf/`      | Ghostty configuration, TermSurf settings (future) |
| `XDG_DATA_HOME`   | `~/.local/share` | `~/.local/share/termsurf/` | Chromium browser profile data                     |
| `XDG_STATE_HOME`  | `~/.local/state` | `~/.local/state/termsurf/` | Log files                                         |

The folder name is always `termsurf` under the XDG base directory.

## What goes where

**Config** (`XDG_CONFIG_HOME/termsurf/`):

- Ghostty config file (managed by upstream Ghostty)
- Future TermSurf-specific settings

**Data** (`XDG_DATA_HOME/termsurf/`):

- `chromium-profiles/<profile>/` — Per-profile Chromium data (cookies,
  localStorage, browsing history, cached assets). One directory per browser
  profile name.

## Environment variables

If `XDG_DATA_HOME` is set, TermSurf uses it. Otherwise it falls back to
`$HOME/.local/share`. The same pattern applies to `XDG_CONFIG_HOME` (handled by
Ghostty for its config) and `XDG_STATE_HOME` (default: `$HOME/.local/state`).

**State** (`XDG_STATE_HOME/termsurf/`):

- `chromium-server.log` — Chromium Profile Server log output. Created
  automatically on server startup.

## Not used yet

`XDG_CACHE_HOME` (`~/.cache`) is not used. Chromium manages its own cache within
the profile data directory. If TermSurf adds non-Chromium cached data in the
future, it should go in `~/.cache/termsurf/`.
