# Experiment 110: Port Escape Tab Set

## Description

Continue the stream/action port by adding horizontal tab set (`HTS`) for the
simple escape sequence `ESC H`.

Roastty already has tabstop storage (Experiment 100) and basic horizontal tab
movement that reads `Terminal.tabstops` (Experiment 108). The stream parser
currently drops every non-CSI escape final, so `ESC H` cannot set a tabstop yet.
In Ghostty, `ESC H` emits the `tab_set` action, and `Terminal.tabSet()` sets a
tabstop at the current cursor column.

This experiment ports only the simple escape path:

- parse `ESC H` as a private `TabSet` stream action;
- set a tabstop at the current active cursor column;
- leave cursor position, screen cells, dirty state, and pending wrap unchanged;
- make subsequent `HT` use the newly set tabstop.

CSI cursor tabulation control (`CSI W` / `CSI 0 W`), tab clear/reset, tab set
via other parser paths, horizontal-tab-back, margins, origin mode, and public
API/ABI remain separate experiments.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/stream.zig` for `ESC H` mapping to
     `.tab_set`.
   - Use `vendor/ghostty/src/termio/stream_handler.zig` for `.tab_set` routing
     to `Terminal.tabSet()`.
   - Use `vendor/ghostty/src/terminal/Terminal.zig` for `tabSet()`.
   - Note but do not port the CSI `W` / `0W` tab-set path in this experiment.
   - Do not modify `vendor/ghostty/`.

2. Extend the stream action surface privately.
   - Add private `Action::TabSet`.
   - In escape state, dispatch `ESC H` as `TabSet`.
   - Consume the `H` escape final and set the parser back to ground state before
     invoking the generic handler, so a handler error cannot leave the parser
     stuck in escape state.
   - Keep unsupported non-CSI escape finals ignored.
   - Keep CSI parsing unchanged.
   - Preserve pending invalid UTF-8 behavior: if an incomplete invalid UTF-8
     sequence is interrupted by `ESC H`, dispatch `U+FFFD` before entering the
     escape sequence, and then dispatch `TabSet`.

3. Wire terminal stream handling to mutable tabstop state.
   - Extend `Terminal::next_slice()` so `TerminalStreamHandler` can mutably set
     `Terminal.tabstops` while also mutating the active `Screen`.
   - Add a private production helper for reading the active cursor column or for
     applying tab-set directly through `Screen`, for example
     `Screen::cursor_x_basic()` or
     `Screen::tab_set_basic(&self, &mut Tabstops)`. Keep this helper private to
     the terminal module; do not expose public API or ABI.
   - Do not clone tabstop state.
   - Do not move tabstop ownership out of `Terminal`.
   - Do not add public API or ABI.

4. Add tab-set behavior.
   - Add a private handler path for `Action::TabSet`.
   - Set `Terminal.tabstops` at the active cursor's current column.
   - Leave cursor position unchanged.
   - Leave `pending_wrap` unchanged.
   - Do not modify cells.
   - Do not dirty rows.

5. Add tests.
   - Stream parser tests:
     - `A\x1bHB` dispatches print, tab-set, print in order;
     - unsupported direct escape finals still do not leak printable bytes;
     - `ESC H` split across `next_slice()` calls still dispatches `TabSet`;
     - pending invalid UTF-8 dispatches `U+FFFD` before split or same-slice
       `ESC H`;
     - CSI `W` and `CSI 0 W` remain unsupported in this experiment and do not
       dispatch `TabSet`.
   - Terminal tests:
     - after clearing default tabstops, printing three cells and receiving
       `ESC H` sets a tabstop at column 3;
     - the new tabstop is observable through the existing test helper;
     - after setting column 3 through `ESC H`, `1\tA` writes `A` at column 3,
       proving `HT` uses the newly set stream tabstop;
     - `ESC H` leaves cursor position unchanged;
     - `ESC H` leaves pending wrap unchanged, including at the right edge;
     - `ESC H` does not dirty rows or modify cells by itself;
     - split-feed `ESC H` works when `ESC` and `H` arrive in separate
       `next_slice()` calls.
   - Existing printable, pending-wrap, wrap-scroll, LF/CR, VT/FF, backspace,
     horizontal-tab, formatter, PageList, and stream tests must keep passing.

6. Verify.
   - Run:

     ```bash
     cargo fmt
     cargo test -p roastty stream
     cargo test -p roastty terminal_formatter
     cargo test -p roastty terminal::terminal
     cargo test -p roastty screen_formatter
     cargo test -p roastty page_string
     cargo test -p roastty terminal::page_list
     cargo test -p roastty
     ```

   - `cargo fmt` output must be accepted as-is.

7. Independent review.
   - Before implementation, get Codex review of this experiment design.
   - Fix all real design findings before implementation.
   - Record the design-review outcome in this experiment file before
     implementation.
   - Commit the approved design before implementation.
   - After implementation and verification, get Codex review of the completed
     result.
   - Fix all real result findings before proceeding.
   - Commit the approved result separately from the design commit.

8. Record the result.
   - Append `## Result` and `## Conclusion` to this file.
   - Include:
     - stream action changes;
     - tabstop borrow/mutation wiring;
     - `ESC H` tab-set behavior;
     - interaction with subsequent `HT`;
     - pending-wrap and dirty-state behavior;
     - what remains deferred;
     - verification command output summary;
     - Codex design-review outcome;
     - Codex result-review outcome.
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- the stream parser dispatches `ESC H` as `Action::TabSet`;
- split-feed `ESC H` dispatches the same action;
- unsupported escape finals remain ignored and do not leak bytes;
- CSI `W` and `CSI 0 W` remain unsupported in this experiment;
- pending invalid UTF-8 emits `U+FFFD` before `ESC H`;
- `ESC H` sets a tabstop at the current active cursor column;
- subsequent `HT` can use the tabstop set by `ESC H`;
- `ESC H` leaves cursor position and pending wrap unchanged;
- `ESC H` does not dirty rows or modify cells by itself;
- no CSI tab-set path, tab clear/reset, horizontal-tab-back, margins, origin
  mode, no-scrollback rotation, styles, hyperlinks, wide/Unicode handling,
  public API, or public ABI are added;
- `cargo fmt` and the listed tests pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- `ESC H` dispatches correctly, but setting `Terminal.tabstops` requires a
  larger ownership refactor than expected.

The experiment fails if:

- `ESC H` remains silently ignored;
- `ESC H` leaks `H` as printable text;
- `ESC H` sets the wrong column;
- `ESC H` moves the cursor, clears pending wrap, dirties rows, or modifies
  cells;
- CSI `W` behavior is added without a separate reviewed experiment;
- public API or ABI changes are added.

## Design Review

Codex reviewed this design before implementation.

Initial review artifacts:

- Prompt: `logs/codex-review/20260601-020742-057089-prompt.md`
- Result: `logs/codex-review/20260601-020742-057089-last-message.md`

Codex found two real design issues: the design needed to specify a private
production helper for accessing the active cursor column without public API/ABI
drift, and the parser-state wording needed to require returning to ground before
invoking the generic handler for `TabSet`. The design was updated for both.

Clean design re-review artifacts:

- Prompt: `logs/codex-review/20260601-020953-678362-prompt.md`
- Result: `logs/codex-review/20260601-020953-678362-last-message.md`

Codex found no remaining blockers and approved implementation.
