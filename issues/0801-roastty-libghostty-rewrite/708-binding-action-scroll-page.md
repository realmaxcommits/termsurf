+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "pending"
model = "pending"
reasoning = "pending"
+++

# Experiment 708: Binding Action Scroll Page

## Description

Experiment 707 added exact `scroll_to_top` and `scroll_to_bottom` binding-action
support through a narrow terminal viewport helper path. Upstream Ghostty's
`performBindingAction` also supports parameterless page scrolling:

- `scroll_page_up` scrolls the viewport up by one visible grid height;
- `scroll_page_down` scrolls the viewport down by one visible grid height;
- both are void actions, so colon-bearing forms such as `scroll_page_up:` and
  `scroll_page_down:now` are malformed;
- both return `true` when performed on an attached surface.

Roastty already has `Terminal::scroll_selection_gesture_viewport(delta)` for
row-delta viewport scrolling and Experiment 707 added exact endpoint helpers.
This experiment adds only the page up/down binding-action path, using the
surface's current grid row count as the page size.

This does not implement `clear_screen`, `scroll_to_row`, `scroll_to_selection`,
fractional/line scroll actions, prompt jumps, search actions, clipboard actions,
cursor-key actions, full keybind storage/lookup, or app-scoped actions.

## Changes

- `roastty/src/terminal/terminal.rs`
  - Add/use a crate-local viewport delta helper if needed, or reuse
    `scroll_selection_gesture_viewport` if its semantics are already the right
    active-screen row-delta behavior.
  - Keep the helper internal; do not add public C ABI terminal functions for
    this experiment.

- `roastty/src/lib.rs`
  - Extend the internal parsed binding-action enum with `ScrollPageUp` and
    `ScrollPageDown`.
  - Extend `parse_binding_action` to accept exact `scroll_page_up` and
    `scroll_page_down` forms and reject any colon-bearing parameters.
  - Add/use a surface helper that locks the active termio worker, computes the
    page delta from `surface.size.rows`, applies a negative delta for page up
    and a positive delta for page down, and requests a render.
  - Treat a zero-row surface size conservatively as a consumed no-op rather than
    inventing a default page size in this binding-action layer.
  - Return `true` for attached parsed page-scroll actions, even when no termio
    worker exists, matching action-consumed semantics.
  - Return `false` for null or detached surfaces.
  - Keep split, close, `text:`, `csi:`, `esc:`, `reset`, `scroll_to_top`, and
    `scroll_to_bottom` semantics unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage that colon-bearing page-scroll forms are rejected
    and exact `scroll_page_up` / `scroll_page_down` can be invoked.

- Tests in `roastty/src/lib.rs`
  - Cover `scroll_page_up:`, `scroll_page_up:now`, `scroll_page_down:`, and
    `scroll_page_down:now` returning false.
  - Cover null and detached surfaces returning false for both actions.
  - Cover attached no-worker surfaces returning true without side effects.
  - Cover worker-backed `scroll_page_up` moving the viewport up by exactly the
    surface row count when scrollback exists.
  - Cover worker-backed `scroll_page_down` moving the viewport back down by
    exactly the surface row count after `scroll_page_up`.
  - Cover a zero-row attached worker-backed surface returning true without
    moving the viewport.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty binding_action -- --nocapture`
- `cargo test -p roastty scroll_page_up -- --nocapture`
- `cargo test -p roastty scroll_page_down -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 708 design and approved it technically. The review
confirmed that the slice is appropriately scoped to exact `scroll_page_up` and
`scroll_page_down`, matches upstream's negative/positive visible-grid-row
deltas, and preserves upstream void-action parsing by rejecting colon-bearing
forms.

The review also confirmed that reusing
`Terminal::scroll_selection_gesture_viewport(delta)` is feasible for the
active-screen row-delta behavior and that treating zero surface rows as a
consumed no-op matches a natural zero-delta interpretation.

The only required fix before plan commit was workflow provenance: replacing the
pending design-review metadata, adding this design-review section, and updating
the README provenance tuple to `Codex/Codex/-`.
