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

# Experiment 584: split tree resize (move a split's divider by a grid-relative delta)

## Description

This experiment ports `resize` (and its helpers `findParentSplit` /
`resizeInPlace`) from upstream `datastruct/split_tree.zig` — the **last
tree-shaping operation**. `resize` moves the nearest layout-matching split's
divider by a signed `ratio` delta expressed as a fraction of the **whole grid**
(so it matches a user's mental model of moving a divider relative to the
window). It reuses the `Backtrack` enum (Experiment 580) and the `Spatial`
representation (Experiment 578). It extends `terminal::split_tree`.

## Upstream behavior

```zig
pub fn resize(self, gpa, from, layout, ratio) !Self {
    assert(ratio >= -1 and ratio <= 1 and !isNan(ratio) and !isInf(ratio));
    if (self.isEmpty()) return .empty;
    var result = try self.clone(gpa);                       // worst case: an unchanged clone

    const parent_handle = switch (self.findParentSplit(layout, from, .root)) {
        .deadend, .backtrack => return result,              // no matching parent split → no change
        .result => |v| v,
    };

    var sp = try result.spatial(gpa);
    // `scale` = the split's grid fraction along the resize axis (normalized, so root's is 1).
    const scale = switch (layout) {
        .horizontal => sp.slots[parent_handle.idx()].width  / sp.slots[0].width,
        .vertical   => sp.slots[parent_handle.idx()].height / sp.slots[0].height,
    };
    if (scale == 0) return result;                          // a zero-extent split can't be resized

    // The grid-relative delta becomes a split-relative delta by dividing by `scale`.
    const new_ratio = result.nodes[parent_handle.idx()].split.ratio + (ratio / scale);
    result.resizeInPlace(parent_handle, @min(@max(new_ratio, 0), 1));  // clamp to [0, 1]
    return result;
}

fn resizeInPlace(self, at, ratio) void { self.nodes[at.idx()].split.ratio = ratio; }

fn findParentSplit(self, layout, from, current) Backtrack {  // nearest ancestor split matching layout
    if (from == current) return .backtrack;
    return switch (self.nodes[current.idx()]) {
        .leaf => .deadend,
        .split => |s| switch (self.findParentSplit(layout, from, s.left)) {
            .result => |v| .{ .result = v },
            .backtrack => if (s.layout == layout) .{ .result = current } else .backtrack,
            .deadend => switch (self.findParentSplit(layout, from, s.right)) {
                .deadend => .deadend,
                .result => |v| .{ .result = v },
                .backtrack => if (s.layout == layout) .{ .result = current } else .backtrack,
            },
        },
    };
}
```

