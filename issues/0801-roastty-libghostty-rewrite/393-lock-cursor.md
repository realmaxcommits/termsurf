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

# Experiment 393: the lock cursor glyph

## Description

`add_cursor` renders the four **sprite** cursor styles (block, hollow block,
bar, underline) but the **lock** style currently draws nothing — it just clears
the cursor and returns (the glyph was deferred in the original port). Upstream
renders the lock cursor as a real codepoint glyph, the Nerd Font lock symbol
`0xF023`, via `renderCodepoint` (now ported as `SharedGrid::render_codepoint`,
Experiment 392). This experiment wires the `Lock` branch: it renders `0xF023`
and, if a font has it, draws it as the cursor glyph exactly like the sprite
cursors; if no font has it, it falls back to a cleared cursor (upstream logs and
returns). roastty embeds no Nerd Font, so the fall-back is the common local path
and what the test asserts.

## Upstream behavior

`addCursor`'s lock branch (`renderer/generic.zig`):

```zig
.lock => self.font_grid.renderCodepoint(
    self.alloc,
    0xF023, // lock symbol
    .regular,
    .text,
    .{ .cell_width = if (wide) 2 else 1, .grid_metrics = self.grid_metrics },
) catch |err| { log.warn(...); return; } orelse {
    // This should never happen because we embed nerd fonts so we just
    // log and return instead of fallback.
    log.warn("failed to find lock symbol for cursor codepoint=0xF023", .{});
    return;
},
```

The resulting `render` then builds the cursor vertex identically to the sprite
cursors — `.atlas = .grayscale`, `.is_cursor_glyph = true`, the same
`grid_pos`/`color`/`glyph_pos`/`glyph_size`/`bearings` — and `setCursor` stores
it. So the lock cursor differs from the sprite cursors only in **how the glyph
is produced**: `renderCodepoint(0xF023, .regular, .text)` instead of
`renderGlyph(sprite_index, cursor_sprite)`. If the codepoint cannot be rendered
(no font has it), upstream draws no cursor.

## Rust mapping (`roastty/src/renderer/cell.rs`)

`add_cursor` is restructured so the per-style `match` produces a `Render` (the
sprite styles via `render_glyph`, the lock via `render_codepoint`), and the
cursor vertex is then built once from that `Render`:

```rust
let opts = RenderOptions {
    grid_metrics: grid.metrics,
    cell_width: Some(if wide { 2 } else { 1 }),
    constraint: Constraint::default(),
    constraint_width: 1,
    thicken: false,
    thicken_strength: 255,
};

// The sprite cursors render a cursor sprite; the lock cursor renders the real
// lock symbol (0xF023). If no font has the lock glyph (roastty embeds no Nerd
// Font), clear the cursor and return, as upstream does.
let render = match cursor_style {
    CursorStyle::Block => {
        grid.render_glyph(Index::special(Special::Sprite), Sprite::CursorRect as u32, &opts)?
    }
    CursorStyle::BlockHollow => {
        grid.render_glyph(Index::special(Special::Sprite), Sprite::CursorHollowRect as u32, &opts)?
    }
    CursorStyle::Bar => {
        grid.render_glyph(Index::special(Special::Sprite), Sprite::CursorBar as u32, &opts)?
    }
    CursorStyle::Underline => {
        grid.render_glyph(Index::special(Special::Sprite), Sprite::CursorUnderline as u32, &opts)?
    }
    CursorStyle::Lock => {
        match grid.render_codepoint(0xF023, Style::Regular, Some(Presentation::Text), &opts)? {
            Some(render) => render,
            None => {
                contents.set_cursor(None, Some(CursorStyle::Lock));
                return Ok(());
            }
        }
    }
};

let vertex = CellTextVertex {
    glyph_pos: [render.glyph.atlas_x, render.glyph.atlas_y],
    glyph_size: [render.glyph.width, render.glyph.height],
    bearings: [
        i16::try_from(render.glyph.offset_x).expect("cursor x bearing fits i16"),
        i16::try_from(render.glyph.offset_y).expect("cursor y bearing fits i16"),
    ],
    grid_pos,
    color: [color[0], color[1], color[2], alpha],
    atlas: CellTextAtlas::Grayscale,
    flags: CellTextFlags::new(false, true),
    _padding: [0, 0],
};
contents.set_cursor(Some(vertex), Some(cursor_style));
Ok(())
```

