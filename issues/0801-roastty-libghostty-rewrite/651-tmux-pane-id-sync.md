+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
session = "019e9ad7-04a6-7b20-823a-fa6e3d24129f"
verdict = "approved"
+++

# Experiment 651: Tmux Pane ID Sync

## Description

Port the first useful slice of upstream `syncLayouts`: discover pane IDs from
window layouts, keep the viewer's tracked pane set in sync, and queue capture
commands for newly discovered panes.

Experiment 650 stopped at window-layout updates. Upstream `syncLayouts` then
walks every window layout, builds a new pane map, preserves existing panes,
creates terminal state for newly discovered panes, queues primary/alternate
history and visible captures for new panes, queues `PaneState` if any panes were
added, and prunes removed panes. This experiment should implement only the
layout-derived pane ID set and command queue behavior. It must not create
per-pane `Terminal` instances or process pane output yet.

## Changes

1. Extend `TmuxViewer` with a tracked pane ID set:
   - store pane IDs in deterministic layout traversal order;
   - expose test accessors if needed;
   - keep IDs unique even if a malformed/repeated layout mentions a pane more
     than once.
2. Add layout traversal helpers:
   - recursively walk `LayoutContent::Horizontal` and `LayoutContent::Vertical`;
   - collect `LayoutContent::Pane(id)` leaves;
   - preserve first-seen order across windows.
3. Add a `sync_layouts` helper for the standalone viewer:
   - collect the pane IDs from all stored windows;
   - identify panes that are present in the new layout set but were not already
     tracked;
   - replace the tracked pane set with the new set, pruning removed pane IDs;
   - for each added pane, queue these commands in upstream order:
     `PaneHistory(primary)`, `PaneVisible(primary)`, `PaneHistory(alternate)`,
     `PaneVisible(alternate)`;
   - if any pane was added, queue `PaneState` after all capture commands.
4. Call `sync_layouts` after successful window-layout changes:
   - after `ListWindows` output is parsed and stored;
   - after a known-window `LayoutChange` updates the stored layout.
5. Preserve command queue sequencing:
   - when sync happens as part of command output, emit `Windows` first, then the
     next queued pane command if one exists;
   - when sync happens from `LayoutChange`, emit `Windows` and emit the first
     queued pane command only if no command was already in flight before the
     notification;
   - do not emit a second command while another command is in flight.
6. Keep these upstream behaviors explicitly out of scope:
   - storing `TmuxPane` terminal state;
   - constructing per-pane `Terminal` instances;
   - applying `PaneHistory`, `PaneVisible`, or `PaneState` output;
   - pane output handling;
   - PTY writes and App/Surface runtime integration.
7. Add tests for:
   - list-windows with a new pane emits `Windows` then first pane capture
     command, and queues the remaining capture/state commands;
   - multiple new panes queue commands in layout traversal order;
   - existing panes are preserved without queuing duplicate captures;
   - removed panes are pruned from the tracked set;
   - duplicate pane IDs in layouts are tracked once;
   - layout-change with an empty queue emits `Windows` then the first queued
     pane command for a new pane;
   - layout-change with an in-flight command queues pane commands but does not
     emit them until that command output is consumed.
8. Keep the README's overall `tmux` checklist item unchecked, refining it after
   the result to say pane ID sync and capture command queueing are done while
   pane terminal state, pane output, PTY, and App integration remain missing.
9. Update this experiment file with result and review records.

## Verification

- `cargo test -p roastty terminal::tmux`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/651-tmux-pane-id-sync.md`
- compare/read the Rust pane sync helpers against:
  - `vendor/ghostty/src/terminal/tmux/viewer.zig` `syncLayouts`
  - `vendor/ghostty/src/terminal/tmux/viewer.zig` `initLayout`
  - `vendor/ghostty/src/terminal/tmux/viewer.zig` command queue emission logic
- `git diff --check`

Pass = Roastty's standalone tmux viewer tracks layout-derived pane IDs, prunes
removed IDs, queues new-pane capture/state commands in upstream order, preserves
command sequencing, and keeps pane terminal state and runtime integration open.

Fail = pane IDs are duplicated or missed, removed panes remain tracked, capture
commands are queued in the wrong order, commands are emitted while another
command is in flight, pane terminal state/output is implemented prematurely, or
the README overclaims full tmux support.

## Design Review

Codex design review session `019e9ad7-04a6-7b20-823a-fa6e3d24129f` found no
blocking issues and approved the experiment for implementation. The reviewer
confirmed that the plan matches the first `syncLayouts` slice: deterministic
layout traversal, unique pane IDs, pruning removed IDs, new-pane capture/state
commands in upstream order, sync after `ListWindows` and known-window
`LayoutChange`, command sequencing preservation, and per-pane terminal state,
pane output, PTY, App, and Surface integration left out of scope.
