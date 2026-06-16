# Experiment 134: Cursor Renderer Runtime

## Description

`RUNTIME-008B2` still groups several renderer-visible gaps together: background
blur, real compositor opacity, window padding layout pixels, cursor style
shape/rendering pixels, custom shader output, and broader GUI/pixel parity.

Roastty already has deterministic renderer coverage for a narrower cursor slice
that does not require a GUI screenshot:

- terminal visual style, visibility, focus, and blink state are converted into
  active renderer cursor overlay/uniform inputs for the non-password,
  non-preedit path;
- focused/unfocused and blink-visible/blink-hidden cursor states choose visible,
  hidden, or hollow cursor styles;
- cursor color resolves from OSC 12, `cursor-color`, the under-cursor cell, and
  defaults;
- `cursor-text` resolves the text redraw color for block cursors;
- cursor sprite styles map to the same sprite glyph classes as pinned Ghostty;
- block cursor vertices route to the first reserved cursor list and non-block
  cursor vertices route to the last reserved cursor list;
- wide cursor rendering uses the two-cell cursor sprite path;
- lock cursor rendering uses the Nerd Font lock codepoint and clears the cursor
  if the glyph is unavailable, once lock style is selected.

This experiment will split the remaining renderer-visible row:

- `RUNTIME-008B2A`: **Oracle complete** for deterministic active cursor
  overlay/uniform input branches, cursor color/text color resolution, selected
  cursor sprite/glyph render data, wide cursor render data, lock fallback
  rendering behavior, and cursor list routing.
- `RUNTIME-008B2B`: **Gap** for background blur, real compositor opacity, window
  padding layout pixels, password/preedit cursor-style priority through the
  active renderer path, GUI cursor pixels/screenshots, custom shader output, and
  broader GUI/pixel parity.

This experiment will not claim GUI cursor pixel parity. It will prove the
renderer data that feeds cursor drawing and leave actual app screenshot
comparison for a later visual walkthrough.

## Changes

- `issues/0805-roastty-ghostty-parity/cursor_renderer_runtime_parity.py`
  - Add a static guard checking pinned Ghostty markers:
    - `renderer/cursor.zig` contains preedit and password-input cursor priority,
      and that priority remains explicitly assigned to `RUNTIME-008B2B`;
    - `renderer/generic.zig` derives cursor style from terminal visual style;
    - focused cursor alpha uses `cursor-opacity`;
    - block, hollow-block, bar, and underline cursor styles map to sprite
      glyphs;
    - lock cursor renders codepoint `0xF023`;
    - cursor vertices carry color, alpha, glyph position/size, bearings, and
      `is_cursor_glyph`;
    - `renderer/cell.zig` routes block cursors before cell text and non-block
      cursors after cell text.
  - Check Roastty markers:
    - `FrameRenderState::from_terminal_with_cursor_options`;
    - `render_state_derives_visible_block_cursor_overlay`;
    - `render_state_cursor_color_comes_from_osc12`;
    - `render_state_block_sets_uniform_underline_does_not`;
    - focused/unfocused blink visibility tests;
    - `add_cursor_maps_styles_and_routes`;
    - `add_cursor_wide_uses_two_cells`;
    - `add_cursor_lock_falls_back_when_glyph_absent`;
    - `cursor_text_color_resolves_the_cursor_text_config`;
    - `cursor_color_resolves_with_precedence`;
    - `block_cursor_pos_adjusts_for_wide_kind`;
    - `set_cursor_*` reserved-list routing and clearing tests;
    - the runtime inventory split and CFG-223 counts.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split `RUNTIME-008B2` into `RUNTIME-008B2A` and `RUNTIME-008B2B`.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Regenerate from the inventory script.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-223 summary. It must remain `Gap`.
- Existing CFG-223 static guards that hard-code current runtime row counts
  - Update expected counts after the split: 43 runtime rows, 36 Oracle complete
    rows, 38 closed rows, and 5 remaining runtime gaps.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- `RUNTIME-008B2A` is Oracle complete and cites concrete deterministic tests for
  active non-password/non-preedit cursor overlay/uniform input branches, cursor
  color/text color resolution, selected sprite style mapping, wide cursor render
  data, lock fallback rendering after lock style selection, and cursor list
  routing.
- `RUNTIME-008B2B` remains `Gap` and explicitly owns background blur, real
  compositor opacity, window padding layout pixels, password/preedit
  cursor-style priority through the active renderer path, GUI cursor pixels,
  custom shader output, and broader GUI/pixel parity.
- `CFG-223` remains `Gap`.
- Existing static parity guards remain internally consistent after the row-count
  change.

Commands:

```bash
cargo test --manifest-path roastty/Cargo.toml render_state_derives_visible_block_cursor_overlay
cargo test --manifest-path roastty/Cargo.toml render_state_cursor_color_comes_from_osc12
cargo test --manifest-path roastty/Cargo.toml render_state_block_sets_uniform_underline_does_not
cargo test --manifest-path roastty/Cargo.toml cursor_blink_render_state
cargo test --manifest-path roastty/Cargo.toml add_cursor
cargo test --manifest-path roastty/Cargo.toml cursor_text_color_resolves_the_cursor_text_config
cargo test --manifest-path roastty/Cargo.toml cursor_color_resolves_with_precedence
cargo test --manifest-path roastty/Cargo.toml block_cursor_pos_adjusts_for_wide_kind
cargo test --manifest-path roastty/Cargo.toml set_cursor
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/cursor_renderer_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
cargo fmt --manifest-path roastty/Cargo.toml
cargo fmt --manifest-path roastty/Cargo.toml --check
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/134-cursor-renderer-runtime.md
git diff --check
```