`Style` is imported at module scope (`crate::font::Style`; `Presentation`
already is). The vertex-building and `set_cursor` are now shared across all five
styles — the lock glyph goes to the grayscale atlas with
`is_cursor_glyph = true`, exactly like the sprite cursors.

## Scope / faithfulness notes

- **Ported (bridged)**: the lock cursor glyph — `add_cursor`'s `Lock` branch
  renders the lock symbol `0xF023` via `render_codepoint` and draws it as the
  cursor (the same vertex/atlas/flags as the sprite cursors), falling back to a
  cleared cursor when no font has the glyph.
- **Faithful**: the codepoint is `0xF023` with `Style::Regular` /
  `Presentation:: Text` (upstream `.regular`, `.text`); the cursor vertex is
  built identically to the sprite cursors (grayscale atlas,
  `is_cursor_glyph = true`, same
  `grid_pos`/`color`/`glyph_pos`/`glyph_size`/`bearings`); the no-glyph case
  draws no cursor (upstream logs and returns — roastty clears, the same visible
  result). The sprite cursors are unchanged (same sprites, same `render_glyph`).
- **Faithful adaptation**: the per-style `match` now yields a `Render` and the
  vertex is built once (deduplicating what was the sprite-only tail), so the
  lock and sprite cursors share the vertex path — upstream likewise shares the
  `setCursor` tail after the `switch`. Upstream's `log.warn` on the missing
  glyph is a no-op clear here (roastty has no renderer logger in this path yet);
  the visible outcome (no cursor) is identical. The lock branch uses
  `render_codepoint(…)?`, **propagating** a render error rather than catching
  and logging it as upstream does — this is intentional and consistent with the
  existing sprite cursor branches, which already propagate `render_glyph` errors
  via `?` through `add_cursor`'s `Result`; only the **`None`** (no font has the
  glyph) case is handled inline as a cleared cursor, matching upstream's
  no-cursor-drawn outcome.
