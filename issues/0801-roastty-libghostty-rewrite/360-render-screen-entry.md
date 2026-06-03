+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"
+++

# Experiment 360: the Screen/Terminal-facing shape entry

## Description

Experiment 359 added `PageList::shape_run_options`, but `PageList` and `Screen`
are `pub(super)` — the renderer (in `lib.rs`) cannot call it. This experiment
adds the two-layer wrapper that lifts `shape_run_options` up to a `pub(crate)`
`Terminal` method the renderer can call, exactly mirroring the existing
`render_rows_snapshot` chain (`PageList` → `Screen` → `Terminal` → `lib.rs`). It
threads the **active screen's selection and cursor** into the assembly so the
caller needs no terminal internals.

## Upstream behavior

Upstream's renderer builds, per row, a
`shape.RunOptions { cells, selection, cursor_x }`. The `selection` is the
screen's selection range; `cursor_x` is set from `state.cursor.viewport` only
when the cursor's viewport row equals the row being shaped (`vp.y == y`), and
`RunOptions` nulls `cursor_x` when the cursor is disabled by config
(`if (!config.cursor) self.cursor_x = null;`, `font/shape.zig:92`). roastty
already threads the screen's `selection` into
`PageList::render_rows_snapshot(self.selection)`; this experiment threads the
**same** `self.selection` plus the **active cursor position** into
`shape_run_options`, so the per-row `cursor_x` filter (Experiment 359,
`cy == y`) matches upstream's `vp.y == y`.

## Rust mapping

The `render_rows_snapshot` chain is the exact template:

- `page_list.rs:2132` — `pub(super) fn render_rows_snapshot(&self, selection)`
- `screen.rs:1553` — `pub(super) fn render_rows_snapshot(&self)` →
  `self.pages.render_rows_snapshot(self.selection)`
- `terminal.rs:1505` — `pub(crate) fn render_rows_snapshot(&self)` →
  `self.screens.active().render_rows_snapshot()`
- `lib.rs:1996` — the renderer calls `terminal.render_rows_snapshot()`

This experiment adds the `shape_run_options` siblings of the **Screen** and
**Terminal** layers (the `PageList` layer already exists from Experiment 359):

```rust
// roastty/src/terminal/screen.rs
use crate::font::run::RunOptions;

impl Screen {
    /// Assemble the per-row [`RunOptions`] for the active viewport, threading the
    /// screen's selection and the active cursor position into
    /// [`PageList::shape_run_options`]. Sibling of
    /// [`Self::render_rows_snapshot`].
    pub(super) fn shape_run_options(&self) -> Vec<RunOptions> {
        self.pages
            .shape_run_options(self.selection, Some((self.cursor.x, self.cursor.y)))
    }
}
```

```rust
// roastty/src/terminal/terminal.rs
use crate::font::run::RunOptions;

impl Terminal {
    /// The renderer-facing entry: assemble the active screen's per-row
    /// [`RunOptions`] for the shaper. Sibling of
    /// [`Self::render_rows_snapshot`].
    pub(crate) fn shape_run_options(&self) -> Vec<RunOptions> {
        self.screens.active().shape_run_options()
    }
}
```

## Scope / faithfulness notes

- **Ported (bridged)**: the `Screen`- and `Terminal`-level entries that expose
  `PageList::shape_run_options` to the renderer, threading the active screen's
  `selection` and cursor.
- **Faithful**: the wrapper chain is identical to `render_rows_snapshot` (same
  `self.pages.<m>(self.selection, …)` at the Screen layer, same
  `self.screens.active().<m>()` at the Terminal layer, same `pub(super)` /
  `pub(crate)` visibilities); the cursor is the active screen cursor
  `(self.cursor.x, self.cursor.y)`, and the per-row `cursor_x` filter
  (`cy == y`, Experiment 359) reproduces upstream's `vp.y == y`.
- **Faithful adaptation**: the `Screen` entry always passes the active cursor
  position. Upstream's **config-gated** null
  (`if (!config.cursor) cursor_x = null`) is a renderer/draw-path concern
  (cursor blink/visibility config), so it is **deferred** to the draw path — the
  same pattern as Experiment 359's raw selection range (the assembly emits the
  true position; the renderer decides whether to honor it). "Active viewport"
  means the active visible rows (`Point::active`), as `render_rows_snapshot`
  uses; scrollback-pinned viewport modes are out of scope (as there), so the
  active cursor is always in-viewport and `Some` is correct.
- **Deferred**: the draw-path wiring — running a `RunIterator` over these
  `RunOptions` (with the `CodepointResolver`) and routing the shaped glyphs into
  the Metal renderer's cell/draw path; and the `config.cursor` visibility gate.
  (Consumed by tests now; the renderer caller is a later experiment.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/terminal/screen.rs`: add `Screen::shape_run_options`; import
   `crate::font::run::RunOptions`.
2. `roastty/src/terminal/terminal.rs`: add `Terminal::shape_run_options`; import
   `crate::font::run::RunOptions`.
3. Test (in `terminal.rs`): drive a small `Terminal` (print a couple of cells,
   move the cursor), then assert `terminal.shape_run_options()`:
   - one `RunOptions` per active row;
   - the printed row's cells decode (codepoints at the written columns);
   - `cursor_x` is `Some(col)` only on the cursor's row and `None` elsewhere,
     proving the cursor was threaded from the active screen;
   - `selection` is `None` with no selection;
   - **selection threading**: after installing a selection (`select_all()` →
     `set_selection(Some(..))`), a selected row's `selection` is
     `Some([0, last_col])` — proving the wrapper passes `self.selection` and
     does not drop it to `None`.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty shape_run_options
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Terminal::shape_run_options` (via `Screen::shape_run_options`) returns the
  active screen's per-row `RunOptions` with the threaded selection and cursor,
  mirroring the `render_rows_snapshot` chain;
- the entry test passes, and the existing tests still pass;
- the draw-path wiring and the `config.cursor` gate stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the wrapper diverges from the `render_rows_snapshot`
chain (wrong selection/cursor threading, wrong visibility), the cursor is read
from the wrong place, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Required** finding, now addressed:

- **Required (addressed):** the planned test proved cursor threading but not
  **selection** threading — it only asserted `selection == None`, which would
  still pass if `Screen::shape_run_options` accidentally called
  `self.pages.shape_run_options(None, …)`. The test plan now installs a real
  selection (`select_all()` → `set_selection(Some(..))`) and asserts a selected
  row's `selection` is `Some([0, last_col])`, proving the wrapper passes
  `self.selection`.

Codex confirmed: reading the cursor as `(self.cursor.x, self.cursor.y)` is
correct for this chain (`ScreenCursor` is active-screen row/column state, and
`PageList::shape_run_options` iterates `Point::active` like
`render_rows_snapshot`, so there is no scrollback-viewport offset mismatch
within scope); always passing the active cursor is an acceptable raw assembly
step provided the later draw path nulls `cursor_x` when the cursor should not
affect shaping (renderer visibility/blink/focus state — correctly deferred); the
`pub(super)`/`pub(crate)` visibilities mirror `render_rows_snapshot` and expose
only the renderer-facing terminal method; and the scope (active visible rows
only, draw-path wiring deferred, no C ABI change) is coherent.

Review artifacts:

- Prompt: `logs/codex-review/20260603-171325-868942-prompt.md` (design)
- Result: `logs/codex-review/20260603-171325-868942-last-message.md` (design)
