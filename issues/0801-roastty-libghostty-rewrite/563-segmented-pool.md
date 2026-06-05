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

# Experiment 563: the SegmentedPool (stable-handle write-request pool)

## Description

This experiment ports upstream `datastruct/segmented_pool.zig` —
`SegmentedPool`, a pool that hands out values which can be "put back" **in
order** and which grows (by doubling) on demand. Upstream built it specifically
for libuv write requests, which must have a **stable pointer** and are processed
in order on a single stream. roastty's fifth `datastruct/` port. It lands at
`terminal::segmented_pool`.

## Upstream behavior

`datastruct/segmented_pool.zig` — `SegmentedPool(T, prealloc)`:

```zig
i: usize = 0,
available: usize = prealloc,
list: std.SegmentedList(T, prealloc) = .{ .len = prealloc },

pub fn get(self) !*T {
    if (self.available == 0) return error.OutOfValues;
    const i = @mod(self.i, self.list.len);
    self.i +%= 1;                 // wrapping increment
    self.available -= 1;
    return self.list.at(i);
}
pub fn getGrow(self, alloc) !*T {
    if (self.available == 0) try self.grow(alloc);
    return try self.get();
}
fn grow(self, alloc) !void {
    try self.list.growCapacity(alloc, self.list.len * 2);
    self.i = self.list.len;       // old len
    self.available = self.list.len;
    self.list.len *= 2;           // new len = 2 * old
}
pub fn put(self) void {
    self.available += 1;
    assert(self.available <= self.list.len);
}
```

- The pool is a **cyclic ring** of `len` slots: `get` hands out slot `i % len`,
  advances the (wrapping) cursor `i`, and decrements `available`; with
  `available == 0`, `get` fails (`OutOfValues`). Because requests are processed
  in order, the slot that comes back around at `i % len` is exactly the one that
  was `put` back.
- `getGrow` grows first if exhausted, then `get`s. `grow` doubles `len`; the
  **new** half becomes the available slots (`i` and `available` are set to the
  old `len`), so the cursor walks the fresh slots first and only wraps back to
  the old ones after they are `put` back.
- `put` just bumps `available` (the caller guarantees in-order return); it does
  not track which slot returned.

The upstream test (`SegmentedPool(u8, 2)`): `get` twice (distinct), the third
`get` is `OutOfValues`; write `42` into the first; `put`, then `get` returns
that same slot (still `42`); exhausted again; `getGrow` returns a slot distinct
from the first two; another `get`, then `OutOfValues`; `put`, then `get` returns
the first slot again.

## Rust mapping (`roastty/src/terminal/segmented_pool.rs`)

Upstream hands out **raw pointers** and needs `std.SegmentedList` purely so
those pointers stay **stable** across growth. roastty hands out **indices**
instead, which are inherently stable across a `Vec` reallocation (index `k` is
still index `k`), so a plain `Vec<T>` is a faithful and simpler backing store.
The cursor / `available` / doubling arithmetic is reproduced exactly.

```rust
//! A pool of values handed out by index, returned in order, that grows on demand (port of
//! upstream `datastruct/segmented_pool`).

/// Returned by `get` when the pool is exhausted (upstream `error.OutOfValues`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OutOfValues;

/// A pool of `T` whose slots are handed out by index and returned in order, growing (by
/// doubling) on demand. `prealloc` is the initial slot count (must be >= 1).
pub(crate) struct SegmentedPool<T> {
    items: Vec<T>,
    i: usize,         // wrapping cursor; the next slot is `i % items.len()`
    available: usize, // slots that can still be handed out before `OutOfValues`
}

impl<T: Default> SegmentedPool<T> {
    /// Create a pool with `prealloc` default-initialized slots (`prealloc` must be >= 1).
    pub(crate) fn new(prealloc: usize) -> Self {
        assert!(prealloc >= 1, "SegmentedPool prealloc must be >= 1");
        Self {
            items: (0..prealloc).map(|_| T::default()).collect(),
            i: 0,
            available: prealloc,
        }
    }

    /// Get the index of the next available slot without growing (upstream `get`).
    pub(crate) fn get(&mut self) -> Result<usize, OutOfValues> {
        if self.available == 0 {
            return Err(OutOfValues);
        }
        let idx = self.i % self.items.len();
        self.i = self.i.wrapping_add(1);
        self.available -= 1;
        Ok(idx)
    }

    /// Get the next available slot index, growing the pool if exhausted (upstream `getGrow`).
    pub(crate) fn get_grow(&mut self) -> usize {
        if self.available == 0 {
            self.grow();
        }
        self.get().expect("grow guarantees availability")
    }

    /// Put a slot back. Caller must return slots in `get` order (upstream `put`).
    ///
    /// Slots are handles, not pointers: a caller stores the index from `get` / `get_grow` and
    /// re-resolves it with `at` / `at_mut`. A borrow from `at_mut` must not be held across a
    /// `get_grow` that grows (safe Rust enforces this naturally), but indices stay valid.
    pub(crate) fn put(&mut self) {
        self.available += 1;
        // Upstream's `inlineAssert` traps in release too, so use `assert!` (not `debug_assert!`):
        // an extra `put` past capacity would corrupt later handout, and this is the only guard on
        // the in-order return contract.
        assert!(self.available <= self.items.len());
    }

    /// Borrow a slot by index (handed out by `get` / `get_grow`).
    pub(crate) fn at(&self, idx: usize) -> &T {
        &self.items[idx]
    }

    /// Mutably borrow a slot by index (handed out by `get` / `get_grow`).
    pub(crate) fn at_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[idx]
    }

    fn grow(&mut self) {
        let old_len = self.items.len();
        self.items
            .extend((0..old_len).map(|_| T::default())); // double the length
        self.i = old_len;
        self.available = old_len;
    }
}
```

