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

# Experiment 390: a highlight-aware selected state

## Description

`selected_state` (Experiment 389) yields only `Selection`/`False` — it does not
yet consult the search highlights. Upstream derives `selected` by checking the
selection range first, then a per-row list of **highlights** (each a column
range with a tag), mapping a matching highlight to `Search` or `SearchSelected`.
This experiment makes `selected_state` highlight-aware: it adds the `Highlight`
/ `HighlightTag` types and the highlight loop, so a cell inside a highlight's
range returns the matching search state. The passes pass an **empty** highlight
slice for now (no per-row highlight source is plumbed yet), so behavior is
unchanged; deriving and plumbing the real per-row highlights into the passes is
the follow-up (Experiment 391). This mirrors Experiment 388's additive pattern
(the search colors were added before being wired).

## Upstream behavior

In `rebuildCells` (`renderer/generic.zig`), after the selection check the
`selected` derivation scans the per-row highlights:

```zig
// (selection checked first → .selection)
for (highlights.items) |hl| {
    if (x_compare >= hl.range[0] and x_compare <= hl.range[1]) {
        const tag: HighlightTag = @enumFromInt(hl.tag);
        break :selected switch (tag) {
            .search_match => .search,
            .search_match_selected => .search_selected,
        };
    }
}
break :selected .false;
```

`HighlightTag` is `enum(u8) { search_match, search_match_selected }`. A
highlight is a column `range` (inclusive `[start, end]`, the same `x_compare`
adjustment as selection) plus a `tag`. The **first** matching highlight wins.
Highlights are a renderer input (the per-row render-state highlights,
`row_data.items(.highlights)`) — **not** part of the font shaper's `RunOptions`
(which carries only `selection` and `cursor_x`, for run breaks). So highlights
do not affect shaping.

## Rust mapping (`roastty/src/renderer/cell.rs`)

The `Highlight`/`HighlightTag` types, a shared `x_compare` helper, and the
highlight loop in `selected_state`:

```rust
/// A search highlight's tag (upstream `HighlightTag`): a plain match or a match
/// inside the active selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HighlightTag {
    SearchMatch,
    SearchMatchSelected,
}

/// A search highlight: an inclusive `[start, end]` column range and its tag. A
/// renderer input (upstream's per-row render-state highlights), not a shaper
/// field — highlights do not affect run breaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Highlight {
    pub range: [u16; 2],
    pub tag: HighlightTag,
}

/// A cell's comparison column: a wide cell's spacer tail compares one column to
/// the left (saturating), faithful to upstream's `x_compare`.
fn x_compare(x: u16, wide: Wide) -> u16 {
    if matches!(wide, Wide::SpacerTail) {
        x.saturating_sub(1)
    } else {
        x
    }
}

fn selected_state(
    selection: Option<[u16; 2]>,
    highlights: &[Highlight],
    x: u16,
    wide: Wide,
) -> Selected {
    // Selection takes precedence.
    if is_selected(selection, x, wide) {
        return Selected::Selection;
    }
    // Then the first matching highlight → its search state.
    let xc = x_compare(x, wide);
    for hl in highlights {
        if xc >= hl.range[0] && xc <= hl.range[1] {
            return match hl.tag {
                HighlightTag::SearchMatch => Selected::Search,
                HighlightTag::SearchMatchSelected => Selected::SearchSelected,
            };
        }
    }
    Selected::False
}
```

`is_selected` is refactored to use the shared `x_compare` (its behavior is
unchanged). The passes (`rebuild_bg_row`, `rebuild_row`) call
`selected_state(selection, &[], x, cell.wide)` — an empty highlight slice, since
no per-row highlight source is plumbed yet — so they still produce only
`Selection`/`False` and behavior is unchanged.

## Scope / faithfulness notes

- **Ported (bridged)**: the highlight part of upstream's `selected` derivation —
  the `Highlight`/`HighlightTag` types and the highlight loop in
  `selected_state` (selection first, then the first matching highlight →
  `Search`/`SearchSelected`, else `False`).
