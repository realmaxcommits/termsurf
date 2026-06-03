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

# Experiment 294: z2d port — Polygon Contour

## Description

The stroke plotter assembles its outline from `Polygon.Contour`s — polylines of
corner points that are later converted into `Polygon` edges. This is the piece
deferred from Experiment 284 (`Contour` + `addEdgesFromContour`). Porting it now
is the last foundation before the stroke plotter. Upstream backs `Contour` with
a doubly-linked list (`vendor/z2d/src/internal/tess/Polygon.zig`); since the
single-segment butt-cap stroke (the box-drawing diagonals) only
appends/prepends/ concatenates corners — never mid-list inserts — a
`Vec<Point>`-backed contour is a faithful behavioral port for this scope.

## Upstream behavior (`Polygon.Contour`)

- `Contour { corners (linked list), len, scale }`.
- `plot(point, before)`: scale the point by `self.scale` and append it (or
  insert it before a given node). `plotReverse(point)`: scale and **prepend**.
- `concat(other)`: move `other`'s corners onto the end of `self` (emptying
  `other`).
- `Polygon.addEdgesFromContour(contour)`: walk the corners in order, `addEdge`
  between each consecutive pair, then close with an edge from the last corner
  back to the first. (The corner points are already scaled by the _contour_'s
  scale; `addEdge` then scales by the _polygon_'s scale — so the stroke path
  keeps the result polygon at `scale = 1` to avoid double-scaling.)

## Rust mapping (`roastty/src/font/sprite/raster.rs`)

- `struct Contour { corners: Vec<Point>, scale: f64 }` with `new(scale)`,
  `len()`, `plot(point)` (append the scaled point), `plot_reverse(point)`
  (prepend), and `concat(&mut other)` (drain `other`'s corners onto `self`).
- `Polygon::add_edges_from_contour(&mut self, contour: &Contour)` — walk the
  corners adding an edge between each consecutive pair, then the closing edge.

## Scope / faithfulness notes

- **Deferred**: the mid-list `plot(point, before)` insertion (used only by the
  stroke **joins**, which the single-segment diagonals do not need) — it will be
  ported as an index-based insert when the stroke plotter's join logic lands.
  The `Contour.List`/`deinit` (arena bookkeeping) are not needed with owned
  `Vec`s.
- The `Vec<Point>` contour is behaviorally identical to the linked list for
  append/prepend/concat/iterate; the points are scaled on `plot`/`plot_reverse`
  exactly as upstream.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/raster.rs`: add `Contour` and
   `Polygon::add_edges_from_contour`.
2. Tests (deterministic):
   - `contour_plot_scales`: a `Contour::new(4.0)`; `plot((1,2))` stores the
     scaled `(4,8)`; `plot_reverse((0,0))` prepends `(0,0)`; `len() == 2`.
   - `contour_concat`: concatenating a second contour appends its corners and
     empties it.
   - `add_edges_from_contour_square`: a square contour `(0,0),(4,0),(4,4),(0,4)`
     at contour `scale = 4` → `add_edges_from_contour` into a
     `Polygon::new(1.0)` yields two edges `{y0:0, y1:16, x_start:16, x_inc:0}`
     (right) and `{y0:16, y1:0, x_start:0, x_inc:0}` (the closing left edge);
     the horizontal top/bottom are filtered.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty raster
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Contour` reproduces z2d's scaled append/prepend/concat and
  `add_edges_from_contour` reproduces the consecutive-pair + closing-edge
  assembly, verified by the tests;
- the mid-list join insert, the stroke plotter, and `Canvas` path methods stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the stroke plotter needs the mid-list insert
sooner than the join experiment.

The experiment **fails** if the contour assembly diverges from z2d or any public
C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no required
changes**. It confirmed a `Vec<Point>` preserves the linked-list behavior needed
here (scaled append/prepend, concat-by-moving with `other` emptied, length,
ordered iteration), that deferring the mid-list `plot(point, before)` is sound
for the single-segment butt-cap stroke, that `add_edges_from_contour` matches
upstream (consecutive pairs then the closing `last → first`), that the
double-scaling note is correct (the receiving stroke polygon uses
`scale = 1.0`), and that the square test recomputes to the two vertical edges
`{0,16,16,0}` / `{16,0,0,0}`.

Review artifacts:

- Prompt: `logs/codex-review/20260603-065227-846314-prompt.md`
- Result: `logs/codex-review/20260603-065227-846314-last-message.md`

## Result

**Result:** Pass

`roastty/src/font/sprite/raster.rs` gained `Contour` (`new`/`len`/`plot`/
`plot_reverse`/`concat`, a `Vec<Point>` of scaled corners) and
`Polygon::add_edges_from_contour` (consecutive-pair edges + the closing edge).

Tests (deterministic):

- `contour_plot_scales` — `plot((1,2))` at scale 4 stores `(4,8)`;
  `plot_reverse((0,0))` prepends; `len == 2`.
- `contour_concat` — drains the other contour onto self (`len` → 0).
- `add_edges_from_contour_square` — a `(0,0),(4,0),(4,4),(0,4)` contour at scale
  4 into a `Polygon::new(1.0)` → the two vertical edges `{0,16,16,0}` /
  `{16,0,0,0}` (horizontals filtered).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty raster` → 66 passed (3 new).
- `cargo test -p roastty` → 2567 passed, 0 failed (no regressions; +3).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

`Contour` — the last stroke-assembly foundation — is in place. Every z2d
ingredient the single-segment stroke needs now exists: `Slope`, `Face`
(corners + butt cap), `PointBuffer`,
`Polygon`/`Contour`/`add_edges_from_contour`, and `fill_polygon`. The next slice
is the **single-segment stroke** — building the outline `Polygon` for a 2-point
butt-cap line from its `Face` (the `p0_cw → p1_cw`, butt cap at `p1`,
`p1_ccw → p0_ccw`, butt cap at `p0` ring) — and then a `Canvas::line` that
applies the padding translation and calls `fill_polygon`. That renders the
box-drawing **diagonals** (`0x2571`–`0x2573`). The multi-segment joins (the
stroke plotter's mid-list insert), round caps/joins (`Pen`), and curve strokes
come after. Alongside the sprite font remain the discovery consumer, the UCD
emoji-presentation default, codepoint overrides, the shaper, the Nerd Font
attribute table, and SVG color detection.

## Completion Review

Codex reviewed the completed implementation and result and found **no required
changes**. It confirmed `plot`/`plot_reverse`/`concat`/`len`/ordered iteration
match the upstream linked-list behavior for the needed operations (only the
mid-list insert deferred), that `add_edges_from_contour` matches the
consecutive-pair + closing-edge flow, and that the three tests correctly cover
scaling/prepending, concat draining, and the scaled square's two vertical edges.
It judged the gates clean.

Review artifacts:

- Prompt: `logs/codex-review/20260603-065430-394556-prompt.md`
- Result: `logs/codex-review/20260603-065430-394556-last-message.md`