## Scope / faithfulness notes

- **Ported (bridged)**: `datastruct.SegmentedPool` →
  `terminal::segmented_pool::SegmentedPool` (`get`, `get_grow`, `put`, plus
  index accessors `at` / `at_mut`).
- **Faithful**: the cyclic-ring slot handout (`i % len`, wrapping cursor,
  `available` count), `OutOfValues` on exhaustion, `getGrow`'s grow-then-get,
  `grow`'s length doubling with `i` / `available` reset to the old length, and
  `put`'s availability bump are all reproduced exactly (same arithmetic, same
  ordering contract).
- **Faithful adaptation**: upstream hands out **raw pointers** and uses
  `std.SegmentedList` only to keep those pointers stable across growth (its
  libuv motivation); roastty hands out **indices** (stable across a `Vec`
  reallocation by construction), so the backing store is a plain `Vec<T>` and
  the accessors `at` / `at_mut` resolve an index to a borrow. The
  caller-provided `alloc` and `deinit` disappear (roastty owns the `Vec`).
- **Constraint**: `prealloc >= 1` (a zero-length ring would divide by zero).
  Upstream's tests use `prealloc = 2`; the type is only ever instantiated with a
  positive prealloc. `new` asserts it.
- **Bound**: `T: Default` initializes the prealloc/grown slots (upstream leaves
  the `SegmentedList` entries uninitialized for the caller to fill; roastty
  initializes them, which is observably identical since the caller always writes
  before reading).
- No C ABI/header/ABI-inventory change (internal Rust). Adds
  `terminal::segmented_pool`.

## Changes

1. `roastty/src/terminal/segmented_pool.rs` (new): `OutOfValues`,
   `SegmentedPool<T>`, with `new` / `get` / `get_grow` / `put` / `at` / `at_mut`
   / `grow` as above.
2. `roastty/src/terminal/mod.rs`: add `#[allow(dead_code)] mod segmented_pool;`
   (alphabetical).
3. Tests (in `segmented_pool.rs`), mirroring the upstream test:
   - **exhaustion**: `new(2)`, two `get`s return distinct indices, the third is
     `Err(OutOfValues)`.
   - **in-order put-back**: write via `at_mut`, `put`, then `get` returns the
     same slot with the written value intact.
   - **grow**: after exhaustion, `get_grow` returns a slot distinct from the
     first two; one more `get`, then `Err(OutOfValues)`; `put`, then `get` wraps
     back to the first slot.
   - **available count / capacity** tracking.
   - **over-`put` panics**: a `put` that would push `available` past the slot
     count panics (`#[should_panic]`), locking down the in-order-return
     invariant.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty terminal::segmented_pool
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/terminal/segmented_pool.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `SegmentedPool` hands out slots cyclically by index (`i % len`, wrapping
  cursor, `available` count), fails with `OutOfValues` when exhausted, grows by
  doubling (resetting `i` / `available` to the old length so the new half is
  handed out first), and `put` returns slots in order — faithful to
  `datastruct/segmented_pool.zig`;
