# Issue 742: Archive Ghostboard

## Goal

Archive the `ghostboard/` directory to reduce maintenance burden. Wezboard is
the sole active GUI during protocol iteration. Ghostboard will be re-created
from a fresh Ghostty fork after the protocol stabilizes, closer to launch.

## Background

TermSurf currently maintains two GUI implementations: Ghostboard (Ghostty fork,
Zig) and Wezboard (WezTerm fork, Rust). Every protocol change requires
implementation in both. The protocol is evolving rapidly — Issue 741 just added
direct TUI↔Browser connections, and many more changes are coming (proto split,
new message types for downloads, dialogs, bookmarks, etc.).

Maintaining two GUIs during this period doubles the implementation work for
every protocol change with no user-facing benefit. Wezboard is the better choice
for iteration because:

- **Cross-platform** — WezTerm works on macOS, Linux, and Windows. Ghostty does
  not support Windows, making Ghostboard unsuitable for the cross-platform
  milestone.
- **Same language as the ecosystem** — Wezboard, Roamium, and the TUI are all
  Rust. Protocol changes often touch all three. Having the GUI in Rust too
  eliminates context-switching between Zig and Rust.
- **Active development** — Wezboard has full protocol support, CALayerHost
  rendering, input forwarding, and the direct TUI↔Browser connection from
  Issue 741. Ghostboard is missing the direct connection (Issue 741 was
  Wezboard-only) and would need porting work just to catch up.

Ghostboard will return. The vision — multiple terminal emulators speaking the
TermSurf protocol — is central to the project. But the right time to fork
Ghostty again is after the protocol stabilizes, when Ghostty itself will have
months of additional development. Re-creating Ghostboard from a fresh fork will
be cleaner than maintaining a stale fork through dozens of protocol changes.

The archived directory will be called "Ghostboard Legacy" to distinguish it from
the future re-creation.

## Analysis

### What to archive

- `ghostboard/` — The entire Ghostty fork directory. This is a git subtree
  import, so all history is preserved in git.

### What to update

- `docs/early-prototypes.md` — Add a "Ghostboard Legacy" entry to the Archive
  Log table with the commit hash, date, and notes.
- `CLAUDE.md` — Remove Ghostboard from the active development sections. Update
  the GUI table to show only Wezboard as active. Remove the Ghostboard source
  layout, build commands, and upstream merge instructions. Keep the mention in
  the vision section (Ghostboard will return).
- `TODO.md` — Update the 1.0 Milestone to reflect that Ghostboard is deferred.
- `scripts/` — Remove or update scripts that reference Ghostboard
  (`build.sh ghostboard`, `install.sh ghostboard`, `rename-ghostty.sh`, etc.).

### What NOT to change

- Issue documents in `issues/` — These are immutable historical records. They
  reference Ghostboard extensively and that's correct — it was active at the
  time.
- `roamium/` — Roamium works with any GUI. No changes needed.
- `webtui/` — The TUI doesn't know which GUI it's connected to. No changes
  needed.