Fail criteria:

- The inventory claims GUI cursor pixel parity, screenshot parity, or full
  renderer visual parity.
- The inventory claims full Ghostty cursor-style priority parity, including
  password/preedit cursor selection, without active-path proof.
- The complete row relies only on parser/default evidence instead of renderer
  input, color resolution, render-data, and list-routing tests.
- `RUNTIME-008B2B` omits background blur, real compositor opacity, window
  padding layout pixels, password/preedit cursor-style priority, GUI cursor
  pixels, custom shader output, or broader GUI/pixel parity.
- CFG-223 is marked complete.

## Design Review

**Reviewer:** Codex adversarial subagent with fresh context.

**Initial verdict:** Changes required.

The reviewer found two required issues:

- The design claimed cursor renderer input derivation without accounting for
  pinned Ghostty's `renderer/cursor.zig` priority rules: preedit forces block
  cursor and password input forces lock cursor before visibility/focus/blink
  checks.
- The pass criteria could mark `RUNTIME-008B2A` complete without proving
  password-input lock selection through Roastty's active renderer path.

**Fixes:**

- Narrowed the complete slice to active non-password/non-preedit cursor
  overlay/uniform branches, color/text-color resolution, selected-style render
  data, lock rendering after lock style selection, wide cursor render data, and
  cursor list routing.
- Added explicit `RUNTIME-008B2B` ownership for password/preedit cursor-style
  priority through the active renderer path.
- Added planned static guard coverage for pinned Ghostty `renderer/cursor.zig`
  preedit/password priority markers and the remaining-gap assignment.

**Re-review verdict:** Approved.

The reviewer confirmed the design now narrows `RUNTIME-008B2A` to the active
non-password/non-preedit cursor slice, assigns password/preedit cursor-style
priority through the active renderer path to `RUNTIME-008B2B`, and prevents a
full cursor-priority claim without active-path proof. The reviewer reported no
new required findings.

## Result

**Result:** Pass

Implemented the deterministic cursor renderer split without claiming GUI cursor
pixel parity or full Ghostty cursor-style priority:

- Added `cursor_renderer_runtime_parity.py` to statically guard pinned Ghostty's
  cursor render markers, Roastty's active cursor render-state tests, cursor
  render-data tests, color-resolution tests, list-routing tests, and the
  remaining password/preedit cursor-priority gap assignment.
- Split `RUNTIME-008B2` into:
  - `RUNTIME-008B2A`: **Oracle complete** for deterministic active cursor
    overlay/uniform branches, cursor color/text-color resolution, selected
    cursor sprite/glyph render data, wide cursor render data, lock fallback
    rendering after lock selection, and cursor list routing.
  - `RUNTIME-008B2B`: **Gap** for background blur, real compositor opacity,
    window padding layout pixels, password/preedit cursor-style priority through
    the active renderer path, GUI cursor pixels, custom shader output, and
    broader GUI/pixel parity.

Verification passed:

```bash
cargo test --manifest-path roastty/Cargo.toml render_state_derives_visible_block_cursor_overlay
cargo test --manifest-path roastty/Cargo.toml render_state_cursor_color_comes_from_osc12
cargo test --manifest-path roastty/Cargo.toml render_state_block_sets_uniform_underline_does_not
cargo test --manifest-path roastty/Cargo.toml cursor_blink_render_state
cargo test --manifest-path roastty/Cargo.toml add_cursor
cargo test --manifest-path roastty/Cargo.toml cursor_text_color_resolves_the_cursor_text_config
cargo test --manifest-path roastty/Cargo.toml cursor_color_resolves_with_precedence
cargo test --manifest-path roastty/Cargo.toml block_cursor_pos_adjusts_for_wide_kind
cargo test --manifest-path roastty/Cargo.toml set_cursor
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/cursor_renderer_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/renderer_knobs_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/renderer_control_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/font_grid_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/osc7_edge_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/osc7_pwd_normalization_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/title_pwd_fallback_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/scrollback_byte_limit_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/shell_startup_rewrite_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/surface_title_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
cargo fmt --manifest-path roastty/Cargo.toml --check
python3 -m py_compile issues/0805-roastty-ghostty-parity/cursor_renderer_runtime_parity.py issues/0805-roastty-ghostty-parity/config_runtime_inventory.py issues/0805-roastty-ghostty-parity/renderer_knobs_runtime_parity.py issues/0805-roastty-ghostty-parity/renderer_control_runtime_parity.py
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/134-cursor-renderer-runtime.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

The regenerated inventory reported:

```text
runtime_rows=43
oracle_complete=36
closed=38
audit_covered=0
incomplete=5
gap=5
cfg223=Gap
```

## Conclusion

The deterministic selected-cursor renderer data slice is now guarded while the
remaining renderer gap stays honest. CFG-223 remains `Gap` with five runtime
gaps, and `RUNTIME-008B2B` owns the remaining renderer visual work, including
password/preedit active-path cursor priority and actual GUI cursor pixels.

## Completion Review

**Reviewer:** Codex adversarial subagent with fresh context.

**Verdict:** Approved.

The reviewer reported no required findings. One nit noted that the result
verification block omitted the already-run Prettier and `git diff --check`
hygiene commands; this file was updated to record them.