- **Faithful**: the highlight loop uses the same `x_compare` adjustment as
  selection (the spacer-tail one-column-left saturating), the inclusive
  `[start, end]` range, the **first**-match-wins order, and the tag mapping
  (`SearchMatch → Search`, `SearchMatchSelected → SearchSelected`) — upstream's
  exact logic. Selection precedence is preserved (`is_selected` checked before
  the loop). `Highlight` is a renderer input, not added to `RunOptions`
  (faithful to upstream's `shape.RunOptions`, which carries no highlights).
- **Faithful adaptation**: `x_compare` is extracted as a shared helper (used by
  both `is_selected` and `selected_state`) to avoid duplicating the spacer-tail
  logic; the passes pass an empty highlight slice until the per-row source is
  plumbed (Experiment 391), so this experiment is additive and
  behavior-preserving (only `Selection`/`False` are produced in the passes).
- **Deferred**: deriving and plumbing the real per-row search highlight ranges
  into the passes (Experiment 391); the lock-cursor glyph + under-cursor
  recolor; the column-ordered decoration merge + link double-underline; the
  Metal upload. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`:
   - add the `HighlightTag` enum and the `Highlight` struct;
   - add the `x_compare` helper; refactor `is_selected` to use it (behavior
     unchanged);
   - `selected_state`: add a `highlights: &[Highlight]` param; after the
     selection check, scan the highlights (first match →
     `Search`/`SearchSelected`);
   - the passes (`rebuild_bg_row`, `rebuild_row`) call
     `selected_state(selection, &[], x, cell.wide)`.
   - Update the existing `selected_state` test for the new signature.
2. Tests (in `cell.rs`): a `selected_state` highlight test —
   - a `SearchMatch` highlight covering a column → `Search`; a
     `SearchMatchSelected` highlight → `SearchSelected`;
   - **selection precedence**: a column in both the selection bounds and a
     highlight → `Selection`;
   - **first-match-wins**: two overlapping highlights with different tags → the
     first's tag;
   - the spacer-tail adjustment applies to highlight matching;
   - empty highlights → `Selection`/`False` as before (the Experiment-389
     behavior).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty selected_state
cargo test -p roastty rebuild
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `selected_state` returns `Search`/`SearchSelected` for a cell inside a
  matching highlight (first-match-wins, after the selection check), `Selection`
  when also selected, and `False` otherwise — faithful to upstream's highlight
  derivation, with the shared `x_compare`;
- the tests pass (the tag mapping, selection precedence, first-match-wins,
  spacer-tail, empty-highlights cases), and the existing tests still pass (the
  passes pass `&[]`, so behavior is unchanged);
- the per-row highlight plumbing, the lock-cursor recolor, and the Metal upload
  stay deferred; `Highlight` is not added to `RunOptions`;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the highlight derivation is wrong (selection not
taking precedence, the wrong tag mapping, last-match instead of first, the
spacer-tail adjustment missing), the passes' behavior changes, or any public C
API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed that not adding highlights to `RunOptions` is correct —
upstream `font.shape.RunOptions` carries shaping inputs (`cells`, `selection`,
`cursor_x`) and no highlights; highlights are renderer row data, so keeping
`Highlight`/`HighlightTag` in `cell.rs` is the faithful boundary. It confirmed
the `selected_state` logic matches upstream (selection precedence, inclusive
highlight ranges, the same spacer-tail `x_compare`, the
`Search`/`SearchSelected` tag mapping, and first-match-wins), and that
extracting `x_compare` is sound and reduces the duplicated spacer-tail logic
without changing `is_selected`. It agreed that passing `&[]` from the row passes
is a clean additive step (preserving current behavior while making the
dispatcher highlight-capable for the next experiment), that deriving the state
independently in the bg/fg passes remains fine (same inputs), and that the test
plan (tag mapping, selection precedence, first-match-wins, spacer-tail matching,
empty-highlight behavior) covers the important failure modes.

Review artifacts:

- Prompt: `logs/codex-review/20260603-203204-234910-prompt.md` (design)
- Result: `logs/codex-review/20260603-203204-234910-last-message.md` (design)

## Result

**Result:** Pass

`selected_state` now consults the search highlights.

- `roastty/src/renderer/cell.rs`:
  - the `HighlightTag` enum (`SearchMatch` / `SearchMatchSelected`) and the
    `Highlight` struct (`range: [u16; 2]`, `tag`) — renderer-local types (not on
    `RunOptions`, since upstream's `shape.RunOptions` carries no highlights);
  - the `x_compare(x, wide)` helper (the spacer-tail `saturating_sub(1)` logic),
    now shared; `is_selected` refactored to use it (behavior unchanged);
  - `selected_state(selection, highlights, x, wide)`: the selection check first
    (→ `Selection`), then the **first** matching highlight (→ `Search` /
    `SearchSelected` by tag), else `False`.
  - both row passes call `selected_state(selection, &[], x, cell.wide)` — an
    empty highlight slice (no per-row source plumbed yet), so behavior is
    unchanged.

Tests (in `cell.rs`):

- `selected_state_yields_selection_or_false` — updated for the new signature
  (passing an empty highlight slice); the `Selection`/`False`/spacer-tail cases
  unchanged.
- `selected_state_consults_highlights` — a `SearchMatch` highlight `[2, 4]` → a
  cell at 3 is `Search`; a `SearchMatchSelected` highlight `[6, 8]` → 7 is
  `SearchSelected`; 5 (outside both) is `False`; selection at the same column
  takes precedence (→ `Selection`); two overlapping highlights with different
  tags → the first's tag (`Search`); a spacer tail at `end + 1 = 5` →
  `x_compare = 4` → in `[2, 4]` → `Search`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2847 passed, 0 failed (+1, no regressions; the
  existing `selected_state` test updated for the new signature, the rebuild
  tests unchanged via `&[]`).
- `cargo build -p roastty` → no warnings (`is_selected` and `x_compare` remain
  used).
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

The per-cell `selected` derivation is now complete and faithful: selection
first, then the first matching search highlight, else nothing. The `Highlight`/
`HighlightTag` types and the highlight-aware `selected_state` are in place; the
only remaining piece for live search highlighting is the per-row highlight
**source** — deriving the real search-match ranges and threading them into the
passes (currently `&[]`).

The remaining renderer-bridge work: deriving and plumbing the per-row search
highlight ranges into the passes (Experiment 391 — threading `&[Highlight]` from
a per-row source through `rebuild_viewport`/`rebuild_bg_row`/`rebuild_row`); the
lock-cursor glyph + under-cursor text recolor; the column-ordered decoration
merge

- link double-underline; and the **Metal upload** of `Contents`.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design:
`HighlightTag` and `Highlight` are renderer-local types not added to
`RunOptions` (faithful, since highlights are render-state input and do not
affect shaping); `x_compare` correctly factors the spacer-tail
`saturating_sub(1)` logic and `is_selected` preserves its prior behavior by
using it; `selected_state` is faithful (selection checked first, highlights
scanned in order with inclusive bounds, first-match-wins, tags mapping to
`Search`/`SearchSelected`); the row passes still pass `&[]` so production
behavior is unchanged until the highlight plumbing arrives in Experiment 391;
`is_selected` and `x_compare` remain used (no dead code); and the tests cover
the tag mapping, selection precedence, first-match-wins, spacer-tail matching,
and empty-highlight behavior, with the diff internal Rust only (no public C
ABI/header change). Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-203438-006187-last-message.md`
