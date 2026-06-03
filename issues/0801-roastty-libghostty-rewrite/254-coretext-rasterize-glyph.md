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

# Experiment 254: CoreText glyph rasterization spike — glyph → alpha bitmap

## Description

The other half of the CoreText face: rasterize a glyph to a grayscale coverage
bitmap. This is the de-risking spike for `renderGlyph` — it proves the
`CGBitmapContext` + `CTFontDrawGlyphs` + buffer-readback path with a live test,
deliberately **minimal**: a single non-color, non-bold glyph rendered into a
bitmap sized to its natural bounding box, no cell **constraints**, no color/sbix
handling, no synthetic bold, and no atlas write — those layer on in the next
slice(s).

Because both Ghostty and Roastty rasterize through **CoreText**, the glyph
coverage is produced by the same system rasterizer — this is _not_ a
fidelity-fixture problem (unlike the z2d sprite path). The test only needs to
confirm a non-empty, plausibly-sized mask.

### Upstream pattern (`renderGlyph`, `font/face/coretext.zig`)

`renderGlyph` gets the glyph bounding rect, allocates a buffer
(`px_width * px_height * depth`), creates a `BitmapContext` over it, clears it,
sets antialiasing/fill, translates/scales the CTM to map the glyph's bounds into
the bitmap, and calls `drawGlyphs(&glyphs, &positions, ctx)`, then copies the
buffer into the atlas. The minimal spike keeps the core (bitmap-context → draw →
readback) and drops the constraint/scale/color/bold/atlas layers (so the CTM is
identity scale and the position is just the negated bearing).

### The objc2 API (verified, `objc2-core-graphics` 0.3.2)

- `CGColorSpace::new_device_gray() -> Option<CFRetained<CGColorSpace>>` (safe).
- `CGBitmapContextCreate(data: *mut c_void, width: usize, height: usize, bits_per_component: usize, bytes_per_row: usize, space: Option<&CGColorSpace>, bitmap_info: u32) -> Option<CFRetained<CGContext>>`
  (`unsafe`). For an 8-bit grayscale mask: `bits_per_component = 8`,
  `bytes_per_row = width`, `bitmap_info = 0` (`kCGImageAlphaNone`).
- `CGContext::set_should_antialias(Some(&ctx), true)`,
  `set_allows_antialiasing(Some(&ctx), true)`,
  `set_gray_fill_color(Some(&ctx), gray, alpha)` (assoc fns taking
  `Option<&CGContext>`; safe).
- `CTFont::draw_glyphs(glyphs: NonNull<CGGlyph>, positions: NonNull<CGPoint>, count: usize, context: &CGContext)`
  (`unsafe`).
- `CGRect`/`CGPoint`/`CGSize` from `objc2-core-foundation` (CFCGTypes, already
  enabled).

### Rust mapping (`roastty/src/font/face/coretext.rs`)

- `roastty/Cargo.toml`: extend `objc2-core-graphics` features with `CGContext`,
  `CGBitmapContext`, `CGColorSpace` (exact set finalized against `cargo build`).
