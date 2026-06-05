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

# Experiment 583: split tree equalize (rebalance split ratios by leaf weight)

## Description

This experiment ports `equalize` (and its helper `weight`) from upstream
`datastruct/split_tree.zig` — the first of the two `f16`-ratio rebalancers.
`equalize` returns a new tree whose every split's `ratio` is set from the
relative **leaf weight** of its two children (so panes end up evenly sized). It
extends `terminal::split_tree`.

## Upstream behavior

```zig
pub fn equalize(self, gpa) !Self {
    if (self.isEmpty()) return .empty;
    const nodes = try alloc.dupe(Node, self.nodes);          // clone the nodes
    for (nodes) |*node| switch (node.*) {
        .leaf => {},
        .split => |*s| {
            const wl = self.weight(s.left, s.layout, 0);
            const wr = self.weight(s.right, s.layout, 0);
            assert(wl > 0 and wr > 0);
            s.ratio = @as(f16, @floatFromInt(wl)) / @as(f16, @floatFromInt(wl + wr));
        },
    };
    try refNodes(gpa, nodes);
    return .{ .arena = arena, .nodes = nodes, .zoomed = self.zoomed };  // zoom preserved
}

fn weight(self, from, layout, acc) usize {
    return switch (self.nodes[from.idx()]) {
        .leaf => acc + 1,
        .split => |s| if (s.layout == layout) self.weight(s.left, layout, acc)
                                            + self.weight(s.right, layout, acc)
                      else 1,                                // a differently-laid split = weight 1
    };
}
```

`weight(from, layout)` counts the leaves reachable from `from` **through splits
of the same layout**: a leaf is `1`, a same-layout split sums its children, and
a _different_-layout split counts as `1` (its internal arrangement is along the
other axis, so it's one "cell" along this axis). Each split's new `ratio` is
then `weight_left / (weight_left + weight_right)` — the fraction of cells on the
left/top side. `equalize` preserves the structure and the zoom; only the ratios
change. The `acc` parameter of `weight` is vestigial (always `0`, passed through
unchanged).

## Rust mapping (`roastty/src/terminal/split_tree.rs`)

The new node `Vec` is cloned (which refs each view); each split's ratio is
recomputed via `weight` over `self`'s (structurally identical) nodes. The
vestigial `acc` is dropped.

```rust
impl<V> SplitTree<V> {
    /// Return a new tree with every split's `ratio` rebalanced to the relative leaf weight of its
    /// children, so panes are evenly sized (upstream `equalize`). Structure and zoom are preserved.
    pub(crate) fn equalize(&self) -> SplitTree<V> {
        if self.is_empty() {
            return SplitTree::empty();
        }

        // Clone the nodes (refs each view); recompute each split's ratio.
        let mut nodes: Vec<Node<V>> = self.nodes.iter().cloned().collect();
        for node in &mut nodes {
            if let Node::Split(s) = node {
                let weight_left = self.weight(s.left, s.layout);
                let weight_right = self.weight(s.right, s.layout);
                assert!(weight_left > 0 && weight_right > 0);
                let total = f16::from_f32((weight_left + weight_right) as f32);
                s.ratio = f16::from_f32(weight_left as f32) / total;
            }
        }

        SplitTree {
            nodes,
            zoomed: self.zoomed,
        }
    }

    /// The number of leaves reachable from `from` through splits of `layout` (upstream `weight`,
    /// without the vestigial `acc`). A leaf is `1`; a same-layout split sums its children; a
    /// different-layout split counts as `1`.
    fn weight(&self, from: Handle, layout: Layout) -> usize {
        match &self.nodes[from.idx()] {
            Node::Leaf(_) => 1,
            Node::Split(s) => {
                if s.layout == layout {
                    self.weight(s.left, layout) + self.weight(s.right, layout)
                } else {
                    1
                }
            }
        }
    }
}
```

## Scope / faithfulness notes

- **Ported**: `equalize` / `weight` → `SplitTree::equalize` / `weight`.
- **Faithful**: the empty-tree → empty case; cloning the nodes (preserving
  structure); recomputing each split's `ratio` as
  `weight_left / (weight_left + weight_right)` in `f16`; `weight`'s same-layout
  leaf counting (leaf = 1, same-layout split = children sum, different-layout
  split = 1); the `weight > 0` assertions; and preserving `zoomed` are all
  reproduced.
- **Faithful adaptation**: `weight` is computed over `self`'s nodes (the cloned
  new nodes are structurally identical — only ratios change — so `self`'s
  structure gives the same weights, as upstream's `self.weight(s.left, …)`
  does); `@floatFromInt` becomes `f16::from_f32(_ as f32)` (exact for the small
  integer weights); the `f16` division reproduces upstream's; `Rc::clone` at
  copy time _is_ the deferred `refNodes`; the vestigial `acc` is dropped.
- **Deferred**: `resize` (the other `f16`-ratio rebalancer) and the formatters.
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `terminal::split_tree`.

## Changes

1. `roastty/src/terminal/split_tree.rs`: add `SplitTree::equalize` and `weight`,
   and update the module doc comment to move `equalize` into the landed
   tree-shaping set (leaving only `resize` + the formatters deferred).
2. Tests (in `split_tree.rs`):
   - **empty / single leaf**: `equalize` of the empty tree is empty; of a single
     leaf is unchanged (no splits).
   - **balanced split**: a 2-leaf split with any starting ratio equalizes to
     `0.5` (weights `1, 1`).
   - **unbalanced same-layout tree**: a horizontal split of (a 2-leaf horizontal
     subtree, a single leaf) equalizes the root to `2/3` (weights `2, 1`) and
     the inner split to `0.5`.
   - **different-layout child counts as weight 1**: a vertical split whose left
     child is a _horizontal_ 2-leaf split equalizes the root to `0.5` (weights
     `1, 1`, since the horizontal child is one cell along the vertical axis),
     while the inner horizontal split is `0.5`.
   - **zoom preserved**: `equalize` keeps the `zoomed` handle.
   - **ref-counting**: after `equalize`, each view's `Rc::strong_count` rises by
     one (the returned tree's reference); dropping the equalized tree releases
     those refs.
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

- `equalize` rebalances each split's `ratio` to
  `weight_left / (weight_left + weight_right)` with `weight`'s same-layout leaf
  counting, preserving structure and zoom (and the empty case) — faithful to
  `datastruct/split_tree.zig`;
- the tests pass (empty/single / balanced / unbalanced / different-layout /
  zoom), and the existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the weight counting, the ratio computation, or the
structure/zoom preservation diverges from upstream, an unrelated item changes,
or any public C API/ABI changes.

## Design Review

Codex reviewed the design and found **no Required findings**, with one Optional
and one Nit, both adopted:

- **Optional (adopted)**: add a ref-count test for `equalize` (each view gains
  exactly one `Rc` reference in the returned tree; dropping it releases those
  refs) to protect the manual `Clone` behavior from regressing.
- **Nit (adopted)**: include the module-doc update in the Changes list — moving
  `equalize` into the landed tree-shaping set, leaving only `resize` + the
  formatters deferred.

Codex confirmed everything else is faithful: dropping the vestigial `acc` is
safe, `weight` preserves the same-layout semantics exactly (a different-layout
split counts as one cell), the ratios are `weight_left / (left + right)` in
`f16`, zoom is preserved, and computing weights from `self.nodes` is equivalent
because `equalize` only changes ratios, not structure.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d583-prompt.md`
- Result: `logs/codex-review/20260604-d583-last-message.md`
