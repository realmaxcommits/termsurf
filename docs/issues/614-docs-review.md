# Issue 614: Review documentation for accuracy and conciseness

## Goal

All living documentation is accurate, concise, and reflects the current state of
the project after the renames in Issues 611–613.

## Background

Issues 611 (rename Ghostty → TermSurf), 612 (app icon), and 613 (rename ghost/ →
gui/, web/ → tui/) changed names, paths, and branding across the project. The
living documents were updated for path references, but haven't been reviewed
holistically for accuracy, stale content, or unnecessary verbosity.

### Files to review

**Top-level:**

- `AGENTS.md`
- `CHANGELOG.md`
- `CLAUDE.md`
- `README.md`
- `TODO.md`

**docs/:**

- `docs/chromium.md`
- `docs/ghostty.md`
- `docs/keybindings.md`
- `docs/vsync.md`

**TUI:**

- `tui/` has no README. One may need to be created.

### What to look for

- Stale references to old names (`ghost/`, `web/`, `Ghostty.app`,
  `com.mitchellh.ghostty`)
- Outdated architecture descriptions that don't match current state
- Unnecessary verbosity — documentation that could be shorter without losing
  information
- Missing information about recent changes (icon, rename, directory structure)
- Accuracy of build commands, launch commands, and file paths

### Out of scope

- `CLAUDE.md` agent instructions (the "AI Guidance" and "AI Reminder" sections)
  — these are functional directives, not documentation
- Historical issue docs (`docs/issues/`) — left as-is per Issue 613
- `gui/` internal docs — owned by upstream Ghostty