`findParentSplit` is a backtracking search (like `previousBacktrack`): it walks
down to `from`, then backtracks up, and the first ancestor split whose layout
matches is the result (backtracking past a mismatched split continues upward).
`resize` then: clones; if there is no matching parent split (or `from` is the
root), returns the clone unchanged; computes `scale` (the parent split's
normalized extent along the axis, with the root's being `1`); divides the
grid-relative `ratio` delta by `scale` to get a split-relative delta; adds it to
the split's current ratio; and writes the clamped `[0, 1]` result. `resize`
itself **preserves** `zoomed` (it returns a `clone`, whose zoom is kept, and
`resizeInPlace` only changes a ratio); any unzooming on a resize happens outside
this function (in the caller).

## Rust mapping (`roastty/src/terminal/split_tree.rs`)

`findParentSplit` mirrors `previous_backtrack` (left, then right, resolving a
backtrack to the current split if its layout matches). `resize` reuses `clone` /
`spatial`, computes the `f16` scale and new ratio, and clamps via comparison.

```rust
impl<V> SplitTree<V> {
    /// Move the nearest split matching `layout` (an ancestor of `from`) by `ratio`, a signed delta
    /// as a fraction of the whole grid (upstream `resize`). Returns a clone unchanged if there is no
    /// matching split or the split has zero extent. `ratio` must be a finite value in `[-1, 1]`.
    pub(crate) fn resize(&self, from: Handle, layout: Layout, ratio: f16) -> SplitTree<V> {
        let zero = f16::from_f32(0.0);
        let one = f16::from_f32(1.0);
        assert!(ratio >= f16::from_f32(-1.0) && ratio <= one);
        assert!(!ratio.is_nan() && !ratio.is_infinite());

        if self.is_empty() {
            return SplitTree::empty();
        }
        let mut result = self.clone(); // worst case: returned unchanged

        let parent = match self.find_parent_split(layout, from, Handle::ROOT) {
            Backtrack::Result(v) => v,
            Backtrack::Deadend | Backtrack::Backtrack => return result,
        };

        let sp = result.spatial();
        let parent_slot = sp.slots()[parent.idx()];
        let root_slot = sp.slots()[0];
        let scale = match layout {
            Layout::Horizontal => parent_slot.width / root_slot.width,
            Layout::Vertical => parent_slot.height / root_slot.height,
        };
        if scale == zero {
            return result;
        }

        let current_ratio = match &result.nodes[parent.idx()] {
            Node::Split(s) => s.ratio,
            Node::Leaf(_) => unreachable!("find_parent_split returns a split"),
        };
        let new_ratio = current_ratio + (ratio / scale);
        let clamped = if new_ratio < zero {
            zero
        } else if new_ratio > one {
            one
        } else {
            new_ratio
        };
        result.resize_in_place(parent, clamped);
        result
    }

    /// Set the split at `at`'s ratio in place (upstream `resizeInPlace`). `at` must be a split.
    fn resize_in_place(&mut self, at: Handle, ratio: f16) {
        match &mut self.nodes[at.idx()] {
            Node::Split(s) => s.ratio = ratio,
            Node::Leaf(_) => unreachable!("resize_in_place expects a split handle"),
        }
    }

    /// The nearest ancestor split of `from` whose layout is `layout` (upstream `findParentSplit`).
    fn find_parent_split(&self, layout: Layout, from: Handle, current: Handle) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }
        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(s) => {
                let s = *s;
                match self.find_parent_split(layout, from, s.left) {
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Backtrack => {
                        if s.layout == layout {
                            Backtrack::Result(current)
                        } else {
                            Backtrack::Backtrack
                        }
                    }
                    Backtrack::Deadend => match self.find_parent_split(layout, from, s.right) {
                        Backtrack::Deadend => Backtrack::Deadend,
                        Backtrack::Result(v) => Backtrack::Result(v),
                        Backtrack::Backtrack => {
                            if s.layout == layout {
                                Backtrack::Result(current)
                            } else {
                                Backtrack::Backtrack
                            }
                        }
                    },
                }
            }
        }
    }
}
```

## Scope / faithfulness notes

- **Ported**: `resize` / `findParentSplit` / `resizeInPlace` → the same in
  `terminal::split_tree`.
- **Faithful**: the `[-1, 1]` / finite `ratio` assertions; the empty-tree →
  empty fast path; the clone-then-maybe-modify (no matching split / zero scale ⇒
  unchanged clone); `findParentSplit`'s backtracking (nearest layout-matching
  ancestor split of `from`, mirroring `previousBacktrack`); the `scale` as the
  split's normalized grid extent along the axis (root's being `1`); the
  grid-relative-to-split-relative `ratio / scale` conversion; the
  `+ current_ratio`; the `[0, 1]` clamp; and `resizeInPlace` are all reproduced.
- **Faithful adaptation**: `findParentSplit` is run on `self` (the clone is
  structurally identical), while `spatial` and `resizeInPlace` run on `result`
  (the clone) — exactly as upstream; `@min(@max( …, 0), 1)` becomes a three-way
  comparison clamp (the inputs are finite, so the comparison is unambiguous);
  the `@constCast` in `resizeInPlace` (to mutate the const-but-owned node) is
  unnecessary in Rust (the clone is plainly mutable); the `f16` arithmetic (`/`,
  `+`, the comparisons) reproduces upstream's binary16.
- **Deferred**: only the formatters (`formatText` / `formatDiagram`) remain
  after this — the tree-shaping operations will be complete.
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `terminal::split_tree`.

## Changes

1. `roastty/src/terminal/split_tree.rs`: add `SplitTree::resize`,
   `resize_in_place`, and `find_parent_split`, and update the module doc comment
   to move `resize` into the landed tree-shaping set (leaving only the
   formatters deferred).
2. Tests (in `split_tree.rs`):
   - **empty**: `resize` of the empty tree is empty.
   - **basic resize (scale 1)**: a 2-leaf horizontal split,
     `resize(leaf, Horizontal, 0.25)` → the root ratio becomes `0.75` (the root
     split spans the whole grid, so `scale == 1`).
   - **layout mismatch is a no-op**: the same tree,
     `resize(leaf, Vertical, 0.25)` leaves the ratio unchanged (no vertical
     ancestor split).
   - **from the root is a no-op**: `resize(ROOT, Horizontal, 0.25)` leaves the
     tree unchanged (the root has no parent split).
   - **clamping**: `resize(leaf, Horizontal, 1.0)` clamps the new ratio to
     `1.0`; `resize(leaf, Horizontal, -1.0)` clamps to `0.0`.
   - **nested scale**: a tree where the target split occupies half the grid
     (`scale == 0.5`) — a `0.125` grid delta becomes a `0.25` split delta, so
     the inner split's ratio goes `0.5 → 0.75`.
   - **zoom preserved**: `resize` keeps the `zoomed` handle on both the changed
     and the no-op paths.
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

