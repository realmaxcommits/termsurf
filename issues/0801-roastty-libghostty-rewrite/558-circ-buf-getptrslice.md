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

# Experiment 558: CircBuf two-span access (getPtrSlice + appendSlice)

## Description

Continuing the `CircBuf` port (Experiments 556–557 did the core ring and
iterator), this experiment ports the **two-span mutable accessor** `getPtrSlice`
(with its private `advance` and `storageOffset` helpers) and
`appendSliceAssumeCapacity`, which is built on it.
`getPtrSlice(offset, slice_len)` returns the (up to) two contiguous storage
spans covering a logical range — and, if the range extends past the current
length, it "claims" that space by advancing `head`. It's the primitive
bulk-write/read accessor the search subsystem (and `appendSliceAssumeCapacity`)
relies on. Only auto-growing (`resize` / `ensureUnusedCapacity`) then remains.

## Upstream behavior

`datastruct/circ_buf.zig`:

```zig
pub fn getPtrSlice(self: *Self, offset: usize, slice_len: usize) [2][]T {
    if (slice_len == 0) return .{ &.{}, &.{} };
    assert(offset + slice_len <= self.capacity());
    const end_offset = offset + slice_len;
    if (end_offset > self.len()) self.advance(end_offset - self.len());  // claim space
    const start_idx = self.storageOffset(offset);
    const end_idx = self.storageOffset(end_offset - 1);
    if (end_idx >= start_idx) return .{ self.storage[start_idx .. end_idx + 1], self.storage[0..0] };
    return .{ self.storage[start_idx..], self.storage[0 .. end_idx + 1] };
}

fn advance(self: *Self, amount: usize) void {
    assert(amount <= self.storage.len - self.len());
    self.head += amount;
    if (self.head >= self.storage.len) self.head = self.head - self.storage.len;
    if (self.full) self.tail = self.head;
    self.full = self.head == self.tail;
}

fn storageOffset(self: Self, offset: usize) usize {
    assert(offset < self.storage.len);
    const fits_offset = self.tail + offset;
    if (fits_offset < self.storage.len) return fits_offset;
    return fits_offset - self.storage.len;
}

pub fn appendSliceAssumeCapacity(self: *Self, slice: []const T) void {
    const storage = self.getPtrSlice(self.len(), slice.len);
    fastmem.copy(T, storage[0], slice[0..storage[0].len]);
    fastmem.copy(T, storage[1], slice[storage[0].len..]);
}
```

- `getPtrSlice`: empty range ⇒ two empty spans; else assert it fits in capacity;
  if the range ends beyond `len`, `advance` to claim it; map `offset` /
  `end_offset - 1` to storage indices via `storageOffset`; if they don't wrap
  (`end_idx >= start_idx`) return one span (and an empty second), else two spans
  (`storage[start_idx..]`, `storage[0..=end_idx]`).
- `advance(amount)`: assert it fits the free space; move `head` (single wrap);
  keep `tail` pinned to `head` while `full`; recompute `full`.
- `storageOffset(offset)`: `(tail + offset)` wrapped into `[0, capacity)`.
- `appendSliceAssumeCapacity`: `getPtrSlice(len, slice.len)` (appends at the
  end, claiming the space), then copy the slice across the two spans.

## Rust mapping (`roastty/src/terminal/circ_buf.rs`)

`get_ptr_slice` returns `(&mut [T], &mut [T])` — two **disjoint** mutable spans
of `storage` (the wrap case uses `split_at_mut` to get them safely):