- **Deferred**: the under-cursor text recolor; the column-ordered decoration
  merge
  - link double-underline; the Metal upload. (Consumed by tests now.) The lock
    glyph's **Some** path (a real lock glyph drawn) is covered transitively —
    `render_codepoint`'s present-glyph render is tested (Experiment 392) and the
    cursor-vertex path is tested by the sprite cursor tests — because roastty
    embeds no Nerd Font, so `0xF023` cannot render in a local/CI test.
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`:
   - import `crate::font::Style` (alongside `Presentation`);
   - `add_cursor`: restructure the per-style `match` to produce a `Render` (the
     sprite styles via `render_glyph`, `Lock` via `render_codepoint(0xF023, …)`
     with a cleared-cursor fall-back when absent); build the cursor vertex once
     from the `Render`. Update the doc comment (lock now renders `0xF023`).
2. Tests (in `cell.rs`):
   - update `add_cursor_lock_clears` (renamed to reflect the fall-back): a
     `Lock` cursor on a Menlo grid (no `0xF023`) renders nothing and clears any
     prior cursor — `add_cursor` returns `Ok` and both cursor lists are empty;
   - confirm the sprite cursor tests (`add_cursor_maps_styles_and_routes`,
     `add_cursor_wide_uses_two_cells`) still pass unchanged.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty add_cursor
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `add_cursor`'s `Lock` branch renders `0xF023` via `render_codepoint` and draws
  it as the cursor (the same vertex path as the sprite cursors), falling back to
  a cleared cursor when no font has the glyph — faithful to upstream's
  `addCursor` lock branch;
- the tests pass (the lock fall-back clears the cursor and returns `Ok`; the
  sprite cursor tests are unchanged);
- the under-cursor recolor and the Metal upload stay deferred; the sprite
  cursors are unchanged;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the lock branch is wrong (the wrong codepoint, a
sprite instead of the codepoint, the vertex differing from the sprite cursors,
no fall-back when the glyph is absent), a sprite cursor changes, or any public C
API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Low** finding, now addressed:

- **Low (addressed):** the `Lock` branch uses `render_codepoint(…)?`, which
  propagates a render error, whereas upstream's lock branch catches the error,
  logs, and returns with no cursor drawn. This is acceptable as a roastty
  adaptation — the existing sprite cursor branches already propagate
  `render_glyph` errors via `?` through `add_cursor`'s `Result` — and the design
  now records that the error propagation is intentional and consistent with the
  current Rust cursor API; only the `None` (no font has the glyph) case is
  handled inline as a cleared cursor.

Codex confirmed everything else is faithful: the codepoint `0xF023`,
`Style::Regular`, `Presentation::Text`, the shared cursor-vertex tail, the
grayscale atlas, `is_cursor_glyph = true`, and the unchanged sprite cursor
rendering. It agreed that, given roastty embeds no Nerd Font, testing the lock
`None` fall-back plus relying on Experiment 392's present-codepoint render test
and the existing sprite cursor vertex tests is sufficient coverage for the lock
glyph.

Review artifacts:

- Prompt: `logs/codex-review/20260603-205421-028293-prompt.md` (design)
- Result: `logs/codex-review/20260603-205421-028293-last-message.md` (design)

## Result

**Result:** Pass

The lock cursor now renders its glyph.

- `roastty/src/renderer/cell.rs`:
  - `add_cursor` is restructured: `opts` is computed once, then a per-style
    `match` yields a `Render` — the four sprite styles via
    `render_glyph(Special::Sprite, Sprite::Cursor*)` (unchanged), and `Lock` via
    `render_codepoint(0xF023, Style::Regular, Some(Presentation::Text), &opts)?`
    (`Some` → the render; `None` → `set_cursor(None, Some(Lock))` + return). The
    cursor vertex is built once from the `Render` (grayscale atlas,
    `is_cursor_glyph = true`, same
    `grid_pos`/`color`/`glyph_pos`/`glyph_size`/`bearings`) and `set_cursor`
    stores it — shared across all five styles. `crate::font::Style` is imported;
    the doc comment now describes the lock-symbol render.

Test (in `cell.rs`): `add_cursor_lock_falls_back_when_glyph_absent` — a `Lock`
cursor on a Menlo grid (no `0xF023`, discovery disabled) → `render_codepoint`
returns `None`, so the lock cursor draws nothing, clears the pre-seeded block
cursor, and returns `Ok`. The sprite cursor tests
(`add_cursor_maps_styles_and_routes`, `add_cursor_wide_uses_two_cells`) are
unchanged and still pass.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2852 passed, 0 failed (no net change — the lock test
  was renamed/updated, not added; no regressions, sprite cursors unchanged).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

The lock cursor is now wired: `add_cursor`'s `Lock` branch renders the Nerd Font
lock symbol (`0xF023`) via `render_codepoint` and draws it through the same
cursor vertex path as the sprite cursors, falling back to a cleared cursor when
no font has the glyph (the local path, since roastty embeds no Nerd Font). All
five cursor styles are now faithfully ported.

The remaining renderer-bridge work: the under-cursor text recolor (the cell
under the cursor is recolored to contrast); the column-ordered decoration
merge + link double-underline; and the **Metal upload** of `Contents`.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design:
`add_cursor` resolves each cursor style to a `Render`, keeps the four sprite
styles on the same `render_glyph(Special::Sprite, Sprite::Cursor*)` path, and
routes `Lock` through
`render_codepoint(0xF023, Style::Regular, Some(Presentation::Text))`, with the
`None` fallback clearing the cursor and returning `Ok(())` (upstream's
no-cursor-drawn outcome). It confirmed the shared vertex tail preserves the
cursor rendering contract across all five styles (grayscale atlas,
`is_cursor_glyph = true`, same grid position/color/glyph
position/size/bearings), that the design-review Low about `?` error propagation
was recorded as an intentional Rust adaptation consistent with the sprite
branches, that the lock fall-back test is appropriate for Menlo/no-Nerd-Font and
the `Some` path is reasonably covered by Experiment 392 plus the unchanged
sprite cursor vertex tests, and that there is no public C ABI/header change.
Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-205717-964028-last-message.md`
