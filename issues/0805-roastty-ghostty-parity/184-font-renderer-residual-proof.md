# Experiment 184: Font renderer residual proof

## Description

`RUNTIME-007B2B2B2B2` is the last font runtime/UI gap. Earlier experiments
closed config propagation, live font-grid updates, shaping-break routing,
non-`sbix` thickening mechanics, OpenType feature/variation propagation, metric
modifier math, and deterministic fallback resolution. The remaining row still
groups four renderer-visible concerns:

- broad fallback/shaping visual output;
- bitmap/color font thickening edge cases;
- glyph metrics as seen by the renderer beyond modifier math;
- broader renderer-visible font pixel parity.

This experiment will turn that residual row into an explicit font renderer proof
instead of a vague bucket. The first pass is an audit of the existing
font-rendering evidence in `roastty/src/font` and `roastty/src/renderer`. If the
audit shows that any of the four concerns are not actually covered, the
implementation will add focused tests or guards for that missing slice. If the
coverage is already present, the implementation will add the missing inventory
guard that binds those tests to CFG-223 and closes the font row.

This experiment will not claim GUI screenshot parity, native macOS app
walkthrough parity, or notification/link/bell presentation parity. Those remain
owned by `RUNTIME-011B2B` and `RUNTIME-012B2B2B2B2B3`.

## Changes

- Audit existing Roastty font renderer tests in `roastty/src/font` and
  `roastty/src/renderer`, including:
  - CoreText grayscale glyph rasterization and atlas placement;
  - stretched-cell glyph pixels and renderer-visible bearings;
  - `font-thicken` canvas padding and strength effects;
  - Apple Color Emoji color-glyph BGRA atlas rendering and wrong-atlas-format
    rejection;
  - CoreText fallback discovery for CJK, supplementary emoji, and LastResort
    rejection;
  - shaping cluster behavior for ASCII, RTL, supplementary characters, and
    combining marks;
  - renderer-facing font-grid metric propagation.
- Add or update focused Rust tests only where the audit finds a concrete missing
  font renderer slice.
- Add `issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py` to
  prove the final font row is backed by concrete Rust tests and Ghostty/Roastty
  source anchors, not only narrative text.
- Update `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py` to
  split or close `RUNTIME-007B2B2B2B2` based on the audit result. The expected
  successful outcome is an Oracle-complete font renderer residual row and
  CFG-223 gap count dropping from 3 to 2.
- Update `issues/0805-roastty-ghostty-parity/README.md` Learnings and
  Experiments index.

## Verification

Pass criteria:

- The new or updated font residual guard proves every former
  `RUNTIME-007B2B2B2B2` concern has concrete evidence:
  - fallback/shaping renderer-visible output;
  - bitmap/color font behavior and thickening edge cases;
  - renderer-visible glyph metrics and bearings;
  - renderer-visible font pixel evidence.
- `config_runtime_inventory.py` reports CFG-223 with 87 runtime rows, 82
  Oracle-complete rows, 85 closed rows, 2 incomplete rows, and 2 runtime gaps,
  or the experiment records a narrower split with an explicit remaining font
  subgap if the audit proves one is still real.
- Existing font guards that previously asserted the residual font gap are
  updated to assert the new narrowed/closed state.

Commands:

```bash
cargo test --manifest-path roastty/Cargo.toml font -- --test-threads=1
cargo test --manifest-path roastty/Cargo.toml render_glyph -- --test-threads=1
cargo test --manifest-path roastty/Cargo.toml shape_ -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
for f in issues/0805-roastty-ghostty-parity/font_*_runtime_parity.py issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py issues/0805-roastty-ghostty-parity/renderer_visual_residual_audit.py; do PYTHONDONTWRITEBYTECODE=1 python3 "$f" || exit 1; done
cargo fmt --check --manifest-path roastty/Cargo.toml
prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/184-font-renderer-residual-proof.md
git diff --check
```

## Design Review

