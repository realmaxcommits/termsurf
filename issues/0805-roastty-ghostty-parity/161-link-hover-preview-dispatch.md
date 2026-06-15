# Experiment 161: Link hover preview dispatch

## Description

Experiment 160 proved link preview configuration predicates and link-specific
context-menu selection, but it deliberately left Ghostty's runtime
`mouse_over_link` hover-preview dispatch as a CFG-223 gap. This experiment tests
and, if needed, implements the surface-side dispatch that Ghostty performs when
the mouse moves over or away from a link.

Pinned Ghostty commit `2c62d182cec246764ff725096a70b9ef44996f7f` dispatches two
runtime actions from `Surface.mouseRefreshLinks`:

- `mouse_shape = pointer` while the cursor is over a link, and a reset to the
  terminal mouse shape when leaving the link.
- `mouse_over_link = url` when the link kind is preview-enabled by
  `link-previews`, and `mouse_over_link = ""` when leaving a previously hovered
  link.

The scope is intentionally limited to deterministic Roastty runtime dispatch.
Native preview window display, native context/menu display, real pointer pixels,
and OS URL-opening flows remain separate GUI/OS proof.

## Changes

- `roastty/src/lib.rs`
  - Audit `roastty_surface_mouse_pos` and the surrounding mouse state to confirm
    whether link hover dispatch already exists.
  - If missing, add a small surface helper that runs after valid mouse-position
    updates, detects the link under the mouse using the existing OSC8 and regex
    link-selection paths, and dispatches `ROASTTY_ACTION_MOUSE_SHAPE` plus
    `ROASTTY_ACTION_MOUSE_OVER_LINK` with Ghostty-compatible preview gating.
  - Track previous hover state and viewport cell according to Ghostty's refresh
    rules: refresh when the cursor was previously over a link, when no previous
    link cell exists, or when the viewport cell changed; do not make
    duplicate-suppression broader than pinned Ghostty.
  - Gate hover refresh the same way as pinned Ghostty: run it only when mouse
    reporting is off, or when shift overrides mouse reporting because the
    terminal is not shift-capturing.
  - Suppress link hover while the left button is pressed and the cursor has
    moved away from the original click cell, matching Ghostty's drag behavior.
  - Clear hover state when moving off-link or out of the viewport.
  - Add focused unit tests for regular-link preview gating, OSC8 preview gating,
    pointer/reset dispatch, clear dispatch on leave, out-of-viewport clearing,
    repeat dispatch while the mouse remains over a previously hovered link,
    normal mouse-reporting suppression, shift-override mouse-reporting refresh,
    and left-click drag suppression.
- `issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py`
  - Add a cheap regression guard that verifies the Ghostty anchors, Roastty
    dispatch implementation, test names, and inventory row split are present.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Split the proven hover-preview dispatch slice out of
    `RUNTIME-012B2B2B2B2B2`.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Update the experiment status and Learnings if this experiment proves a
    reusable dispatch pattern.

## Verification

- Run the focused Rust tests:

  ```bash
  cargo test --manifest-path roastty/Cargo.toml link_hover_preview_dispatch -- --test-threads=1
  ```

- Run the new parity guard:

  ```bash
  PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py
  ```

- Regenerate and validate the config runtime inventory:

  ```bash
  PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
  ```

The experiment passes if Roastty dispatches the same deterministic surface-level
hover actions as pinned Ghostty for regular and OSC8 links, follows Ghostty's
refresh and mouse-reporting gates, suppresses hover during left-click drag,
clears hover state when leaving a link, and the inventory has a new complete row
for runtime link-hover preview dispatch while leaving native GUI/OS effects in
the remaining gap row.

## Design Review

**Reviewer:** Euler the 2nd (`019eca87-3987-7af1-9d17-67bdc7f3cb4a`)

**Result:** Blocked

The first review found real design issues:

- The plan incorrectly required broad duplicate suppression for unchanged links.
  Pinned Ghostty refreshes when the cursor was previously over a link so it can
  update changed text under a stationary cursor.
- The plan missed Ghostty's mouse-reporting gate: hover refresh only runs when
  mouse reporting is off, or when shift overrides mouse reporting because the
  terminal is not shift-capturing.
- The plan missed Ghostty's left-click drag suppression, where moving away from
  the original click cell while the left button is pressed suppresses link
  hover.

The design has been updated to make those semantics required acceptance
criteria.

**Re-review Result:** Approved

Euler the 2nd re-reviewed the revised design and approved it with no blocking
findings. The reviewer confirmed the design now requires Ghostty's refresh
semantics, mouse-reporting/shift-override gate, left-click drag suppression, and
keeps the scope limited to deterministic runtime dispatch without overclaiming
native GUI/OS proof.
