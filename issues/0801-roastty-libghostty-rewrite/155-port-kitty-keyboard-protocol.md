# Experiment 155: Port Kitty Keyboard Protocol

## Description

Port Ghostty's Kitty keyboard protocol runtime commands for the active screen.

Roastty already has the lower-level state type in
`roastty/src/terminal/kitty.rs`:

- `KeyFlags`
- `KeySetMode`
- `KeyFlagStack`

Roastty also already has formatter support for emitting active Kitty keyboard
state through `ScreenFormatterExtra::kitty_keyboard(true)`. What is missing is
the parser/runtime path that lets terminal input mutate and query that stack.

Upstream Ghostty source references:

- `vendor/ghostty/src/terminal/stream.zig`
  - parses `CSI ? u` as `kitty_keyboard_query`;
  - parses `CSI > Ps u` as `kitty_keyboard_push`;
  - parses `CSI < Ps u` as `kitty_keyboard_pop`;
  - parses `CSI = Ps ; Pm u` as set/or/not Kitty keyboard flags.
- `vendor/ghostty/src/terminal/stream_terminal.zig`
  - applies push/pop/set/or/not to `terminal.screens.active.kitty_keyboard`;
  - responds to query with `CSI ? {flags} u`.
- `vendor/ghostty/src/terminal/kitty.zig`
  - defines the five-bit flag mapping.
- `vendor/ghostty/src/terminal/formatter.zig`
  - emits Kitty keyboard state via `CSI = {flags} u`.

This experiment should make the Kitty keyboard protocol functional at the
terminal stream layer, scoped to state/query behavior only. It should not add
key event encoding, platform input handling, public ABI, app integration, or
mouse/keyboard frontend behavior.

## Changes

1. Make Kitty keyboard state runtime-accessible.
   - Remove test-only gates from `KeyFlagStack::set`, `push`, and `pop` so the
     terminal runtime can use the existing implementation.
   - Keep `KeyFlags::from_int` private unless the stream parser needs a narrow
     helper such as `KeyFlags::from_protocol_int(u16) -> Option<KeyFlags>` that
     rejects values outside the five-bit Kitty flag range.

2. Extend stream actions.
   - Add actions for:
     - Kitty keyboard query;
     - Kitty keyboard push with flags;
     - Kitty keyboard pop with count;
     - Kitty keyboard set;
     - Kitty keyboard set-or;
     - Kitty keyboard set-not.
   - Keep these actions internal to `roastty/src/terminal/stream.rs`; do not add
     public ABI or app-visible API.

3. Parse Kitty keyboard CSI `u` forms.
   - Preserve existing `CSI u` restore-cursor behavior with no intermediates.
   - Parse `CSI ? u` as query.
   - Parse `CSI > Ps u` as push. Missing parameter defaults to `0`; one
     parameter must fit in the five-bit flag range; extra parameters are invalid
     and ignored.
   - Parse `CSI < Ps u` as pop. Missing parameter defaults to `1`; extra
     parameters are invalid and ignored.
   - Parse `CSI = Ps ; Pm u` as set/or/not. Missing `Ps` defaults to `0`;
     missing `Pm` defaults to `1`; `Pm=1` means set, `Pm=2` means OR, and `Pm=3`
     means NOT. Invalid flags or mode values are ignored.
   - Do not reinterpret unrelated CSI `u` forms as Kitty keyboard commands.

4. Apply runtime behavior on the active screen.
   - Query writes `\x1b[?{flags}u` to the PTY response buffer, where `{flags}`
     is the active screen's current `KeyFlags::int()`.
   - Push, pop, set, set-or, and set-not mutate only the active screen's
     `kitty_keyboard` stack.
   - Primary and alternate screen Kitty keyboard states remain isolated because
     the state belongs to `Screen`.
   - RIS remains a full reset and clears Kitty keyboard state through
     `Screen::reset()`.

5. Preserve existing behavior.
   - `CSI u` with no intermediates must still restore cursor.
   - Existing formatter Kitty keyboard extra behavior must remain unchanged.
   - Existing parser invalid-form behavior must remain non-mutating.

## Verification

Run:

```bash
cargo fmt
cargo test -p roastty kitty_keyboard
cargo test -p roastty save_cursor
cargo test -p roastty ris
cargo test -p roastty
```

Required test coverage:

- Stream parser tests:
  - `CSI ? u` dispatches query.
  - `CSI > u` defaults push flags to `0`.
  - `CSI > 3 u` dispatches push with flags `3`.
  - `CSI < u` defaults pop count to `1`.
  - `CSI < 2 u` dispatches pop count `2`.
  - `CSI = u` defaults to set flags `0`.
  - `CSI = 3 u`, `CSI = 3 ; 1 u`, `CSI = 3 ; 2 u`, and `CSI = 3 ; 3 u` dispatch
    set/set-or/set-not correctly.
  - Invalid flag values above the five-bit range, invalid set modes, and extra
    parameters are ignored without dispatching an action.
  - `CSI u` remains restore-cursor.

- Runtime tests:
  - Query on default state writes `\x1b[?0u`.
  - Push changes the active flags and query reports the pushed value.
  - Pop restores the previous stack value; oversized pop resets to disabled.
  - Multiple pushes followed by `CSI < 2 u` pop two stack entries and restore
    the expected earlier value.
  - Set replaces, set-or ORs, and set-not clears bits from the current flags.
  - Primary and alternate screens maintain independent Kitty keyboard stacks.
  - RIS clears Kitty keyboard state on the active screen and on future alternate
    entries.

- Regression tests:
  - Existing formatter Kitty keyboard extra tests still pass.
  - Existing save/restore cursor tests still pass.
  - Existing RIS tests still pass.
  - No public ABI, app integration, PTY process, renderer, browser overlay,
    mouse input, or key event encoding behavior changes.

## Non-Negotiable Invariants

- Do not add key event encoding in this experiment.
- Do not add platform keyboard translation or macOS input integration.
- Do not add public ABI, app API, renderer behavior, PTY process behavior, or
  browser overlay behavior.
- Do not add mouse protocol behavior.
- Do not add Kitty graphics behavior.
- Do not add Linux or other non-macOS platform paths.
- Do not add `ghostty_*` names. Use Roastty names except when citing upstream
  Ghostty source paths or behavior.
- Run `cargo fmt` and accept its output.

## Failure Criteria

This experiment fails if:

- Kitty keyboard CSI forms parse but do not mutate/query the active screen;
- `CSI u` restore-cursor behavior regresses;
- invalid Kitty keyboard forms mutate state or write query responses;
- query responses use the wrong VT format;
- push/pop/set/or/not behavior diverges from `KeyFlagStack` semantics;
- primary and alternate screen Kitty keyboard state leaks across screens;
- RIS leaves stale Kitty keyboard state;
- the patch adds key event encoding, platform input handling, public ABI,
  renderer/app behavior, PTY process behavior, browser overlay behavior, mouse
  protocol behavior, Kitty graphics, or non-macOS platform paths.
