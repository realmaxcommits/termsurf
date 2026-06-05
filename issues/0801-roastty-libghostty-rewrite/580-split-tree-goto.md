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

# Experiment 580: split tree previous / next traversal and goto dispatch

## Description

This experiment ports the in-order **`previous` / `next`** leaf traversal of
upstream `datastruct/split_tree.zig` (with its `Backtrack` recursive search),
and the **`goto`** dispatcher that ties navigation together. With the spatial
navigation (`nearest_wrapped`, Experiment 579), `deepest`, and `spatial` already
ported, `goto` can now be completed — finishing the split_tree navigation
surface. It extends `terminal::split_tree`.

## Upstream behavior

`previous` / `next` find the in-order previous / next **view** of a node via a
recursive backtracking search (the design note: split trees are shallow, so the
stack is a safe scratch allocator):

```zig
const Backtrack = union(enum) { deadend, backtrack, result: Node.Handle };

fn previousBacktrack(self, from, current) Backtrack {
    if (from == current) return .backtrack;       // reached the target → backtrack
    return switch (self.nodes[current.idx()]) {
        .leaf => .deadend,                          // a different leaf → dead end
        .split => |s| switch (self.previousBacktrack(from, s.left)) {
            .result => |v| .{ .result = v },
            .backtrack => .backtrack,               // can't see before the left → keep backtracking
            .deadend => switch (self.previousBacktrack(from, s.right)) {
                .result => |v| .{ .result = v },
                .deadend => .deadend,               // not in this split
                .backtrack => .{ .result = self.deepest(.right, s.left) },  // the immediate previous
            },
        },
    };
}
fn previous(self, from) ?Node.Handle {
    return switch (self.previousBacktrack(from, .root)) { .result => |v| v, else => null };
}
```

`nextBacktrack` / `next` are the exact mirror: recurse the **right** child
first, then the left, and the "immediate next" is `deepest(.left, s.right)`.

`goto` dispatches a `Goto` to a result handle:

```zig
pub fn goto(self, alloc, from, to) !?Node.Handle {
    return switch (to) {
        .previous => self.previous(from),
        .next => self.next(from),
        .previous_wrapped => self.previous(from) orelse self.deepest(.right, .root),
        .next_wrapped => self.next(from) orelse self.deepest(.left, .root),
        .spatial => |d| { var sp = try self.spatial(alloc); defer sp.deinit(alloc);
                          break :spatial self.nearestWrapped(sp, from, d); },
    };
}
```

## Rust mapping (`roastty/src/terminal/split_tree.rs`)

A `Backtrack` enum and the two mirrored recursive searches; `goto` reuses the
already-ported `previous` / `next` / `deepest` / `spatial` / `nearest_wrapped`.

```rust
/// The result of the backtracking previous/next search (upstream `Backtrack`).
enum Backtrack {
    Deadend,
    Backtrack,
    Result(Handle),
}

impl<V> SplitTree<V> {
    /// The in-order previous view of `from`, or `None` if it is the first (upstream `previous`).
    fn previous(&self, from: Handle) -> Option<Handle> {
        match self.previous_backtrack(from, Handle::ROOT) {
            Backtrack::Result(v) => Some(v),
            Backtrack::Backtrack | Backtrack::Deadend => None,
        }
    }

    /// The in-order next view of `from`, or `None` if it is the last (upstream `next`).
    fn next(&self, from: Handle) -> Option<Handle> {
        match self.next_backtrack(from, Handle::ROOT) {
            Backtrack::Result(v) => Some(v),
            Backtrack::Backtrack | Backtrack::Deadend => None,
        }
    }

    fn previous_backtrack(&self, from: Handle, current: Handle) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }
        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(s) => {
                let s = *s;
                match self.previous_backtrack(from, s.left) {
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Backtrack => Backtrack::Backtrack,
                    Backtrack::Deadend => match self.previous_backtrack(from, s.right) {
                        Backtrack::Result(v) => Backtrack::Result(v),
                        Backtrack::Deadend => Backtrack::Deadend,
                        Backtrack::Backtrack => Backtrack::Result(self.deepest(Side::Right, s.left)),
                    },
                }
            }
        }
    }

    fn next_backtrack(&self, from: Handle, current: Handle) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }
        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(s) => {
                let s = *s;
                match self.next_backtrack(from, s.right) {
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Backtrack => Backtrack::Backtrack,
                    Backtrack::Deadend => match self.next_backtrack(from, s.left) {
                        Backtrack::Result(v) => Backtrack::Result(v),
                        Backtrack::Deadend => Backtrack::Deadend,
                        Backtrack::Backtrack => Backtrack::Result(self.deepest(Side::Left, s.right)),
                    },
                }
            }
        }
    }

    /// Resolve a `Goto` to a target handle, or `None` (upstream `goto`).
    pub(crate) fn goto(&self, from: Handle, to: Goto) -> Option<Handle> {
        match to {
            Goto::Previous => self.previous(from),
            Goto::Next => self.next(from),
            Goto::PreviousWrapped => self
                .previous(from)
                .or_else(|| Some(self.deepest(Side::Right, Handle::ROOT))),
            Goto::NextWrapped => self
                .next(from)
                .or_else(|| Some(self.deepest(Side::Left, Handle::ROOT))),
            Goto::Spatial(d) => {
                let sp = self.spatial();
                self.nearest_wrapped(&sp, from, d)
            }
        }
    }
}
```

