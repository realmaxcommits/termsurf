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

# Experiment 709: Binding Action Scroll Lines

## Description

Experiment 708 added `scroll_page_up` and `scroll_page_down` binding-action
support by applying row-delta viewport movement. Upstream Ghostty's
`performBindingAction` also supports `scroll_page_lines:<i16>`, which applies
the signed line count directly to the viewport:

- positive values scroll downwards;
- negative values scroll upwards;
- `+N` is accepted by Zig's decimal integer parser;
- empty, whitespace, malformed, or out-of-range parameters are invalid;
- the action returns `true` when performed on an attached surface.

Roastty already has the row-delta helper used by Experiment 708. This experiment
adds only the signed line-count parser and the `scroll_page_lines:<i16>`
binding-action path.

This does not implement `clear_screen`, `scroll_to_row`, `scroll_to_selection`,
fractional page scrolling, prompt jumps, search actions, clipboard actions,
cursor-key actions, full keybind storage/lookup, or app-scoped actions.

## Changes

- `roastty/src/lib.rs`
  - Add a small ASCII decimal `i16` parser that mirrors upstream
    `std.fmt.parseInt(i16, value, 10)` for this action: accept optional leading
    `+` or `-`, require at least one digit, reject whitespace/trailing bytes,
    and reject values outside the `i16` range.
  - Extend the internal parsed binding-action enum with `ScrollPageLines(i16)`.
  - Extend `parse_binding_action` to accept `scroll_page_lines:<i16>` and reject
    missing, empty, malformed, whitespace, extra-colon, and out-of-range
    parameters.
  - Add/use a surface helper that locks the active termio worker, applies the
    parsed signed row delta to the terminal viewport, and requests a render.
  - Treat a zero line count as a consumed no-op, matching a zero-delta
    interpretation.
  - Return `true` for attached parsed line-scroll actions, even when no termio
    worker exists, matching action-consumed semantics.
  - Return `false` for null or detached surfaces.
  - Keep split, close, `text:`, `csi:`, `esc:`, `reset`, top/bottom scroll, and
    page up/down semantics unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage that malformed line-scroll forms are rejected and
    representative negative, positive, and explicit-plus forms can be invoked.

- Tests in `roastty/src/lib.rs`
  - Cover invalid forms returning false: missing parameter, empty parameter,
    whitespace, malformed bytes, extra colon, and values outside the `i16`
    range.
  - Cover null and detached surfaces returning false.
  - Cover attached no-worker surfaces returning true without side effects.
  - Cover worker-backed `scroll_page_lines:-N` moving the viewport up by exactly
    `N` rows when scrollback exists.
  - Cover worker-backed `scroll_page_lines:+N` and `scroll_page_lines:N` moving
    the viewport down by exactly `N` rows.
  - Cover `scroll_page_lines:0` returning true without moving the viewport.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty binding_action -- --nocapture`
- `cargo test -p roastty scroll_page_lines -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 709 design and approved it technically. The review
confirmed that the scope is upstream-compatible: `scroll_page_lines:i16` is a
focused signed row-delta slice and preserves upstream direct viewport movement
semantics while excluding fractional, row, selection, and prompt actions.

The review also confirmed that a small ASCII `i16` parser matching
`std.fmt.parseInt(i16, value, 10)` plus reuse of
`Terminal::scroll_selection_gesture_viewport(delta)` is feasible. The proposed
tests were accepted as sufficient for malformed parameters, signed forms,
null/detached/no-worker behavior, exact worker-backed movement, zero no-op, ABI
smoke coverage, and prior-action regression coverage.

The only required fix before plan commit was workflow provenance: replacing the
pending design-review metadata, adding this design-review section, and updating
the README provenance tuple to `Codex/Codex/-`.