- `resize` finds the nearest layout-matching ancestor split, scales the
  grid-relative delta by the split's grid extent, applies and clamps the new
  ratio (with the empty / no-match / zero-scale / root-from no-ops) — faithful
  to `datastruct/split_tree.zig`;
- the tests pass (empty / basic / layout-mismatch / root / clamp /
  nested-scale), and the existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the parent-split search, the scale computation, the
ratio delta / clamp, or the no-op cases diverge from upstream, an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and found **no Required findings**, with two Optionals
and a Nit, all adopted:

- **Optional (adopted)**: add a zoom-preservation test — `resize` clones and
  only mutates a ratio, so `zoomed` should be preserved on both the changed and
  no-op paths.
- **Optional (adopted)**: make `resize_in_place` trap on a non-split
  (`Node::Leaf(_) => unreachable!(...)`) rather than silently doing nothing — an
  invariant check matching upstream's trap-on-wrong-union-field behavior
  (`find_parent_split` only returns split handles).
- **Nit (fixed)**: reworded the zoom note — `resize` itself **preserves**
  `zoomed` (the clone keeps it, `resizeInPlace` only changes a ratio); any
  unzooming happens outside this function.

Codex confirmed the parent-split backtracking matches upstream exactly
(including the root case returning `Backtrack` and thus an unchanged clone), and
that the scale computation, the grid-relative delta conversion, the finite/range
assertions, the zero-scale no-op, and the finite clamp are faithful — and that
the test math for `scale 1.0` and nested `scale 0.5` is sound.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d584-prompt.md`
- Result: `logs/codex-review/20260604-d584-last-message.md`

## Result

**Result:** Pass

`terminal::split_tree` gained `SplitTree::resize`, `resize_in_place`, and
`find_parent_split`. `resize` asserts a finite `ratio` in `[-1, 1]`, fast-paths
the empty tree, clones, finds the nearest layout-matching ancestor split of
`from` (returning the unchanged clone if there is none — including when `from`
is the root), computes the split's grid-relative `scale` from the clone's
`spatial` slots, converts the grid delta to a split delta (`ratio / scale`),
adds it to the split's ratio, clamps to `[0, 1]`, and writes it (preserving
`zoomed`). `resize_in_place` traps on a non-split handle. The module doc comment
was updated — only the formatters remain.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3226 passed, 0 failed (seven new tests; no
  regressions, up from 3219).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + terminal/split_tree.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

The seven new tests: empty resize → empty, a basic `scale 1` resize (`+0.25` →
root `0.75`), a layout mismatch and a from-root resize both no-ops, clamping
(`+1.0` → `1.0`, `-1.0` → `0.0`), a nested `scale 0.5` (a `0.125` grid delta → a
`0.25` split delta, inner `0.5 → 0.75`), and zoom preservation on both the
changed and the no-op paths.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here). Codex confirmed `find_parent_split`
matches upstream's backtracking search (including the root / no-parent case),
that `resize` clones first, returns the unchanged clone for the empty / no-match
/ zero-scale paths, computes `scale` from the clone's spatial slots, applies
`ratio / scale`, clamps to `[0, 1]`, and preserves `zoomed`, that
`resize_in_place` now enforces the split-handle invariant, and that the tests
cover the important behavior.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r584-prompt.md` (result)
- Result: `logs/codex-review/20260604-r584-last-message.md` (result)

## Conclusion

This experiment ports `resize` — the twelfth split_tree slice and the **last
tree-shaping operation**. `resize` moves the nearest layout-matching ancestor
split's divider by a grid-relative delta, scaling it to the split's own extent
(via the `Spatial` representation) and clamping to `[0, 1]`. With this, **all of
split_tree's tree-shaping operations are ported** (`split` / `remove` /
`equalize` / `resize`), alongside the vocabulary, payloads, geometry, arena,
structural queries, iterator, navigation, and the `Spatial` container. The only
remaining split_tree pieces are the **formatters** (`formatText` /
`formatDiagram`), which render a tree as a textual diagram. The other remaining
big-ticket subsystem is the terminal **search subsystem** (coupled to `PageList`
/ `Pin` / `Screen` / `Selection` / `PageFormatter`); the dependency-blocked
helpers persist (regex/oniguruma for `Link::oniRegex`, a URI parser for
`os/uri`, the config-directory naming decision for `file_load` / `edit` /
`loadDefaultFiles`).