## Scope / faithfulness notes

- **Ported**: `Backtrack`, `previousBacktrack` / `nextBacktrack`, `previous` /
  `next`, and `goto` → the same in `terminal::split_tree`.
- **Faithful**: the recursive backtracking search (target ⇒ `Backtrack`; a
  different leaf ⇒ `Deadend`; a split tries the near child then the far child,
  with a far-child `Backtrack` resolving to `deepest` of the near child);
  `previous` recurses left-then-right with `deepest(Right, left)`, `next` the
  exact mirror (right-then-left, `deepest(Left, right)`); and `goto`'s five-way
  dispatch (`previous` / `next`, the `_wrapped` fallbacks to `deepest` at the
  root, and `spatial` → `nearest_wrapped`) are reproduced exactly.
- **Faithful adaptation**: the `Split` is copied out of the borrow
  (`let s = *s;`) before recursing (`Split` is `Copy`). `goto` takes no
  allocator (`spatial` owns its `Vec`); the `orelse deepest` becomes
  `.or_else(|| Some(self.deepest(...)))`. `goto` is `pub(crate)` (the public
  navigation entry point); `previous` / `next` / the backtrack helpers stay
  private.
- **Deferred**: the tree-shaping operations (`split` / `remove` / `equalize` /
  `resize`) and the formatters — the only remaining split_tree pieces.
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `terminal::split_tree`.

## Changes

1. `roastty/src/terminal/split_tree.rs`: add the `Backtrack` enum, `previous` /
   `next` / `previous_backtrack` / `next_backtrack`, and `goto`.
2. Tests (in `split_tree.rs`), using the 2×2 grid (in-order leaves
   `TL, BL, TR, BR` = handles `2, 3, 5, 6`):
   - **next traversal**: `next(2) == Some(3)`, `next(3) == Some(5)`,
     `next(5) == Some(6)`, `next(6) == None`.
   - **previous traversal**: `previous(6) == Some(5)`, …, `previous(2) == None`
     (the mirror).
   - **goto previous/next**: `goto(2, Next) == Some(3)`;
     `goto(6, Next) == None`.
   - **goto wrapped**: `goto(6, NextWrapped) == Some(2)` (wraps to
     `deepest(Left, root)`); `goto(2, PreviousWrapped) == Some(6)` (wraps to
     `deepest(Right, root)`).
   - **goto spatial**: `goto(2, Spatial(SpatialDirection::Right)) == Some(5)`
     (the `nearest_wrapped` result).
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

- `previous` / `next` reproduce the in-order backtracking traversal and `goto`
  dispatches all five `Goto` variants (with the wrapped `deepest` fallbacks and
  the spatial `nearest_wrapped`) — faithful to `datastruct/split_tree.zig`;
