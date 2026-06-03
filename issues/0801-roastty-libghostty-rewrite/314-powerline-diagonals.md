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

# Experiment 314: the powerline diagonal spacers (E0B9/E0BB/E0BD/E0BF)

## Description

The thin diagonal powerline separators `E0B9`/`E0BF` (`╲`) and `E0BB`/`E0BD`
(`╱`) are drawn by upstream `powerline.zig` as **box-drawing diagonals** — each
delegates to `box.lightDiagonalUpperLeftToLowerRight` or
`box.lightDiagonalUpperRightToLowerLeft`, the same routines that draw `U+2572`
and `U+2571` (ported in Experiment 296 as `draw_box_diagonal`). This experiment
ports `draw_powerline_diagonal`, a thin dispatch that maps each powerline
codepoint to the equivalent box diagonal and delegates to the existing
`draw_box_diagonal`. With it, all of `E0B0`–`E0BF` are covered.

## Upstream behavior (`powerline.zig`)

- `drawE0B9` / `drawE0BF`:
  `box.lightDiagonalUpperLeftToLowerRight(metrics, canvas)` — the `╲` diagonal
  (the same as `U+2572`).
- `drawE0BB` / `drawE0BD`:
  `box.lightDiagonalUpperRightToLowerLeft(metrics, canvas)` — the `╱` diagonal
  (the same as `U+2571`).

(The four ignore `width`/`height` and use the cell metrics, via the box diagonal
routines.)

## Rust mapping (`roastty/src/font/sprite/draw.rs`)

`pub(crate) fn draw_powerline_diagonal(cp: u32, metrics: &Metrics, canvas: &mut Canvas) -> bool`
— map the powerline codepoint to the equivalent box-diagonal codepoint and
delegate to the already-ported `draw_box_diagonal`:

- `0xE0B9` | `0xE0BF` → `0x2572` (`╲`, upper-left to lower-right);
- `0xE0BB` | `0xE0BD` → `0x2571` (`╱`, upper-right to lower-left);
- `_ => return false`.

`draw_powerline_diagonal(cp, metrics, canvas)` returns
`draw_box_diagonal(box_cp, metrics, canvas)`. Update the module doc.

## Scope / faithfulness notes

- **Ported**: the four diagonal powerline spacers (delegating to the existing
  box diagonals).
- **Deferred**: the powerline flames (`E0D2`/`E0D4`) and the sprite dispatch.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/draw.rs`: add `draw_powerline_diagonal`; update the
   module doc.
2. Tests (deterministic — the fixture `9×18` cell; the diagonals pass through
   the center `(4, 9)`, like `draw_box_diagonal`):
   - `powerline_e0b9_backslash` / `_e0bf_backslash`: `E0B9`/`E0BF` draw the `╲`
     diagonal — the center `(4, 9)` is inked, the top-right corner `(8, 1)` is
     not (matching `U+2572`).
   - `powerline_e0bb_slash` / `_e0bd_slash`: `E0BB`/`E0BD` draw the `╱` diagonal
     — the center `(4, 9)` is inked, the top-left corner `(0, 1)` is not
     (matching `U+2571`).
   - `powerline_diagonal_matches_box`: each powerline diagonal's rendered buffer
     equals the corresponding `draw_box_diagonal` (`0x2572`/`0x2571`) buffer —
     pinning the delegation.
   - `draw_powerline_diagonal_excludes`: `0x2500`, `0xE0B0`, `'M'` return
     `false` and draw nothing.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty sprite
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `draw_powerline_diagonal` maps `E0B9`/`E0BF` to `╲` and `E0BB`/`E0BD` to `╱`
  via `draw_box_diagonal`, returning `false` otherwise;
- the diagonal-orientation, delegation-equality, and exclusion tests confirm the
  rendering;
- the flames and the sprite dispatch stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a powerline diagonal needs geometry beyond the
box diagonals (it should not — upstream delegates to them directly).

The experiment **fails** if the diagonal mapping diverges from z2d, or any
public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
changes**. It confirmed the mapping is correct (`E0B9 | E0BF → 0x2572` the `╲`
backslash, `E0BB | E0BD → 0x2571` the `╱` slash), matching upstream's
`lightDiagonalUpperLeftToLowerRight`/`lightDiagonalUpperRightToLowerLeft` and
the existing `draw_box_diagonal` orientation from Experiment 296; that
delegating is faithful because upstream delegates to the same box routines and
ignores the glyph `width`/`height`; and that the orientation tests plus the
buffer-equality delegation test are sound. No Optional findings.

Review artifacts:

- Prompt: `logs/codex-review/20260603-091310-356821-prompt.md`
- Result: `logs/codex-review/20260603-091310-356821-last-message.md`
