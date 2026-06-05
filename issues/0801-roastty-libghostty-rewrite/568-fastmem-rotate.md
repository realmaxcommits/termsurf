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

# Experiment 568: fastmem single-step slice rotations

## Description

This experiment ports the **rotation helpers** from upstream `fastmem.zig` —
`rotateOnce`, `rotateOnceR`, `rotateIn`, and `rotateInR`, single-step slice
rotations the terminal uses (e.g. in the page list). roastty has no equivalent
helper yet. It lands at the crate root: `crate::fastmem` (mirroring upstream's
`src/fastmem.zig`).

## Upstream behavior

`fastmem.zig` provides two memory primitives and four rotation helpers:

- `move(T, dest, source)` / `copy(T, dest, source)` — thin wrappers that prefer
  libc `memmove` / `memcpy` (when linking libc) over the Zig builtins, purely as
  a speed optimization.
- `rotateOnce(items)` — moves the first item to the end: `0 1 2 3 → 1 2 3 0`.
  (Implemented as a `tmp` save of `items[0]`, a `move` of `items[1..]` down by
  one, then writing `tmp` at the end — the same result as
  `std.mem.rotate(items, 1)` but with one `memmove` instead of three reverses.)
- `rotateOnceR(items)` — moves the last item to the front: `0 1 2 3 → 3 0 1 2`
  (the reverse).
- `rotateIn(items, item) -> T` — rotates `item` in at the **end**, returning the
  displaced first item: rotating `4` into `0 1 2 3` gives `1 2 3 4` and returns
  `0`.
- `rotateInR(items, item) -> T` — rotates `item` in at the **front**, returning
  the displaced last item: rotating `4` into `0 1 2 3` gives `4 0 1 2` and
  returns `3` (the reverse).

All four require a non-empty slice (they index `items[0]` / `items[len-1]`).

## Rust mapping (`roastty/src/fastmem.rs`)

The `move` / `copy` primitives exist only for the libc-vs-builtin speed choice,
which is moot in Rust — `slice::copy_within` and `slice::copy_from_slice`
already lower to the optimized `memmove` / `memcpy` intrinsics. So they are
**not** ported as standalone functions; the four rotation helpers are ported
directly:

All four are fully generic over `T` (matching upstream): `rotate_once` /
`rotate_once_r` are `slice::rotate_left(1)` / `slice::rotate_right(1)`, and
`rotate_in` / `rotate_in_r` displace the boundary element with
`std::mem::replace` and then rotate — no `Copy` bound, no unsafe.

```rust
//! Single-step slice rotations (port of the rotation helpers in upstream `fastmem`).
//!
//! Upstream also wraps libc `memmove` / `memcpy` (`move` / `copy`) purely to prefer them over the
//! Zig builtins for speed. Rust's `slice::copy_within` / `copy_from_slice` already lower to those
//! intrinsics, so only the rotation helpers are ported here.

/// Moves the first item to the end: `0 1 2 3` → `1 2 3 0` (upstream `rotateOnce`). The slice must
/// be non-empty.
pub(crate) fn rotate_once<T>(items: &mut [T]) {
    items.rotate_left(1);
}

/// Moves the last item to the start: `0 1 2 3` → `3 0 1 2` (upstream `rotateOnceR`). The slice must
/// be non-empty.
pub(crate) fn rotate_once_r<T>(items: &mut [T]) {
    items.rotate_right(1);
}

/// Rotates `item` in at the end, returning the displaced first item: rotating `4` into `0 1 2 3`
/// gives `1 2 3 4` and returns `0` (upstream `rotateIn`). The slice must be non-empty.
pub(crate) fn rotate_in<T>(items: &mut [T], item: T) -> T {
    // Put `item` at the front, take the old first out, then rotate it to the end.
    let removed = std::mem::replace(&mut items[0], item);
    items.rotate_left(1);
    removed
}

/// Rotates `item` in at the start, returning the displaced last item: rotating `4` into `0 1 2 3`
/// gives `4 0 1 2` and returns `3` (upstream `rotateInR`). The slice must be non-empty.
pub(crate) fn rotate_in_r<T>(items: &mut [T], item: T) -> T {
    // Put `item` at the back, take the old last out, then rotate it to the front.
    let n = items.len();
    let removed = std::mem::replace(&mut items[n - 1], item);
    items.rotate_right(1);
    removed
}
```

For `rotate_in`, replacing `items[0]` with `item` and rotating left by one is
equivalent to upstream's "save `items[0]`, shift the rest down, write the new
item last": e.g. `[0,1,2,3]` + `4` → replace ⇒ `[4,1,2,3]` (removed `0`) →
`rotate_left(1)` ⇒ `[1,2,3,4]`. Symmetrically for `rotate_in_r`.

## Scope / faithfulness notes

- **Ported**: the four `fastmem` rotation helpers →
  `crate::fastmem::{rotate_once, rotate_once_r, rotate_in, rotate_in_r}`.
- **Faithful**: the exact rotation semantics and the displaced-element return
  values (`rotate_in` returns the old first, `rotate_in_r` the old last),
  matching the upstream doc-comment examples.
