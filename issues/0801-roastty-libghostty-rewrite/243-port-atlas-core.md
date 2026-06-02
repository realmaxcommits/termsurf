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

# Experiment 243: Port the Atlas core (skyline packing: `reserve`/`fit`/`merge`/`set`)

## Description

Begin the texture atlas port from upstream `font/Atlas.zig`. The atlas is a 2D
rectangle bin-packer (skyline / "shelf-next-fit" variant from Jukka Jylänki's "A
Thousand Ways to Pack the Bin", as in freetype-gl) that hands out sub-rectangles
of a square texture for glyph sprites. This experiment ports the **allocation
core**: the value types (`Format`, `Node`, `Region`, the `AtlasFull` error), the
struct and its `new`/`clear` setup, `reserve` (with its `fit` and `merge`
helpers), and `set` (writing packed data into a reserved region). `grow`,
`setFromLarger`, and `dump` are deferred to Experiment 244, and the WASM
bindings are out of scope (macOS-only — see the issue Scope).

This is one coherent data structure with a single novel mechanism (the skyline
packer) and the upstream allocation/write tests as exact pass criteria, so it
stays one experiment. It pairs with the already-ported `Glyph` value type
(`font/glyph.rs`).

### Upstream types and fields (lines 27–93)

```zig
data: []u8,                                  // raw texture bytes
size: u32 = 0,                               // texture is always square (size×size)
nodes: std.ArrayListUnmanaged(Node) = .{},   // skyline nodes (free-space frontier)
format: Format = .grayscale,
modified: std.atomic.Value(usize) = .{ .raw = 0 },  // bumped on every data change
resized: std.atomic.Value(usize) = .{ .raw = 0 },   // bumped on every resize

pub const Format = enum(u8) { grayscale = 0, bgr = 1, bgra = 2,
    pub fn depth(self) u8 { .grayscale=>1, .bgr=>3, .bgra=>4 } };
const Node = struct { x: u32, y: u32, width: u32 };
pub const Error = error{ AtlasFull };
pub const Region = extern struct { x: u32, y: u32, width: u32, height: u32 };
const node_prealloc: usize = 64;
```

`modified` and `resized` are atomic counters so a GPU-upload thread can observe
"texture changed" / "texture resized" without locking (the user reads them
atomically). They are bumped by `clear`/`set` (and later `grow`), **not** by
`reserve`.

### `init` + `clear` (lines 100–120, 367–376)

`init(size, format)` allocates `size*size*depth` zeroed bytes, preallocates 64
nodes, and calls `clear`. `clear` zeroes the data, empties the node list, bumps
`modified`, and seeds a single node `{ x: 1, y: 1, width: size - 2 }` — the 1px
border on every edge avoids sampling artifacts, so usable space is
`(size-2)×(size-2)`.

### `reserve` + `fit` + `merge` (lines 136–251)

`reserve(width, height)`:

- `width == 0 && height == 0` returns `{0,0,0,0}` as-is (lets callers write
  empty data without special-casing).
- Scans every node calling `fit(i, width, height)`, which returns the `y` at
  which the rectangle can sit above node `i` (spanning as many nodes as needed
  to cover `width`), or `null` if it would cross the right/bottom border
  (`node.x + width > size - 1`, or `y + height > size - 1`). The chosen node
  uses the **exact** upstream guard: take node `i` when
  `(y + height) < best_height`, **or**
  `(y + height) == best_height && node.width > 0 && node.width < best_width`.
  The `node.width > 0` part of the tie-break matters because `reserve` only
  special-cases `width == 0 && height == 0`, so a `reserve(0, h)` can leave a
  genuine zero-width node that must not win a tie.
- No fit → `Error.AtlasFull`.
- Inserts a new node `{ x, y + height, width }` at the chosen index, then walks
  forward trimming/removing nodes the new node overlaps
  (`node.x += shrink; node.width -|= shrink;` removing any whose width hits 0),
  and finally `merge`s adjacent equal-`y` nodes.

`reserve` does **not** bump `modified` (no pixel data changes).

### `set` (lines 256–275)

`set(reg, data)` copies `reg.height` rows of `reg.width * depth` packed bytes
into the texture at the region's offset
(`((reg.y + i) * size + reg.x) * depth`), asserts the region is within the
border, and bumps `modified`. Data must be packed (no stride) and in the atlas's
format.

### Rust mapping

New file `roastty/src/font/atlas.rs`; `pub(crate) mod atlas;` in `font/mod.rs`.

- `pub(crate) enum Format { Grayscale = 0, Bgr = 1, Bgra = 2 }` (`repr(u8)`,
  `Debug, Clone, Copy, PartialEq, Eq`), with `pub(crate) fn depth(self) -> u32`
  (`1`/`3`/`4`; `u32` rather than `u8` so it composes with offset arithmetic
  without casts).
- `struct Node { x: u32, y: u32, width: u32 }` (`Copy`).
- `pub(crate) struct Region { x: u32, y: u32, width: u32, height: u32 }`
  (`Debug, Clone, Copy, PartialEq, Eq`).
- `pub(crate) enum AtlasError { AtlasFull }`
  (`Debug, Clone, Copy, PartialEq, Eq`).
- `pub(crate) struct Atlas { data: Vec<u8>, size: u32, nodes: Vec<Node>, format: Format, modified: AtomicUsize, resized: AtomicUsize }`.
  (No `Clone`/`Copy` — the atomics and the buffer make it a unique owner,
  matching upstream.)
- `pub(crate) const NODE_PREALLOC: usize = 64;`
- `pub(crate) fn new(size: u32, format: Format) -> Atlas`: the Rust analog of
  `init`. **Infallible** — `Vec` allocation aborts on OOM rather than returning
  a result, so there is no `Allocator.Error` to thread (see Faithfulness notes).
  Build `data = vec![0u8; (size*size*depth) as usize]`,
  `nodes = Vec::with_capacity(NODE_PREALLOC)`, atomics `0`, then call `clear`.
  (No separate `deinit` — `Drop` frees the `Vec`s.)
- `pub(crate) fn clear(&mut self)`: bump `modified`, `data.fill(0)`,
  `nodes.clear()` (retains capacity), push
  `Node { x: 1, y: 1, width: self.size - 2 }`.
- `fn fit(&self, idx: usize, width: u32, height: u32) -> Option<u32>`: faithful
  port — return `None` when `node.x + width > self.size - 1` or
  `y + height > self.size - 1`, walking nodes from `idx` accumulating `width`,
  taking the max `y`, with `width_left = width_left.saturating_sub(n.width)`.
- `pub(crate) fn reserve(&mut self, width: u32, height: u32) -> Result<Region, AtlasError>`:
  faithful port of the best-fit scan using the exact tie-break guard above,
  `AtlasFull` on no fit, node insert at the chosen index, the forward
  overlap-trim loop (which mutates the **overlapped node's** width —
  `node.x += shrink; node.width = node.width.saturating_sub(shrink)`, removing
  the node when its width reaches `0`), and `merge`. Uses
  `Vec::insert`/`Vec::remove` for `insert`/`orderedRemove`. **Infallible
  allocation** (the node insert cannot return an allocator error in Rust).
- `fn merge(&mut self)`: collapse adjacent equal-`y` nodes; loop guarded by
  `i + 1 < self.nodes.len()` (avoids the `len - 1` underflow when the list could
  be short — upstream relies on a non-empty list).
- `pub(crate) fn set(&mut self, reg: Region, data: &[u8])`: per-row
  `copy_from_slice` into `self.data` at `((reg.y + i) * size + reg.x) * depth`,
  reading `i * reg.width * depth` from `data`, then bump `modified`. Offsets are
  computed in `usize` (equivalent to upstream's `u32` math for in-range atlas
  sizes, without the `u32` overflow hazard). Bounds preconditions are
  `debug_assert!`ed (Rust's bounds-checked slicing is the always-on safety net,
  so the asserts document the contract without duplicating the panic).
- `modified`/`resized` use `std::sync::atomic::AtomicUsize`; bumps are
  `fetch_add(1, Ordering::Relaxed)` (`Relaxed` is Rust's `monotonic`), reads are
  `load(Ordering::Relaxed)`. A `pub(crate) fn modified(&self) -> usize` and
  `pub(crate) fn resized(&self) -> usize` expose them for tests/consumers.

### Faithfulness and scope notes

- **OOM error paths are not ported.** Upstream `init`/`reserve` return
  `Allocator.Error` and have `init error`/`reserve error` tests that inject a
  failing allocator. Rust's default allocation is infallible (aborts on OOM), so
  there is no allocator to inject and no error to return; `new`/`reserve` are
  infallible except for the domain error `AtlasFull`. Per the issue's Test
  Parity rule, this is the documented case where an upstream test "cannot be
  automated" without a fallible-allocator abstraction the rest of the crate does
  not yet use. `grow error`/`grow OOM` are likewise deferred with `grow` (Exp
  244).
- The skyline packer (`reserve`/`fit`/`merge`/the trim loop) is ported
  line-for-line; only the allocator threading and the WASM layer are dropped.
- `grow`, `setFromLarger`, and `dump` are deferred to Experiment 244 (the resize
  - strided-copy half, with the `grow`/`grow BGR`/`writing from larger` tests).
- No C ABI, header, or ABI inventory changes; no new dependencies (std only).

## Changes

1. `roastty/src/font/mod.rs`: add `pub(crate) mod atlas;` (and a one-line module
   note in the doc comment).

2. `roastty/src/font/atlas.rs` (new): `Format` (+`depth`), `Node`, `Region`,
   `AtlasError`, `Atlas` with `new`, `clear`, `fit`, `reserve`, `merge`, `set`,
   `modified`, `resized`, and `NODE_PREALLOC`.

3. Tests in `roastty/src/font/atlas.rs` — port the upstream allocation/write
   tests (those not needing `grow`/`setFromLarger`):
   - `format_depth`: `Grayscale→1`, `Bgr→3`, `Bgra→4`.
   - `exact_fit`: `new(34, Grayscale)`; `reserve(32, 32)` succeeds and leaves
     `modified` unchanged; a following `reserve(1, 1)` is `Err(AtlasFull)`.
   - `doesnt_fit`: `new(32, Grayscale)`; `reserve(32, 32)` is `Err(AtlasFull)`
     (the border leaves only 30×30).
   - `fit_multiple`: `new(32, Grayscale)`; two `reserve(15, 30)` succeed, the
     third `reserve(1, 1)` is `Err(AtlasFull)`.
   - `writing_data`: `new(32, Grayscale)`; `reserve(2, 2)`;
     `set(reg, &[1,2,3,4])` bumps `modified`;
     `data[33]==1, data[34]==2, data[65]==3, data[66]==4`.
   - `writing_bgr_data`: `new(32, Bgr)`; `reserve(1, 2)`;
     `set(reg, &[1,2,3, 4,5,6])`; with `depth==3`, `data[33*3..]==[1,2,3]` and
     `data[65*3..]==[4,5,6]`.

4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo test -p roastty font
cargo test -p roastty
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Format`/`depth`, `Node`, `Region`, `AtlasError`, and the `Atlas` struct match
  upstream, with the 1px-border `clear` seeding and the atomic `modified`/
  `resized` counters;
- `reserve`/`fit`/`merge` reproduce the skyline packing exactly — best-fit by
  `y + height` (tie-broken by node width), `AtlasFull` when nothing fits, the
  node insert, the forward overlap-trim, and the equal-`y` merge — and `reserve`
  does not bump `modified`;
- `set` writes packed rows at the correct offsets for grayscale and BGR and
  bumps `modified`;
- all six ported tests pass with the exact expected values;
- the OOM/allocator-error tests are documented as not ported, and `grow`/
  `setFromLarger`/`dump` are cleanly deferred;
- no C ABI, header, or ABI inventory changes;
- `cargo fmt` accepted and `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if `reserve`/`fit` needs a representation the
later `grow` slice forces to change (e.g. the node list or offset typing).

The experiment **fails** if the packing diverges from upstream (wrong best-fit,
a missed trim/merge, an off-by-one on the border), if `set` writes to the wrong
offsets or for the wrong depth, if `reserve` bumps `modified`, or if any public
C API/ABI changes.

## Design Review

Codex reviewed this design before implementation.

Review artifacts:

- Prompt: `logs/codex-review/20260602-091322-566574-prompt.md`
- Result: `logs/codex-review/20260602-091322-566574-last-message.md`

Codex confirmed the upstream source: `clear` seeds
`{ x: 1, y: 1, width: size - 2 }` and bumps `modified`; `reserve` does not bump
it; `fit` uses the `size - 1` border checks; `set` row offsets match; `Relaxed`
is the right analog for `monotonic`; deferring the OOM-path tests is acceptable
for this Rust slice; and the six test expectations are correct.

Two findings, both fixed in the design above before this commit:

1. **Medium — exact best-fit tie-break.** The original prose said the tie-break
   was "the smaller node width," omitting upstream's `node.width > 0` guard
   (`(y + height) == best_height && node.width > 0 && node.width < best_width`).
   That guard matters because `reserve` only special-cases
   `width == 0 && height == 0`, so a `reserve(0, h)` can leave a real zero-width
   node that must not win a tie. The design now states the exact condition, and
   the implementation ports it verbatim. (Codex suggested a narrow zero-width
   regression test; declined — upstream ships none, the node list is internal
   state rather than a stable contract, and a synthetic test risks pinning
   implementation detail. The exact guard plus the ported upstream tests cover
   the behavior.)
2. **Low — trim-loop variable.** The Rust-mapping line read
   `width = width.saturating_sub(shrink)`; upstream trims the **overlapped
   node's** width (`node.width -|= shrink`). Corrected to
   `node.width = node.width.saturating_sub(shrink)`.