```rust
/// The (up to two) contiguous storage spans covering the logical range `[offset,
/// offset+slice_len)` (upstream `getPtrSlice`). If the range extends past the current length,
/// the space is claimed by advancing `head`. The second span is empty when the range does not
/// wrap.
pub(crate) fn get_ptr_slice(&mut self, offset: usize, slice_len: usize) -> (&mut [T], &mut [T]) {
    if slice_len == 0 {
        // Two disjoint empty spans (deriving them from a split avoids two `&mut []` literals,
        // which would be two mutable borrows of the same promoted empty array).
        let (a, b) = self.storage.split_at_mut(0);
        return (a, &mut b[..0]);
    }
    assert!(offset + slice_len <= self.capacity());
    let end_offset = offset + slice_len;
    let cur_len = self.len();
    if end_offset > cur_len {
        self.advance(end_offset - cur_len);
    }
    let start_idx = self.storage_offset(offset);
    let end_idx = self.storage_offset(end_offset - 1);
    if end_idx >= start_idx {
        // Non-wrap: one span `storage[start_idx..=end_idx]`, empty second. Split at `end_idx +
        // 1` so the second (empty) span is a disjoint sub-slice, not a `&mut []` literal.
        let (front, back) = self.storage.split_at_mut(end_idx + 1);
        (&mut front[start_idx..], &mut back[..0])
    } else {
        // Wrap: span0 = storage[start_idx..], span1 = storage[0..=end_idx]; disjoint because
        // end_idx < start_idx. `split_at_mut(start_idx)` yields the two disjoint halves.
        let (left, right) = self.storage.split_at_mut(start_idx);
        (right, &mut left[..=end_idx])
    }
}

/// Advance `head` by `amount`, claiming free space (upstream `advance`).
fn advance(&mut self, amount: usize) {
    assert!(amount <= self.storage.len() - self.len());
    self.head += amount;
    if self.head >= self.storage.len() {
        self.head -= self.storage.len();
    }
    if self.full {
        self.tail = self.head;
    }
    self.full = self.head == self.tail;
}

/// Map a logical offset (from the oldest) to a storage index (upstream `storageOffset`).
fn storage_offset(&self, offset: usize) -> usize {
    assert!(offset < self.storage.len());
    let fits = self.tail + offset;
    if fits < self.storage.len() {
        fits
    } else {
        fits - self.storage.len()
    }
}

/// Append a slice, assuming there is capacity (upstream `appendSliceAssumeCapacity`).
pub(crate) fn append_slice_assume_capacity(&mut self, slice: &[T]) {
    let len = self.len();
    let (span0, span1) = self.get_ptr_slice(len, slice.len());
    let first = span0.len();
    span0.copy_from_slice(&slice[..first]);
    span1.copy_from_slice(&slice[first..]);
}
```

The wrap case mirrors upstream's `[storage[start_idx..], storage[0..=end_idx]]`:
the two spans' lengths sum to `slice_len`
(`(capacity - start_idx) + (end_idx + 1)`). `split_at_mut` gives the two
**disjoint** `&mut` halves safely (no `unsafe`). Every case derives both spans
from a `split_at_mut` of `self.storage`, so even an empty second span is a
disjoint sub-slice (never a `&mut []` literal — which would be two mutable
borrows of the same promoted empty array). `append_slice_assume_capacity` calls
`get_ptr_slice(len, slice.len)` — which advances `head` to claim the new space —
then copies the slice across the (possibly two) spans
(`span0.len() + span1.len() == slice.len()`).

## Scope / faithfulness notes

- **Ported (bridged)**: `getPtrSlice` → `get_ptr_slice`; `advance` /
  `storageOffset` → `advance` / `storage_offset`; `appendSliceAssumeCapacity` →
  `append_slice_assume_capacity`.
- **Faithful**: the empty-range fast path; the capacity assert; the
  `advance`-to-claim when the range ends past `len`; the `storageOffset`
  translation; the one-span (non-wrap) vs two-span (wrap) result; `advance`'s
  head move / `tail` pinning / `full` recompute; the slice copy across the
  spans.
- **Faithful adaptation**: `[2][]T` → `(&mut [T], &mut [T])` (each case via
  `split_at_mut` for two disjoint `&mut` halves — no `unsafe`, and no `&mut []`
  literal — the safe equivalent of Zig's two slice fat-pointers); `fastmem.copy`
  → `copy_from_slice` (`T: Copy`).
- **Deferred**: auto-growing (`resize` / `ensureUnusedCapacity`) — the last
  remaining CircBuf slice.
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `terminal::circ_buf`.

## Changes

1. `roastty/src/terminal/circ_buf.rs`: add `get_ptr_slice`, `advance`,
   `storage_offset`, `append_slice_assume_capacity`.
2. Tests (in `circ_buf.rs`):
   - **get_ptr_slice non-wrap**: a `[1, 2, 3]` buffer (capacity 5);
     `get_ptr_slice(0, 3)` yields span0 `[1, 2, 3]` and an empty span1 (no
     `advance`, since `end_offset == len`).
   - **get_ptr_slice wrap**: a wrapped buffer (`[4, 2, 3]` with `tail = 1` after
     a `delete_oldest` + wrapping `append`); `get_ptr_slice(0, 3)` yields span0
     `[2, 3]` and span1 `[4]` (logical order `2, 3, 4`).
   - **get_ptr_slice claims space** (Codex design review): on an empty
     `new(4, 0)`, `get_ptr_slice(0, 4)` advances `head` so `len()` becomes `4`
     (full) and exposes a writable span; writing `[10, 20, 30, 40]` into it
     makes those iterator-visible (the distinctive side effect, not just reading
     existing spans).
   - **append_slice non-wrap**: `new(5, 0)`, `append(1)`, then
     `append_slice_assume_capacity(&[2, 3, 4])` ⇒ the iterator yields
     `1, 2, 3, 4`.
   - **append_slice across the wrap**: a buffer positioned so the appended slice
     wraps (e.g. `new(4, 0)`, fill+`delete_oldest` to leave one element near the
     end, then `append_slice_assume_capacity` of 3) ⇒ the iterator yields the
     existing element followed by the three appended, in order.
   - **append_slice empty**: `append_slice_assume_capacity(&[])` is a no-op
     (length unchanged).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty circ_buf
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/terminal/circ_buf.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `get_ptr_slice` returns the correct one- or two-span storage view of a logical
  range, advancing `head` to claim space when the range ends past `len`;
  `append_slice_assume_capacity` appends a slice across the (possibly wrapping)
  spans — faithful to `datastruct/circ_buf.zig`;
- the tests pass (non-wrap / wrap span access + append-slice non-wrap / wrap /
  empty), and the existing tests still pass;
- the resize/grow pieces stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the span computation, the `advance` side effect, or
the slice copy diverges from upstream, an unrelated item changes, or any public
C API/ABI changes.

## Design Review

Codex's first design review raised **one Required** finding (and an Optional),
both now addressed; the corrected design was **re-reviewed and approved with no
findings**.

- **`&mut []` empty spans (Required, fixed)**: the design returned
  `(&mut [], &mut [])` (and a `&mut []` non-wrap second span), which is two
  mutable borrows of the same promoted empty array — unsound/rejected. Fixed so
  **every** case derives both spans from a `split_at_mut` of `self.storage`
  (empty case `split_at_mut(0)`; non-wrap `split_at_mut(end_idx + 1)`), so even
  empty spans are disjoint sub-slices.
- **(Optional, adopted)**: a claim-space test was added — `get_ptr_slice(0, 4)`
  on an empty `new(4, 0)` advances `head` (so `len()` becomes 4 / full) and
  exposes a writable span whose writes become iterator-visible.

On re-review Codex confirmed the revised span construction is sound and faithful
(all returned slices come from `split_at_mut`; the non-wrap case maps
`front[start_idx..]` to `storage[start_idx..=end_idx]` with the empty second
span from `back[..0]`), and the claim-space test covers the key side effect.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d558-prompt.md` (design),
  `logs/codex-review/20260604-d558b-prompt.md` (design re-review)
- Result: `logs/codex-review/20260604-d558-last-message.md` (design),
  `logs/codex-review/20260604-d558b-last-message.md` (design re-review)