- `pub(crate) struct RasterizedGlyph { pub width: u32, pub height: u32, pub bitmap: Vec<u8> }`
  (grayscale coverage, one byte per pixel, row-major in CoreGraphics bottom-up
  order — the orientation flip is the atlas-write slice's concern).
- `pub(crate) fn rasterize_glyph(&self, glyph: u16) -> Option<RasterizedGlyph>`:
  1. Get the glyph's full bounding rect (`CGRect`) via
     `bounding_rects_for_glyphs` with a null per-glyph buffer (origin + size).
  2. If `size.width < 0.25 || size.height < 0.25` → `None` (no/too-small
     outline, matching upstream).
  3. `px_w = size.width.ceil() as usize`, `px_h = size.height.ceil() as usize`
     (both `>= 1` after the guard).
  4. `colorspace = CGColorSpace::new_device_gray()?`.
  5. `let mut buf = vec![0u8; px_w * px_h];`.
  6. `let ctx = unsafe { CGBitmapContextCreate(buf.as_mut_ptr().cast(), px_w, px_h, 8, px_w, Some(&colorspace), 0) }?;`.
  7. `CGContext::set_should_antialias(Some(&ctx), true)`,
     `set_allows_antialiasing(Some(&ctx), true)`,
     `set_gray_fill_color(Some(&ctx), 1.0, 1.0)` (white glyph on the zeroed
     black buffer; the gray value is the coverage).
  8. `positions = [CGPoint { x: -rect.origin.x, y: -rect.origin.y }]` (negate
     the bearing so the bounding box maps to the bitmap origin);
     `glyphs = [glyph]`.
  9. `unsafe { self.font.draw_glyphs(NonNull(glyphs), NonNull(positions), 1, &ctx) }`.
  10. `drop(ctx)` (release the context **before** moving `buf`, since it holds a
      raw pointer into it), then return
      `Some(RasterizedGlyph { width: px_w as u32, height: px_h as u32, bitmap: buf })`.

### Faithfulness and scope notes

- The core rasterization (grayscale bitmap context, antialiased white-on-black
  draw, negated-bearing position, buffer readback) mirrors upstream. The CTM
  scale is identity here because cell **constraints** are deferred; the
  color/sbix and synthetic-bold branches and the atlas write are deferred too.
- The buffer is CoreGraphics' native bottom-up row order; the atlas-write slice
  applies the vertical flip when copying into the atlas.
- `objc2-core-graphics` gains the `CGContext`/`CGBitmapContext`/`CGColorSpace`
  features (it already provides `CGGlyph`).
- No C ABI, header, or ABI inventory changes.

## Changes

1. `roastty/Cargo.toml`: extend `objc2-core-graphics` features.
2. `roastty/src/font/face/coretext.rs`: add `RasterizedGlyph` and
   `rasterize_glyph`.
3. Tests in `coretext.rs` (live CoreText, macOS):
   - `rasterize_glyph_has_ink`: `Face::new("Menlo", 32.0)`; map `'M'` to a
     glyph; `rasterize_glyph(glyph)` is `Some(rg)` with `rg.width > 0`,
     `rg.height > 0`, `rg.bitmap.len() == (rg.width * rg.height) as usize`, and
     **some** byte `> 0` (the glyph has ink), with a non-trivial fraction
     non-zero.
   - `rasterize_space_is_empty_or_none`: the space glyph either rasterizes to
     all zero bytes or returns `None` (no outline) — i.e. it has no ink.

4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo test -p roastty face
cargo test -p roastty
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `rasterize_glyph` creates a grayscale bitmap context over a Rust buffer, draws
  the glyph antialiased at the negated bearing, and returns the coverage bitmap;
- a too-small/outline-less glyph yields `None`;
- a live `'M'` rasterizes to a correctly-sized, non-empty mask, and the space
  glyph has no ink;
- constraints, color, synthetic bold, and the atlas write are cleanly deferred;
- the context is released before the buffer is moved (no dangling pointer);
- no C ABI, header, or ABI inventory changes;
- `cargo fmt` accepted and `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the objc2 CGContext API needs a different
call/feature shape than expected.

The experiment **fails** if the bitmap is empty for an inked glyph, if the
context outlives the buffer, or if any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no required
changes**.

Review artifacts:

- Prompt: `logs/codex-review/20260602-202921-505704-prompt.md`
- Result: `logs/codex-review/20260602-202921-505704-last-message.md`

Codex confirmed the bitmap-context parameters are correct for a
one-byte-per-pixel grayscale no-alpha mask (`DeviceGray`, `8` bits/component,
`bytes_per_row = width`, `bitmap_info = 0`), that drawing white into a zeroed
buffer with antialiasing gives coverage values, that the **buffer-lifetime
safety is handled** (dropping `ctx` before moving `buf` is the critical step and
the design calls it out), that the `(-origin.x, -origin.y)` positioning with the
deferred bottom-up flip is appropriate for the spike, and that the `< 0.25`
guard matches upstream. Scope is clean (no constraints/color/bold/atlas) and the
tests are robust (inked glyph has nonzero coverage without pixel pinning; space
allows `None` or all-zero).