- the tests pass (next / previous traversal / goto previous-next / wrapped /
  spatial), and the existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the backtracking traversal, the `previous` / `next`
direction, or the `goto` dispatch diverges from upstream, an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and **approved it with no findings**. It confirmed the
backtracking design matches upstream exactly (`previous_backtrack` searches left
then right and resolves a right-side backtrack to `deepest(Right, left)`, while
`next_backtrack` mirrors it with right then left and `deepest(Left, right)`;
`previous` / `next` expose only `Result` as `Some`), that `goto` matches
upstream (direct previous/next, the wrapped fallbacks through root `deepest`,
and spatial navigation via `spatial()` + `nearest_wrapped`, with dropping the
allocator being the right Rust adaptation since `Spatial` owns its `Vec`), and
**verified the 2×2 in-order trace** (for
`root = H(left = V(2,3), right = V(5,6))` the leaves are `2, 3, 5, 6`;
`NextWrapped(6) = 2`, `PreviousWrapped(2) = 6`, and spatial-right from `2`
resolves to `5`).

Review artifacts:

- Prompt: `logs/codex-review/20260604-d580-prompt.md`
- Result: `logs/codex-review/20260604-d580-last-message.md`

## Result

**Result:** Pass

`terminal::split_tree` gained the in-order traversal and navigation dispatch:
the `Backtrack` enum, `previous_backtrack` (left-then-right, a right-side
backtrack resolving to `deepest(Right, left)`) and `next_backtrack` (the
mirror), `previous` / `next` (mapping only `Result` to `Some`), and `goto`
(`pub(crate)`; the five-way dispatch — direct `previous` / `next`, the
`_wrapped` fallbacks to root `deepest`, and `Spatial` → `spatial()` +
`nearest_wrapped`). The module doc comment was updated to mark the navigation
complete.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3199 passed, 0 failed (five new tests; no
  regressions, up from 3196).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + terminal/split_tree.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

The five new tests (over the 2×2 grid, in-order leaves `2, 3, 5, 6`): the `next`
forward traversal (`2 → 3 → 5 → 6 → None`), the `previous` mirror
(`6 → 5 → 3 → 2 → None`), `goto` direct (`goto(2, Next) == Some(3)`,
`goto(6, Next) == None`), `goto` wrapped (`NextWrapped(6) == 2`,
`PreviousWrapped(2) == 6`), and `goto` spatial (`Spatial(Right)` from `TL` →
`TR`, `Spatial(Down)` → `BL`).

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here). Codex confirmed the implementation
matches upstream — `previous_backtrack` searches left then right and resolves
the right-side backtrack through `deepest(Right, left)`, `next_backtrack`
mirrors it (right then left, `deepest(Left, right)`), `previous` / `next` map
only `Result` to `Some`, and `goto` correctly dispatches the direct, wrapped,
and spatial variants — and that the 2×2 traversal tests are sound.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r580-prompt.md` (result)
- Result: `logs/codex-review/20260604-r580-last-message.md` (result)

## Conclusion

This experiment ports the split_tree in-order `previous` / `next` backtracking
traversal and the `goto` dispatch — the eighth split_tree slice — **completing
the split_tree navigation surface**. With this, split_tree's read-side is fully
ported: the vocabulary, the `Split` / `Slot` payloads, the spatial geometry, the
`Node<V>` arena and structural queries, the iterator / zoom / `Goto`, the
`Spatial` container, the spatial `nearest` / `nearest_wrapped`, the in-order
`previous` / `next`, and `goto`. The only remaining split_tree pieces are the
**tree-shaping operations** (`split` / `remove` / `equalize` / `resize` — arena
rewrites that _build_ new trees, where `split` reuses the already-ported
`Direction::split_layout` and `Handle::offset`) and the **formatters**
(`formatText` / `formatDiagram`). The other remaining big-ticket subsystem is
the terminal **search subsystem** (coupled to `PageList` / `Pin` / `Screen` /
`Selection` / `PageFormatter`); the dependency-blocked helpers persist
(regex/oniguruma for `Link::oniRegex`, a URI parser for `os/uri`, the
config-directory naming decision for `file_load` / `edit` / `loadDefaultFiles`).
