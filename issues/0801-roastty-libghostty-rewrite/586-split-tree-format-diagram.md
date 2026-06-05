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

# Experiment 586: split tree formatDiagram and the combined format

## Description

This experiment ports `formatDiagram` and the combined `format` from upstream
`datastruct/split_tree.zig` — the **last split_tree pieces**. `formatDiagram`
renders the tree as an ASCII-art box diagram (one bordered cell per leaf, laid
out by the spatial representation); `format` runs the diagram then the textual
dump (`format_text`, Experiment 585). With these, `terminal::split_tree` is a
complete port of `datastruct/split_tree.zig`. It extends `terminal::split_tree`.

## Upstream behavior

`formatDiagram(writer)`:

1. Empty tree → `empty`.
2. Build the `spatial` representation, then **re-scale** it so the smallest
   nonzero leaf is `1` unit: `min_w`/`min_h` = the smallest nonzero
   `width`/`height` (capped at the normalized `1`), `ratio_w` = `1/min_w`,
   `ratio_h` = `1/min_h`, and every slot field is multiplied by its ratio.
3. `max_label_width` (index path) = `log10(slots.len) + 1` — the digit count
   reserved per label.
4. `cell_width` = `2 + max_label_width + 2` (border + ws + label + ws + border);
   `cell_height` = `3` (border + label + border).
5. Allocate a char `grid`: `ceil(root.width) * cell_width` columns ×
   `ceil(root.height) * cell_height` rows, each row `width + 1` chars (a
   trailing `'\n'`), filled with spaces.
6. For each **leaf** slot (splits skipped; zero-extent slots skipped): cell
   coords `x = floor(slot.x) * cell_width`, `y = floor(slot.y) * cell_height`,
   `w = max(floor(slot.width), 1) * cell_width`,
   `h = max(floor(slot.height), 1) * cell_height`. Draw a box — top/bottom rows
   `+---…-+`, left/right columns `|` — and write the label (the handle index)
   centered: `x_mid = w/2 + x`, `y_mid = h/2 + y`,
   `label_start = x_mid - label_width/2`.
7. Output each row until the first row that starts with a space (a workaround
   for an upstream trailing-blank-line bug in the height calculation).

`format(writer)`: empty → `empty`; else `formatDiagram` (ignoring its error)
then `formatText`.

## Rust mapping (`roastty/src/terminal/split_tree.rs`)

A `Vec<Vec<u8>>` grid (each row `width + 1` bytes ending `'\n'`), the same
scaling / cell math, box drawing, centered index label, and trailing-blank-row
truncation. `f16` `ceil` / `floor` / `max` go through `f32` (the `f16` → `f32`
widening is lossless).