- the tests pass (exhaustion / in-order put-back / grow / counts), and the
  existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the slot-handout order, the grow arithmetic, the
`OutOfValues` condition, or the in-order put-back contract diverges from
upstream, an unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and found **one Required** finding plus two Optionals,
all addressed:

- **Required (fixed)**: `put` must use `assert!`, not `debug_assert!`, for the
  `available <= items.len()` invariant — upstream's `inlineAssert` traps in
  release too, so a `debug_assert!` would let an extra `put` in release silently
  push `available` past capacity and corrupt later handout. Changed to
  `assert!`.
- **Optional (adopted)**: added an over-`put` `#[should_panic]` test to lock the
  invariant down (the only guard on the in-order-return contract).
- **Optional (adopted)**: documented the handle contract on `put` — callers
  store the index and re-resolve via `at` / `at_mut`; a borrow from `at_mut`
  must not be held across a growing `get_grow` (safe Rust enforces this), but
  indices stay valid; an FFI consumer needing raw-pointer stability across
  growth would instead need boxed/pinned slots.

Codex confirmed the ring arithmetic is otherwise faithful — it traced the
upstream test against the mapping (after a grow from length 2,
`i = old_len = 2`, `available = 2`, so slots 2 and 3 are handed out first, then
after a `put`, `i % 4` wraps back to slot 0) — and that `T: Default` is an
acceptable Rust initialization adaptation and `prealloc >= 1` a reasonable guard
for the nonzero ring upstream always uses.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d563-prompt.md`
- Result: `logs/codex-review/20260604-d563-last-message.md`

## Result

**Result:** Pass

`terminal::segmented_pool::SegmentedPool<T: Default>` was added: a `Vec<T>`
backing store with a wrapping cursor `i` and an `available` count.
`new(prealloc)` asserts `prealloc >= 1` and default-initializes the slots; `get`
returns `Result<usize, OutOfValues>` (slot `i % len`, wrapping cursor advance,
`available` decrement); `get_grow` grows-then-gets (infallible `usize`); `put`
bumps `available` guarded by a release-trapping `assert!` (the Required fix);
`at` / `at_mut` resolve an index to a borrow; and `grow` doubles the length via
`extend`, resetting `i` and `available` to the old length so the fresh half is
handed out first. Registered via `#[allow(dead_code)] mod segmented_pool;` in
`terminal/mod.rs`.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3122 passed, 0 failed (five new tests; no
  regressions, up from 3117).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + terminal/segmented_pool.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

The five new tests: distinct slots until exhausted, in-order put-back returning
the same slot with its written value intact, growth handing out the fresh half
first (then wrapping back to the first old slot after a `put`),
capacity/availability tracking across a grow, and the over-`put`
`#[should_panic]` guard.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here as part of result recording). Codex
confirmed the implementation matches upstream and the approved design: `get`
uses `i % len` with a wrapping cursor and `available` decrement; `get_grow`
grows first when exhausted; `grow` doubles and hands out the fresh half first by
setting `i` and `available` to `old_len`; `put` uses the release-active
`assert!`; the index-handle adaptation is documented; and the tests cover
exhaustion, in-order put-back with value preservation, growth/wrap behavior,
count tracking, and the over-`put` panic.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r563-prompt.md` (result)
- Result: `logs/codex-review/20260604-r563-last-message.md` (result)

## Conclusion

`terminal::segmented_pool::SegmentedPool` is ported from
`datastruct/segmented_pool.zig` — roastty's fifth `datastruct/` type. The key
adaptation was turning upstream's libuv-motivated **raw-pointer +
`std.SegmentedList`** design (pointer stability across growth) into an
**index-handle + `Vec<T>`** design: indices are stable across a `Vec`
reallocation by construction, so the simpler backing store is faithful, with the
ring arithmetic (cursor / `available` / doubling) reproduced exactly. The
remaining `datastruct/` work is `intrusive_linked_list` (the raw-pointer
doubly-linked list — an arena/index redesign like `Lru`), `blocking_queue`
(channel-like, thread-synchronized), and the large `split_tree`. The terminal
**search subsystem** (now `CircBuf` and the cache/pool datastructs are in place)
is the other natural target. The objc/bundle-id helpers, the `home()` resolver,
and config `loadDefaultFiles` remain deferred pending roastty's naming decision;
`background-image-opacity` stays float-blocked.