- **Faithful adaptation**: upstream's hand-rolled `tmp` + `move` rotation (a
  `memmove`-based optimization of `std.mem.rotate(items, 1)`) becomes Rust's
  `slice::rotate_left(1)` / `rotate_right(1)` — the same result via Rust's
  already-optimized stdlib. `rotate_in` / `rotate_in_r` displace the boundary
  element with `std::mem::replace` and then rotate, so **all four stay fully
  generic over `T`** (matching upstream's generic API) with no `Copy` bound and
  no unsafe.
- **Not ported (subsumed)**: `move` / `copy` — the libc-`memmove` / `memcpy`
  preference is a no-op in Rust, where `copy_within` / `copy_from_slice` already
  use those intrinsics.
- **Precondition**: all four require a non-empty slice, exactly as upstream
  (which indexes `[0]` / `[len-1]`). `rotate_left(1)` / `rotate_right(1)` also
  require a non-empty slice.
- No C ABI/header/ABI-inventory change (internal Rust). Adds a crate-root
  `fastmem` module.

## Changes

1. `roastty/src/fastmem.rs` (new): `rotate_once`, `rotate_once_r`, `rotate_in`,
   `rotate_in_r`.
2. `roastty/src/lib.rs`: add `#[allow(dead_code)] mod fastmem;` (alphabetical,
   before `file_type`).
3. Tests (in `fastmem.rs`), the upstream doc examples plus edge cases:
   - **rotate_once**: `[0, 1, 2, 3] → [1, 2, 3, 0]`.
   - **rotate_once_r**: `[0, 1, 2, 3] → [3, 0, 1, 2]`.
   - **rotate_in**: into `[0, 1, 2, 3]` with `4` → `[1, 2, 3, 4]`, returns `0`.
   - **rotate_in_r**: into `[0, 1, 2, 3]` with `4` → `[4, 0, 1, 2]`, returns
     `3`.
   - **inverses**: `rotate_once` then `rotate_once_r` restores the original (and
     vice versa).
   - **single element**: each is the identity / returns the lone element as
     documented.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty fastmem
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/fastmem.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- the four rotation helpers reproduce upstream's semantics and displaced-element
  returns exactly (matching the doc-comment examples) — faithful to
  `fastmem.zig`;
- the tests pass (the four examples / inverses / single element), and the
  existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if any rotation's result or returned element diverges
from upstream, an unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and found **no Required findings**, with one Optional
— adopted:

- **Optional (adopted)**: keep `rotate_in` / `rotate_in_r` fully generic over
  `T` (matching upstream) instead of narrowing to `T: Copy`. A safe generic
  version uses `std::mem::replace` (to displace the boundary element) plus
  `rotate_left(1)` / `rotate_right(1)`. Adopted, so all four helpers stay
  generic with no `Copy` bound and no unsafe.

Codex confirmed the four rotations' directions, displaced return values, and
shift ranges match upstream's `move(dest, source)` ranges, that not porting the
standalone `move` / `copy` wrappers is acceptable in Rust, and that the proposed
tests cover the important examples and the single-element edge case.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d568-prompt.md`
- Result: `logs/codex-review/20260604-d568-last-message.md`

## Result

**Result:** Pass

`crate::fastmem` was added with four fully-generic single-step slice rotations:
`rotate_once` (`rotate_left(1)`), `rotate_once_r` (`rotate_right(1)`),
`rotate_in` (`mem::replace` at index 0, then `rotate_left(1)`, returning the
displaced first), and `rotate_in_r` (`mem::replace` at the last index, then
`rotate_right(1)`, returning the displaced last). The libc-`memmove` / `memcpy`
wrappers (`move` / `copy`) were not ported — they are a no-op optimization in
Rust. Registered via `#[allow(dead_code)] mod fastmem;` at the crate root in
`lib.rs`. No other files were touched.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3147 passed, 0 failed (seven new tests; no
  regressions, up from 3140).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + fastmem.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

The seven new tests: the four upstream doc examples (`[0,1,2,3]` → `[1,2,3,0]` /
`[3,0,1,2]`; `rotate_in` 4 → `[1,2,3,4]` returns 0; `rotate_in_r` 4 →
`[4,0,1,2]` returns 3), `rotate_once` / `rotate_once_r` as inverses (both
directions), single-element identity / lone-element return, and a non-`Copy`
`String` case validating the generic surface.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here as part of result recording). Codex
confirmed the four rotations match upstream's documented behavior and returned
elements, the generic `replace`-then-`rotate_left/right` implementation is
correct for non-`Copy` values and preserves the non-empty precondition, and the
tests cover the examples, inverses, single-element behavior, and the non-`Copy`
surface.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r568-prompt.md` (result)
- Result: `logs/codex-review/20260604-r568-last-message.md` (result)

## Conclusion

`crate::fastmem`'s four rotation helpers are ported from `fastmem.zig`. The
adaptation was idiomatic: upstream's hand-rolled `memmove`-based rotation (an
optimization of `std.mem.rotate(items, 1)`) becomes Rust's already-optimized
`slice::rotate_left` / `rotate_right`, and the displace-and-return helpers use
`std::mem::replace` so all four stay fully generic over `T` (adopting the
review's Optional rather than narrowing to `Copy`). The `move` / `copy` libc
micro-optimizations are subsumed by Rust's `copy_within` / `copy_from_slice`.
This is the second crate-root (`src/`) leaf port (after `file_type`), and
follows the established pattern of porting small self-contained upstream files
while the big-ticket subsystems (`datastruct/split_tree`, 2517 lines; the
terminal **search subsystem** coupled to `PageList` / `Pin` / `Screen` /
`Selection` / `PageFormatter`) await dedicated multi-experiment efforts. Other
unported leaves include `terminal/csi`, `terminal/ScreenSet`, and `src/quirks`.
The objc/bundle-id helpers, the `home()` resolver, and config `loadDefaultFiles`
remain deferred pending roastty's naming decision; `background-image-opacity`
stays float-blocked.