```rust
impl<V> SplitTree<V> {
    /// Write the tree as an ASCII-art box diagram, one bordered cell per leaf (upstream
    /// `formatDiagram`). An empty tree writes `empty`.
    pub(crate) fn format_diagram(&self, out: &mut String) {
        if self.nodes.is_empty() {
            out.push_str("empty");
            return;
        }

        // Scale the spatial representation so the smallest nonzero leaf is 1 unit.
        let mut sp = self.spatial();
        let one = f16::from_f32(1.0);
        let zero = f16::from_f32(0.0);
        let mut min_w = one;
        let mut min_h = one;
        for slot in sp.slots() {
            if slot.width > zero && slot.width < min_w {
                min_w = slot.width;
            }
            if slot.height > zero && slot.height < min_h {
                min_h = slot.height;
            }
        }
        let ratio_w = one / min_w;
        let ratio_h = one / min_h;
        let slots: Vec<Slot> = sp
            .slots()
            .iter()
            .map(|s| Slot {
                x: s.x * ratio_w,
                y: s.y * ratio_h,
                width: s.width * ratio_w,
                height: s.height * ratio_h,
            })
            .collect();

        // Cell dimensions (index-label path: reserve log10(n)+1 digits).
        let max_label_width = self.nodes.len().ilog10() as usize + 1;
        let cell_width = 2 + max_label_width + 2;
        let cell_height = 3;

        // Grid sized from the (scaled) root, rounded up.
        let grid_w = (slots[0].width.to_f32().ceil() as usize) * cell_width;
        let grid_h = (slots[0].height.to_f32().ceil() as usize) * cell_height;
        let mut grid: Vec<Vec<u8>> = (0..grid_h)
            .map(|_| {
                let mut row = vec![b' '; grid_w + 1];
                row[grid_w] = b'\n';
                row
            })
            .collect();

        // Draw each leaf as a box with its handle index centered.
        for (handle, slot) in slots.iter().enumerate() {
            if !matches!(self.nodes[handle], Node::Leaf(_)) {
                continue; // splits are layout-only
            }
            if slot.width == zero || slot.height == zero {
                continue;
            }
            let x = (slot.x.to_f32().floor() as usize) * cell_width;
            let y = (slot.y.to_f32().floor() as usize) * cell_height;
            let w = (slot.width.to_f32().floor().max(1.0) as usize) * cell_width;
            let h = (slot.height.to_f32().floor().max(1.0) as usize) * cell_height;

            // Top and bottom borders.
            for &row_y in &[y, y + h - 1] {
                let row = &mut grid[row_y];
                row[x] = b'+';
                for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
                    *cell = b'-';
                }
                row[x + w - 1] = b'+';
            }
            // Left and right borders.
            for row in grid.iter_mut().take(y + h - 1).skip(y + 1) {
                row[x] = b'|';
                row[x + w - 1] = b'|';
            }

            // Centered handle-index label.
            let label = handle.to_string();
            let x_mid = w / 2 + x;
            let y_mid = h / 2 + y;
            let label_start = x_mid - label.len() / 2;
            grid[y_mid][label_start..label_start + label.len()].copy_from_slice(label.as_bytes());
        }

        // Output rows until the first blank-leading row (the upstream trailing-blank-line workaround).
        for row in &grid {
            if row[0] == b' ' {
                break;
            }
            out.push_str(std::str::from_utf8(row).expect("ascii grid"));
        }
    }

    /// Write the tree as the diagram followed by the textual dump (upstream `format`).
    pub(crate) fn format(&self, out: &mut String) {
        if self.nodes.is_empty() {
            out.push_str("empty");
            return;
        }
        self.format_diagram(out);
        self.format_text(out);
    }
}
```

## Scope / faithfulness notes

- **Ported**: `formatDiagram` / `format` → `SplitTree::format_diagram` /
  `format`. With these, the **index-label** port of `datastruct/split_tree.zig`
  into `terminal::split_tree` is complete; the `splitTreeLabel` view-label path
  of both formatters remains a future refinement (it needs a view-label trait,
  and the index path is upstream's faithful no-label `else` branch).
- **Faithful**: the empty → `empty` case; the spatial re-scaling (smallest
  nonzero leaf → `1`); the index-path `max_label_width` (`log10(n) + 1`); the
  `cell_width` / `cell_height`; the grid sizing (`ceil(root.dim) * cell`); the
  per-leaf box drawing (`+`/`-`/`|` borders) at the floored, cell-scaled
  coordinates; the centered handle-index label (`x_mid`/`y_mid`/`label_start`);
  the trailing-blank-row truncation; and `format`'s diagram-then-text
  composition are all reproduced.
- **Faithful adaptation**: the Zig allocator-backed row arrays become a
  `Vec<Vec<u8>>` (each row `width + 1` bytes ending `'\n'`); `f16` `@ceil` /
  `@floor` / `@max` go through `f32` (lossless widening, then
  `f32::ceil`/`floor`/`max`); `std.math.log10(len)` becomes `len.ilog10()`; the
  index label uses `handle.to_string()`; the `splitTreeLabel` view-label path is
  deferred (the index path is upstream's `else` branch, as in `format_text`);
  `formatDiagram`'s allocation-error paths (`error.WriteFailed`) vanish (Rust
  `Vec` here is infallible).
- **Deferred**: only the `splitTreeLabel` view-label path of the two formatters
  (a future refinement needing a view-label trait); the index path is upstream's
  faithful no-label branch. Otherwise the index-label `split_tree` port is
  complete.
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `terminal::split_tree`.

## Changes

1. `roastty/src/terminal/split_tree.rs`: add `SplitTree::format_diagram` and
   `SplitTree::format`, and update the module doc comment to note the formatters
   (and the whole module) are now complete.
2. Tests (in `split_tree.rs`), with exact ASCII output:
   - **single leaf**: a `5×3` box with `0` centered: `"+---+\n| 0 |\n+---+\n"`.
   - **horizontal split**: two side-by-side `5×3` boxes (`1`, `2`):
     `"+---++---+\n| 1 || 2 |\n+---++---+\n"`.
   - **vertical split**: two stacked boxes (`1` over `2`).
   - **format (combined)**: a single leaf → the diagram followed by `leaf: 0\n`.
   - **format / format_diagram empty**: `empty`.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty terminal::split_tree
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/terminal/split_tree.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `format_diagram` reproduces upstream's box diagram (re-scaling, cell math, box
  drawing, centered index label, trailing-blank-row truncation) and `format`
  composes the diagram then the text — faithful to `datastruct/split_tree.zig`;