Fresh-context Codex adversarial reviewer `Lorentz the 3rd` reviewed the design
and returned `VERDICT: APPROVED` with no findings. After the review, the
verification count expectation was corrected from 84 closed / 3 incomplete rows
to 85 closed / 2 incomplete rows for the successful font-row-closure path,
matching `config_runtime_inventory.py`'s closed-row calculation.

## Result

**Result:** Pass

The residual font renderer row is now closed. The implementation added
`font_renderer_residual_color_sbix_thicken_skips_canvas_padding`, a focused
CoreText test proving the Ghostty-compatible bitmap-color `sbix` path ignores
`font-thicken` canvas padding while preserving the rendered glyph metrics and
bearings. The new `font_renderer_residual_parity.py` guard ties the closed row
to pinned Ghostty source anchors and Roastty evidence for:

- grayscale glyph rasterization and atlas placement;
- stretched-cell glyph pixels and bearings;
- non-`sbix` thicken padding and strength effects;
- `sbix` bitmap-color thicken-padding suppression;
- Apple Color Emoji BGRA atlas rendering and wrong-atlas-format rejection;
- CoreText fallback discovery for CJK and supplementary emoji plus LastResort
  rejection;
- shaping clusters for ASCII, RTL, supplementary scalars, and combining marks;
- renderer-facing font-grid metric propagation.

`RUNTIME-007B2B2B2B2` is now `Oracle complete`. CFG-223 now reports 87 runtime
rows, 82 Oracle-complete rows, 85 closed rows, 2 incomplete rows, and 2 runtime
gaps.

Verification run:

```bash
cargo test --manifest-path roastty/Cargo.toml font_renderer_residual_color_sbix_thicken_skips_canvas_padding -- --test-threads=1
cargo test --manifest-path roastty/Cargo.toml render_glyph -- --test-threads=1
cargo test --manifest-path roastty/Cargo.toml shape_ -- --test-threads=1
cargo test --manifest-path roastty/Cargo.toml font -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
for f in issues/0805-roastty-ghostty-parity/font_*_runtime_parity.py issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py issues/0805-roastty-ghostty-parity/renderer_visual_residual_audit.py; do PYTHONDONTWRITEBYTECODE=1 python3 "$f" || exit 1; done
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py
python3 -m py_compile issues/0805-roastty-ghostty-parity/font_renderer_residual_parity.py issues/0805-roastty-ghostty-parity/font_fallback_runtime_parity.py issues/0805-roastty-ghostty-parity/font_grid_runtime_parity.py issues/0805-roastty-ghostty-parity/font_live_grid_update_runtime_parity.py issues/0805-roastty-ghostty-parity/font_shaping_break_runtime_parity.py issues/0805-roastty-ghostty-parity/font_thicken_render_runtime_parity.py issues/0805-roastty-ghostty-parity/font_feature_runtime_parity.py issues/0805-roastty-ghostty-parity/font_variation_runtime_parity.py issues/0805-roastty-ghostty-parity/font_metric_modifier_runtime_parity.py issues/0805-roastty-ghostty-parity/renderer_visual_residual_audit.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py
cargo fmt --check --manifest-path roastty/Cargo.toml
```

## Conclusion

Font renderer parity no longer has a residual CFG-223 gap. The remaining Issue
805 runtime/UI gaps are the live macOS app walkthrough row `RUNTIME-011B2B` and
notification/link/bell GUI effects row `RUNTIME-012B2B2B2B2B3`.

## Completion Review

Fresh-context Codex adversarial reviewer `Arendt the 3rd` reviewed the completed
experiment and returned `VERDICT: CHANGES REQUIRED` for one required finding:
the regenerated `config-runtime-inventory.md` and `config-matrix.md` files were
not prettier-formatted. The generated markdown files were formatted with
prettier, and the failing formatting check plus the font residual, renderer
residual, terminal residual, and `git diff --check` guards were rerun.

Fresh-context Codex re-reviewer `Nietzsche the 3rd` reviewed the formatting fix
and returned `VERDICT: APPROVED` with no findings.
