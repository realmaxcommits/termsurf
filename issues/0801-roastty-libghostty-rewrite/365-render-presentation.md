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

# Experiment 365: render returns the glyph's presentation

## Description

`SharedGrid::render_glyph` (Experiments 363–364) returns a bare `Glyph` — but
the draw path needs to know **which atlas** the glyph landed in (grayscale vs
color) to sample the right texture, and a bare `Glyph` drops that. Upstream's
`renderGlyph` returns a `Render { glyph, presentation }` precisely so the
renderer can `switch (render.presentation)` to choose the atlas. This experiment
closes that gap: `render_glyph` returns a `Render` carrying the glyph and its
presentation, and the cache stores `Render` (as upstream's
`glyphs: HashMap<GlyphKey, Render>` does).

## Upstream behavior

```zig
pub const Render = struct {
    glyph: Glyph,
    presentation: Presentation,
};
```

`SharedGrid.renderGlyph` returns `Render`, and `glyphs` is
`HashMap<GlyphKey, Render>`. The renderer uses `render.presentation` to pick the
atlas when emitting the GPU cell (`.emoji → .color`, `.text → .grayscale`) and
`render.glyph` for the atlas coordinates, size, and bearings. roastty already
computes the presentation inside `render_glyph` (to select the atlas); this
experiment simply returns it alongside the glyph and caches both.

## Rust mapping (`roastty/src/font/shared_grid.rs`)

```rust
/// A rendered glyph paired with the presentation that decided its atlas. Faithful
/// port of upstream `SharedGrid.Render`: the draw path uses `presentation` to
/// sample the right atlas (`Emoji → color`, `Text → grayscale`) and `glyph` for
/// the atlas placement, size, and bearings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Render {
    pub glyph: Glyph,
    pub presentation: Presentation,
}
```

`SharedGrid.glyphs` becomes `HashMap<GlyphKey, Render>`, and `render_glyph`
returns `Result<Render, ResolverRenderError>`:

```rust
pub(crate) fn render_glyph(
    &mut self,
    index: Index,
    glyph_index: u32,
    opts: &RenderOptions,
) -> Result<Render, ResolverRenderError> {
    let key = GlyphKey::new(index, glyph_index, opts);
    if let Some(&render) = self.glyphs.get(&key) {
        return Ok(render); // cache hit carries the glyph and its presentation
    }

    let presentation = self.resolver.get_presentation(index, glyph_index as u16)?;
    let glyph = match presentation {
        Presentation::Emoji => { /* …emoji constraint, atlas_color… */ }
        Presentation::Text => { /* …atlas_grayscale… */ }
    }?;

    let render = Render { glyph, presentation };
    self.glyphs.insert(key, render);
    Ok(render)
}
```

The presentation is already computed to select the atlas; this just keeps it.

## Scope / faithfulness notes

- **Ported (bridged)**: `SharedGrid::render_glyph` now returns upstream's
  `Render { glyph, presentation }`, and the glyph cache stores `Render` —
  matching upstream's `glyphs: HashMap<GlyphKey, Render>`.
- **Faithful**: the presentation returned is exactly the one used to select the
  atlas (`Emoji → color`, `Text → grayscale`); the cache hit returns the same
  `Render` (glyph + presentation) it stored; the key is unchanged (Experiment
  364).
- **Faithful adaptation**: `Render` derives `Copy` (both `Glyph` and
  `Presentation` are `Copy`), so the cache hit and the return are by-value
  copies — no clone. No behavior change beyond surfacing the presentation.
- **Deferred**: the Metal draw path that consumes `Render` (picking the atlas by
  `presentation`, placing the glyph at `run.offset + cell.x`), and cache
  invalidation. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/font/shared_grid.rs`: add the `Render` struct; change `glyphs`
   to `HashMap<GlyphKey, Render>`; change `render_glyph`'s return type to
   `Result<Render, ResolverRenderError>` (cache and return `Render`).
2. Update the existing `shared_grid` tests to read `render.glyph.*`, and assert
   the returned `presentation` (text/sprite → `Text`).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty shared_grid
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `render_glyph` returns `Render { glyph, presentation }` with the presentation
  that selected the atlas, and the cache stores `Render` — faithful to
  upstream's `Render` and `glyphs` map;
- the updated tests pass (glyph fields via `render.glyph`, presentation
  asserted), and the existing tests still pass;
- the Metal consumer and cache invalidation stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the returned presentation disagrees with the atlas
chosen, the cache stores the wrong value, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed: returning and caching `Render { glyph, presentation }`
is faithful to upstream — the presentation is already computed before atlas
selection, so returning exactly that value preserves the invariant the draw path
needs (`Text → grayscale`, `Emoji → color`); caching `Render` rather than
recomputing the presentation on a hit is correct (presentation is deterministic
for `(index, glyph)` within a grid, and reload invalidation is already
deferred); `Render` can safely be `Copy` because both `Glyph` and `Presentation`
are `Copy`, and the `HashMap<GlyphKey, Render>` value change keeps the
Experiment 364 key semantics with clean equality/copy behavior; and updating the
tests to use `render.glyph.*` and assert `presentation == Text` for Menlo and
sprite is sufficient for this environment. Its one note — that an
emoji/color-atlas test would be useful later but should not block this
experiment without a stable bundled color font — is recorded for a future
experiment.

Review artifacts:

- Prompt: `logs/codex-review/20260603-174752-681545-prompt.md` (design)
- Result: `logs/codex-review/20260603-174752-681545-last-message.md` (design)
