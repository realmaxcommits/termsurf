+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 58: Phase F — selection behavior config

## Description

Experiments 54-57 made the first font and clipboard config slices app-facing.
The next narrow Phase-F slice is the remaining selection behavior pair adjacent
to the already represented selection colors and `selection-clear-on-copy`:

- `selection-clear-on-typing`
- `selection-word-chars`

Roastty already has most lower-level machinery for this slice:
`SelectionWordChars` exists as a parsed config leaf type, the terminal selection
APIs accept custom word-boundary codepoint slices, and text/mouse input paths
already clear selection in some hardcoded cases. This experiment routes the two
fields through aggregate config and connects them to the existing app/surface
paths.

This experiment intentionally excludes unrelated selection, mouse, cursor, and
clipboard options; it also does not change the C ABI selection helpers except
where existing app/surface behavior needs to pass configured word boundaries.

## Changes

- `roastty/src/config/mod.rs`
  - Add `selection-clear-on-typing = true` to `Config`.
  - Add `selection-word-chars = SelectionWordChars::default()` to `Config`.
  - Route both fields through `Config::set`, `format_config`, CLI/file loading,
    clone/equality, and diagnostics.
  - Preserve the local formatter order around the existing selection fields:
    selection colors, `selection-clear-on-typing`, then later the local
    clipboard/copy selection group containing `selection-clear-on-copy`.
  - Add aggregate config tests for defaults, parser routing, formatter order,
    CLI loading, and file loading. Keep the existing leaf `SelectionWordChars`
    parser tests as the syntax oracle.
- `roastty/src/lib.rs`
  - Store the configured selection behavior on `Surface` or read it from the
    existing parsed app config snapshot, following the least invasive local
    pattern.
  - Refresh any cached surface selection behavior through the same config update
    paths used by Experiment 57 (`roastty_app_update_config` and
    `roastty_surface_update_config`).
  - Pass `selection-word-chars` into mouse-driven word selection paths
    (double-click word selection, drag-extending word selection, autoscroll
    ticks, and deep press if locally represented), matching upstream
    `Surface.zig`'s use of `self.config.selection_word_chars`.
  - Pass `selection-word-chars` into app-facing quicklook word selection
    (`quicklook_word_text` / `roastty_surface_quicklook_word`), matching
    upstream `apprt/embedded.zig`'s call that supplies
    `surface.config.selection_word_chars`.
  - Use `selection-clear-on-typing` to gate selection clearing for real text
    input paths. When false, typed text and ordinary key input should not clear
    the active selection, but Escape remains an upstream exception that clears
    selection even when `selection-clear-on-typing = false`. Preedit/composition
    start may still need upstream parity scrutiny before changing behavior, so
    any retained hardcoded clear must be documented and tested.
  - Do not affect copy-to-clipboard clearing; `selection-clear-on-copy` remains
    the field for that behavior.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, add any durable operating note for selection behavior
    config.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/config/mod.rs roastty/src/lib.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/58-selection-behavior-config.md`
- Run targeted tests:
  - `cargo test -p roastty selection_behavior_config`
  - `cargo test -p roastty config_format_config`
  - `cargo test -p roastty double_click_word_triple_click_line`
  - `cargo test -p roastty mouse_drag_selects`
  - `cargo test -p roastty surface_key`
  - `cargo test -p roastty surface_text`
  - `cargo test -p roastty surface_preedit`
  - `cargo test -p roastty surface_quicklook_word`
  - `cargo test -p roastty app_and_surface_update_config`
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the two selection behavior fields are represented on `Config`,
round-trip through config loading/formatting, refresh through app/surface config
updates, and affect only the intended selection word-boundary and typing-clear
paths; targeted and full tests pass.

**Partial** = config representation and one runtime behavior land, but another
behavior exposes a bounded missing prerequisite in text/preedit or mouse gesture
plumbing; record the exact hardcoded behavior left behind.

**Fail** = current selection/input ownership cannot safely route these fields
without a broader selection-state refactor.

## Design Review

Reviewed by Codex adversarial reviewer (`Franklin`,
`019eb32b-050f-7a32-b4ad-9d4c8813f216`).

**Initial verdict:** Changes required.

- **Required:** The original plan missed app-facing quicklook word selection.
  Upstream passes `surface.config.selection_word_chars` into
  `ghostty_surface_quicklook_word`, while local quicklook currently falls back
  to default word boundaries.
- **Required:** The original `selection-clear-on-typing` plan did not call out
  the upstream Escape exception. Escape must still clear selection even when
  `selection-clear-on-typing = false`.

Fix:

- Added quicklook word selection to the planned `selection-word-chars` runtime
  routing and verification.
- Added the Escape exception to the planned `selection-clear-on-typing` runtime
  behavior and verification.

**Final verdict:** Approved.

No findings.

## Result

**Result:** Pass

Implemented the two Phase-F selection behavior fields:

- `selection-clear-on-typing` is now represented on aggregate `Config`, defaults
  to `true`, loads through CLI/config files, formats with the local selection
  group, and refreshes cached `Surface` behavior through app and surface config
  update paths.
- `selection-word-chars` is now represented on aggregate `Config`, defaults to
  the upstream word-boundary set, loads/formats through the existing
  `SelectionWordChars` parser, and refreshes cached `Surface` word-boundary
  state through config updates.
- Mouse word selection and quicklook word selection now pass configured
  word-boundary codepoints instead of falling back to terminal defaults.
- Text entry, raw text entry, key input, and preedit state changes now honor
  `selection-clear-on-typing`; Escape remains the explicit exception and clears
  selection even when clear-on-typing is disabled.

Verification passed:

- `cargo fmt -- roastty/src/config/mod.rs roastty/src/lib.rs`
- `cargo test -p roastty selection_behavior_config`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty double_click_word`
- `cargo test -p roastty mouse_drag_selects`
- `cargo test -p roastty surface_key`
- `cargo test -p roastty surface_text`
- `cargo test -p roastty surface_preedit`
- `cargo test -p roastty surface_quicklook_word`
- `cargo test -p roastty app_and_surface_update_config`
- `cargo test -p roastty` — 4474 unit tests passed, 1 ABI harness test passed, 0
  doc-tests; the ABI harness still prints the pre-existing enum-cast warnings.

## Conclusion

The selection behavior config slice is now app-facing and runtime-visible. The
critical review findings from the design gate are covered by regression tests:
quicklook uses configured word boundaries, and Escape keeps clearing selection
when `selection-clear-on-typing = false`. The next Phase-F slice can continue
with another small config group.

## Completion Review

Reviewed by Codex-native adversarial reviewer (`Huygens`,
`019eb33a-cf6e-7402-a959-add518d6494f`) with fresh context.

**Verdict:** Approved.

- **Nit:** `SelectionWordChars` had a stale comment saying `formatEntry` would
  be ported later even though formatting is now implemented.

Fix:

- Updated the stale comment to describe only the current invariant:
  word-boundary codepoints always start with the null codepoint.
