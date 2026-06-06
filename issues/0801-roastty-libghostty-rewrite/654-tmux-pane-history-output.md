# Experiment 654: Tmux Pane History Output

## Description

Experiment 653 wired `PaneVisible` command output into each tracked pane
terminal. The preceding `PaneHistory` command output is still ignored, so tmux
bootstrap can reconstruct the visible area but not the historical scrollback.

Upstream Ghostty's `Viewer.receivedPaneHistory` switches the pane terminal to
the requested primary or alternate screen, streams the captured history bytes
through the terminal parser, then clears the active area by issuing carriage
return, indexing once per terminal row, and homing the cursor to `1,1`. The
active area is intentionally left blank because the following visible capture
fills it.

This experiment ports only that history command-output behavior. Pane state,
live `%output`, PTY startup, and App integration remain future work.

## Changes

- `roastty/src/terminal/terminal.rs`
  - Add a narrow tmux-facing helper that runs the post-history cleanup sequence:
    carriage return, true `index` semantics once per terminal row, and cursor
    home. This must use the existing internal `index()` behavior or an exact
    equivalent based on `line_feed_basic`, not normal line-feed handling that
    honors linefeed mode by also doing carriage return.
  - Reuse the existing tmux screen-switch helper from Experiment 653.
- `roastty/src/terminal/tmux.rs`
  - Route `TmuxCommand::PaneHistory` command output to the matching
    `TmuxPane.terminal`.
  - Ignore history output for unknown panes, matching upstream stale-pane
    behavior.
  - Treat terminal replay or post-history cleanup failures as viewer-defunct
    conditions.
  - Preserve command-queue continuation after consuming history output.
  - Keep `PaneState`, live `%output`, PTY startup, and App integration as future
    work.
- Tests in `roastty/src/terminal/tmux.rs`
  - Verify primary history replay leaves the active area empty and creates
    scrollback containing the replayed history.
  - Verify alternate history replay affects the alternate screen without
    polluting the primary screen.
  - Verify consuming `PaneHistory` emits the next queued command when present.
  - Verify stale pane IDs are consumed without defuncting the viewer or changing
    tracked pane content.
  - Verify terminal replay/cleanup failure defuncts the viewer if a practical
    fixture exists; otherwise document why this path is not directly
    fixture-tested.

## Verification

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/654-tmux-pane-history-output.md`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `cargo test -p roastty terminal::tmux`
- `git diff --check`

## Design Review

**Result:** Not approved on first review.

Codex found one blocker: the original design said "line-feed/index," but
upstream specifically uses `index()` and Roastty's normal line-feed handling can
also do carriage return when linefeed mode is enabled. The design was revised to
require true `index` semantics through the existing internal `index()` behavior
or an exact `line_feed_basic` equivalent. The review also suggested making the
scrollback assertion explicit and documenting replay/cleanup failure coverage;
both were added.

**Re-review result:** Approved.

Codex confirmed the blocker was resolved and the design now matches upstream's
`carriageReturn`, `index` once per row, then `setCursorPos(1,1)` sequence. It
also confirmed the scope remains narrow and the verification is sufficient.
