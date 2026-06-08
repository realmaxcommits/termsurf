+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.result]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 844: Derive row_never_extend from the terminal rows

## Description

Exp 842/843 derive the render input's colors, palette, and cursor from the live
terminal but `FrameRenderState.row_never_extend` is still the all-false stub
(named deferred in 842). This experiment, the next slice of the input-derivation
arc, replaces it with the real per-row derivation
`cell::row_never_extend_bg_flags` (`cell.rs:272`) — the **last** stubbed field
in the assembled input.

`row_never_extend_bg(row, palette, default_bg)` returns true when a row should
not have window padding extended into it: a semantic-prompt row, a perfect-fit
powerline cell, or any cell whose background resolves to the default background
(`cell.rs:238`). It needs the shaped rows (`&[RunOptions]`), which the terminal
provides via `terminal.shape_run_options()` — the same call the snapshot makes.

**Known perf cost (deferred, not hidden):** `from_terminal` will call
`shape_run_options()` to derive `row_never_extend`, and
`FrameTerminalSnapshot:: collect` calls it again during the frame — two shapings
per frame. They align because both shape the same terminal state. Sharing one
shaping (e.g. threading the snapshot's rows into the input, or moving the input
derivation into the snapshot) is a later refactor slice; this slice keeps
`FrameRenderState` independent of the snapshot.

## Changes

`roastty/src/renderer/frame_renderer.rs` (production code + tests).

- In `FrameRenderState::from_terminal`, replace the
  `row_never_extend: vec![false; terminal.rows() as usize]` stub with:

  ```rust
  let rows = terminal.shape_run_options();
  let row_never_extend = row_never_extend_bg_flags(&rows, &palette, default_bg);
  ```

  (`row_never_extend_bg_flags` imported from `crate::renderer::cell`.)

- Update the `FrameRenderState` doc comment: `row_never_extend` is no longer a
  stub — it is derived; the doc's "stubs until their own slices" list drops it.

No change elsewhere; `row_never_extend` length is still `terminal.rows()` (one
flag per shaped row), so padding-extend validation is unaffected.

## Verification

Per the bounded-run convention (15-min cap, Central-stamped, single tracked
task, no poll-watcher). Fast non-Metal unit tests in `frame_renderer.rs`:

- **Derived row_never_extend matches the per-row helper:** for a populated
  terminal,
  `from_terminal().row_never_extend == row_never_extend_bg_flags(&terminal.shape_run_options(), &palette, default_bg)`
  (faithful wiring), and length `== terminal.rows()`.
- **Concrete flags (all-true default case):** a 4×3 terminal yields
  `row_never_extend == [true, true, true]` — **every** row never-extends,
  because a blank cell is a `Codepoint` cell with the default style whose
  `resolve_bg` is `None`, hitting `row_never_extend_bg`'s default-background arm
  (matching upstream `row.zig`). So the all-false stub was behaviorally wrong,
  not just incomplete.
- **A non-default-background row is false:** fill one row's columns with an
  explicit non-default background (e.g. `\x1b[2;1H\x1b[41mBBBB`, palette-red bg
  on row 1) and assert that row's flag is `false` while the others stay `true` —
  proving the derivation distinguishes extend from never-extend, not just
  returns all-true.
- **Still drives a frame:** `FrameRenderState::from_terminal` + `rebuild_input`
  feeds `FrameRenderer::update_frame` on a 4×3 terminal and rebuilds the full
  frame with the derived (non-stub) `row_never_extend` (the padding-extend stage
  accepts it).
- `cargo build -p roastty` — no warnings. `cargo fmt -p roastty -- --check` —
  clean. Full suite via `scripts/bounded-run.sh` (default parallelism) stays
  green. No-ghostty grep — clean. `git diff --check` — clean.

**Pass** = the new row_never_extend tests pass, a terminal-derived input still
rebuilds a frame, and the full suite stays green. **Partial/Fail** = any test
fails or the suite regresses.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Verified the args (`&palette`/`default_bg` match the helper), the
one-flag-per-row length, the honest duplicate-shaping flag and alignment claim,
and that the 842 frame test's assertions are insensitive to the padding-extend
path.

**Verdict:** CHANGES REQUIRED → fixed. One Required + one Optional:

- **Required — wrong concrete expectation.** `[true, false, false]` was wrong: a
  blank cell is a `Codepoint` cell (`page.rs:3289`, default content tag 0) with
  the default style whose `resolve_bg` is `None`, so `row_never_extend_bg`
  returns **true** for empty rows too — a default 4×3 terminal is **all-true**
  `[true, true, true]` (faithful to upstream `row.zig:55-57`). **Fixed:** the
  concrete case now asserts `[true, true, true]`, and a separate test fills a
  row with a non-default explicit background to exercise the `false` branch.
- **Optional — stale line refs.** **Fixed:** `row_never_extend_bg_flags` is
  `cell.rs:272`, `row_never_extend_bg` is `cell.rs:238`.

(The reviewer could not, under the read-only constraint, prove
`refine_padding_extend_rows` does not panic on a `true` flag; the implementation
will verify the frame test passes before recording the result.)

## Conclusion

_(to be written after the run)_
