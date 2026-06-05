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

# Experiment 601: search reload_active support accessors (next_node_ptr, no_scrollback)

## Description

The next big `ScreenSearch` slice is `reload_active` (the construction/re-search
core). Two small supporting accessors it needs are self-contained and testable
ahead of it: `PageList::next_node_ptr` (walk pages **forward** — newer — the
symmetric counterpart to `prev_node_ptr` from Experiment 593, used by
`reload_active` to walk from a prior history start to the new one) and
`Screen::no_scrollback` (whether the screen has no scrollback — `reload_active`
special-cases it). This experiment ports those two accessors. It extends
`terminal::page_list` and `terminal::screen`.

## Upstream behavior

`reload_active` walks forward through history pages with `node.next`:

```zig
while (true) {
    _ = try window.append(history.start_pin.node);
    if (history.start_pin.node == history_node) break;
    const next = history.start_pin.node.next orelse break;  // the newer node
    history.start_pin.node = next;
}
```

and special-cases the no-scrollback screen:

```zig
if (self.screen.no_scrollback and self.active_results.items.len > 0) active_prune: { ... }
```

- `node.next` is the next (newer) page node in the intrusive list, or `null` for
  the newest.
- `screen.no_scrollback` is true when the screen keeps no scrollback (a discrete
  special case in the `PageList` scrollback model).

## Rust mapping

`next_node_ptr` mirrors `prev_node_ptr` (which returns `pages[idx - 1]`): it
returns `pages[idx + 1]` (the newer node), or `None` if `node` is the newest or
not in the list. `no_scrollback` delegates to the existing
`PageList::scrollback_disabled` (`explicit_max_size == 0`).

```rust
// page_list.rs
impl PageList {
    /// The page node immediately newer than `node` (upstream `node.next`); `None` if `node` is the
    /// newest page or not in this list. The forward counterpart to `prev_node_ptr`.
    pub(in crate::terminal) fn next_node_ptr(&self, node: NonNull<Node>) -> Option<NonNull<Node>> {
        let idx = self.node_index(node)?;
        self.pages.get(idx + 1).map(|p| NonNull::from(p.as_ref()))
    }
}

// screen.rs
impl Screen {
    /// Whether this screen keeps no scrollback (upstream `screen.no_scrollback`). The search
    /// special-cases this in `reload_active`.
    pub(in crate::terminal) fn no_scrollback(&self) -> bool {
        self.pages.scrollback_disabled()
    }
}
```

## Scope / faithfulness notes

- **Ported**: `node.next` (the navigation) → `PageList::next_node_ptr`;
  `screen.no_scrollback` → `Screen::no_scrollback`.
- **Faithful**: `next_node_ptr` returns the newer node or `None` at the newest /
  for an unknown node (mirroring `prev_node_ptr`'s oldest/`None` behavior);
  `no_scrollback` reflects the screen's scrollback-disabled state.
- **Faithful adaptation**: the intrusive `node.next` becomes an index lookup
  (`pages[node_index + 1]`) over the `Vec<Box<Node>>`, consistent with
  `prev_node_ptr`; `no_scrollback` delegates to the existing
  `scrollback_disabled` (`explicit_max_size == 0`).
- **Deferred**: `reload_active` (which consumes these), `init`, the `select`
  dispatcher, `feed` / `search_all`; plus `ViewportSearch` and the search
  `Thread`. (`reload_active` also needs `Pin::before` and a `top_left`-pin
  accessor for its no-scrollback pruning branch — those land with it.)
- No C ABI/header/ABI-inventory change (internal Rust). Adds one `PageList`
  accessor and one `Screen` accessor.

## Changes

1. `roastty/src/terminal/page_list.rs`: add `PageList::next_node_ptr`.
2. `roastty/src/terminal/screen.rs`: add `Screen::no_scrollback`.
3. Tests:
   - **`next_node_ptr`** (in `page_list.rs`): a two-page list (via
     `grow_to_two_pages_for_tests`) → `next_node_ptr(first) == Some(last)`,
     `next_node_ptr(last) == None`, `next_node_ptr(dangling) == None`, and the
     symmetry `prev_node_ptr(last) == Some(first)` (the ordering invariant in
     one place). (A single-page list → `next_node_ptr(only) == None`.)
   - **`no_scrollback`** (in `screen.rs`): `Screen::init(10, 10, Some(0))` →
     `no_scrollback()` is `true`; `Screen::init(10, 10, None)` → `false`.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty terminal::page_list
cargo test -p roastty terminal::screen
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/terminal/page_list.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `next_node_ptr` returns the newer node (or `None` at the newest / unknown) and
  `no_scrollback` reflects the scrollback-disabled state — faithful to the
  `node.next` / `screen.no_scrollback` usage in `terminal/search/screen.zig`;
- the tests pass (`next_node_ptr` / `no_scrollback`), and the existing tests
  still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if `next_node_ptr`'s ordering/bounds or
`no_scrollback`'s meaning diverges from upstream, an unrelated item changes, or
any public C API/ABI changes.

## Design Review

Codex reviewed the design and **approved it**, confirming the mapping is
faithful: `PageList` stores pages oldest-to-newest, and the existing
`prev_node_ptr` uses `idx - 1` for the older node, so `next_node_ptr` using
`idx + 1` is the correct equivalent of upstream's `node.next` (the newer node),
returning `None` for the newest or an unknown node (matching the intrusive-list
behavior); and `Screen::no_scrollback` delegating to
`PageList::scrollback_disabled` (`explicit_max_size == 0`) is the right local
equivalent (`Screen::init(..., Some(0))` produces `explicit_max_size == 0`,
while `None` does not). One Optional, adopted:

- **Optional (adopted)**: add a symmetry assertion in the two-page test —
  `prev_node_ptr(last) == Some(first)` alongside
  `next_node_ptr(first) == Some(last)` — making the ordering invariant explicit
  in one place.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d601-prompt.md`
- Result: `logs/codex-review/20260604-d601-last-message.md`