- the tests pass (single / horizontal / vertical / combined / empty), and the
  existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the scaling, the cell / grid math, the box drawing,
the label centering, the truncation, or the `format` composition diverges from
upstream, an unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and **approved it** with **no Required or Optional
findings** (one Nit, adopted): the "completes the full `split_tree` port"
wording was softened to "completes the **index-label** port; the
`splitTreeLabel` label-trait path remains a future refinement", to avoid
overstating given the deferred label path.

Codex confirmed the diagram plan is faithful: the scaling uses the minimum
nonzero width/height across all slots with the `1` cap, the cell sizing matches
upstream's index-label branch, the leaf boxes skip splits and zero extents, the
border and label-centering math matches the upstream slices, and the row output
stops at the first blank-leading row; the expected single-leaf and
horizontal-split strings are correct; and `format` as diagram-then-text matches
upstream because the diagram ends with a newline on non-empty trees.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d586-prompt.md`
- Result: `logs/codex-review/20260604-d586-last-message.md`

## Result

**Result:** Pass

`terminal::split_tree` gained `SplitTree::format_diagram` and
`SplitTree::format`. `format_diagram` writes `empty` for an empty tree;
otherwise it builds the spatial representation, re-scales it so the smallest
nonzero leaf is `1` unit (`ratio_w` = `1/min_w`, `ratio_h` = `1/min_h`), sizes
the cells (`cell_width` = `2 + (log10(n)+1) + 2`, `cell_height` = `3`),
allocates a `Vec<Vec<u8>>` grid (`ceil(root.dim) * cell`, each row ending
`'\n'`), draws a `+`/`-`/`|` box for every leaf (splits and zero-extent slots
skipped) at the floored, cell-scaled coordinates with the centered handle-index
label, and outputs rows until the first blank-leading row. `format` writes
`empty` for an empty tree; otherwise the diagram then the textual dump. The
module doc comment was updated to mark the index-label `split_tree` port
complete (only the `splitTreeLabel` view-label path remains a future
refinement).

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3238 passed, 0 failed (six new tests; no regressions,
  up from 3232).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + terminal/split_tree.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

The six new tests (exact-string asserts): `format_diagram` empty → `empty`,
single leaf → `"+---+\n| 0 |\n+---+\n"`, a horizontal split (leaves `1`, `2`) →
`"+---++---+\n| 1 || 2 |\n+---++---+\n"`, a vertical split (stacked `1` over
`2`) → `"+---+\n| 1 |\n+---+\n+---+\n| 2 |\n+---+\n"`, `format` (combined) on a
single leaf → the diagram plus `leaf: 0\n`, and `format` empty → `empty`.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here). Codex confirmed the implementation is
faithful to upstream's index-label path: the scaling uses the minimum nonzero
slot width/height capped at `1`, the cell and grid dimensions match, leaf-only
box drawing skips splits and zero extents, the border placement and centered
handle labels match the upstream math, the output truncates at the first
blank-leading row, and `format` composes the diagram then the text with no extra
separator; the exact-string tests cover single / horizontal / vertical /
combined non-empty / empty, and the gates are clean.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r586-prompt.md` (result)
- Result: `logs/codex-review/20260604-r586-last-message.md` (result)

## Conclusion

This experiment ports `formatDiagram` and the combined `format` — the fourteenth
and final split_tree slice. `format_diagram` renders the tree as an ASCII-art
box diagram (spatial re-scaling so the smallest leaf is one cell, per-leaf box
drawing, centered handle-index labels, trailing-blank-row truncation); `format`
runs the diagram then the textual dump. With these, **`terminal::split_tree` is
a complete index-label port of `datastruct/split_tree.zig`** — only the
`splitTreeLabel` view-label path of the two formatters remains a future
refinement (it needs a view-label trait, and the index path is upstream's
faithful no-label `else` branch). The remaining big-ticket subsystem is the
terminal **search subsystem** (coupled to `PageList` / `Pin` / `Screen` /
`Selection` / `PageFormatter`); the dependency-blocked helpers persist
(regex/oniguruma for `Link::oniRegex`, a URI parser for `os/uri`, the
config-directory naming decision for `file_load` / `edit` / `loadDefaultFiles`).
