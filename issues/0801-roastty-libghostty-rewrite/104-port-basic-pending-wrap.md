# Experiment 104: Port Basic Pending Wrap

## Description

Extend Experiment 103's basic print mutation with the next upstream
`Terminal.print()` behavior: pending wrap for one-cell printable characters.

Experiment 103 deliberately rejected right-edge input so it could establish the
stream-to-terminal mutation path without implementing wrap semantics. Upstream
Ghostty does not reject the right edge. For a one-cell character at the right
limit, it writes that character, leaves the cursor at the right edge, and sets a
pending-wrap flag. The next printable character performs the wrap first, marks
the previous row as wrapped, moves to the next row's left edge, and then prints.

This experiment ports only the no-scroll version of that behavior for the active
screen's full width. It should be enough to mirror upstream's
`Terminal: input with basic wraparound` test for a 5-column, multi-row terminal.
Scrolling, margins, insert mode, wide characters, Unicode width, and managed
cell cleanup remain deferred.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/Terminal.zig` for:
     - `print()` pending-wrap behavior;
     - `printWrap()`;
     - `Terminal: input with basic wraparound`;
     - `Terminal: input with basic wraparound dirty`.
   - Do not modify `vendor/ghostty/`.

2. Add pending-wrap state to the Rust screen cursor.
   - Add a private `pending_wrap: bool` field to `ScreenCursor`.
   - Default it to `false`.
   - Add narrow test-only accessors if needed to verify cursor state.
   - Do not add public API or ABI.

3. Update the basic print path.
   - Before writing a supported one-cell printable character, if `pending_wrap`
     is true:
     - mark the current row as wrapped;
     - move the cursor to the next row at `x = 0`;
     - mark the destination row as a wrap continuation;
     - clear `pending_wrap`;
     - mark both the old row and new row dirty as needed to match upstream's
       cursor-moved/redraw expectations.
   - After writing a supported one-cell printable character:
     - if `cursor.x` is at the right edge, keep the cursor there and set
       `pending_wrap = true`;
     - otherwise advance `cursor.x` by one and keep `pending_wrap = false`.
   - This replaces Experiment 103's `RightEdgeUnsupported` behavior for ordinary
     supported one-cell characters when there is room to wrap later.

4. Keep scrolling explicitly out of scope.
   - If `pending_wrap` is true and the cursor is already on the bottom row,
     return a private `ScrollUnsupported`-style error before writing the next
     printable character.
   - The bottom-row error must preserve state exactly: keep `pending_wrap` true,
     keep the cursor position unchanged, keep existing cells unchanged, and do
     not mark/clear row wrap metadata as part of the failed operation.
   - Do not grow `PageList`, scroll, prune history, or modify viewport state in
     this experiment.
   - Do not implement margins or origin mode; use the full screen width and left
     edge.

5. Preserve existing safety boundaries.
   - Keep Experiment 103's managed-cell guard.
   - Keep explicit private errors for unsupported non-ASCII codepoints other
     than `U+FFFD`.
   - Do not add Unicode width tables, wide characters, zero-width characters,
     grapheme clustering, charsets, styles, hyperlinks, semantic prompt state,
     insert mode, CSI, OSC, DCS, APC, PTY IO, public API, or public ABI.

6. Add or update tests.
   - Add tests for:
     - writing exactly the terminal width fills the first row, leaves the cursor
       at the last column, and sets pending wrap;
     - the next printable byte wraps to the next row before writing;
     - the formatted plain text for `helloworldabc12` on a 5-column terminal is
       `hello\nworld\nabc12`, matching the upstream basic wraparound test;
     - final cursor state for that case is row 2, column 4, pending wrap true;
     - the first and second rows are marked wrapped, and rows 1 and 2 are marked
       wrap-continuation, so formatters can treat them as soft wraps;
     - the existing formatter still produces hard-line plain output
       `hello\nworld\nabc12` for the default formatter path;
     - the existing unwrap formatter option joins the soft-wrapped rows as
       `helloworldabc12`;
     - dirty state covers both the old right-edge row and the new row after a
       pending wrap, verified by clearing dirty state after the right-edge fill
       and before triggering the wrap;
     - attempting to print after pending wrap on the bottom row returns the
       private scroll-unsupported error, does not write the next character, and
       preserves cursor position, pending-wrap state, cell contents, and row
       wrap metadata;
     - unsupported non-ASCII and managed-cell errors from Experiment 103 still
       behave as before.
   - Existing stream, formatter, screen formatter, page string, and PageList
     tests must keep passing.

7. Verify.
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

8. Independent review.
   - Before implementation, get Codex review of this experiment design.
   - Fix all real design findings before implementation.
   - Record the design-review outcome in this experiment file before
     implementation.
   - After implementation and verification, get Codex review of the completed
     result.
   - Fix all real result findings before proceeding.

9. Record the result.
   - Append `## Result` and `## Conclusion` to this file.
   - Include:
     - pending-wrap state shape;
     - right-edge behavior;
     - wrap-before-next-print behavior;
     - dirty/wrap-row behavior;
     - bottom-row scroll-unsupported behavior;
     - what remains deferred from upstream `Terminal.print()` / `printWrap()`;
     - verification command output summary;
     - Codex design-review outcome;
     - Codex result-review outcome.
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- supported one-cell printable characters at the right edge write the cell and
  set pending wrap instead of returning `RightEdgeUnsupported`;
- the next supported printable character performs the pending wrap before
  writing;
- `helloworldabc12` on a 5-column terminal formats as `hello\nworld\nabc12`;
- final cursor state for that case is row 2, column 4, pending wrap true;
- old-row wrap metadata, new-row wrap-continuation metadata, default hard-line
  formatting, unwrap soft-wrap formatting, and dirty state are observable and
  tested;
- bottom-row pending wrap returns a private scroll-unsupported error without
  writing the next character or partially mutating cursor, pending-wrap,
  existing cells, or row wrap metadata;
- Experiment 103's unsupported non-ASCII and managed-cell protections remain;
- no scrolling, margins, insert mode, Unicode width, wide characters, zero-width
  characters, grapheme clustering, charsets, styles, hyperlinks, semantic prompt
  state, CSI, OSC, DCS, APC, PTY IO, public API, or public ABI are added;
- `cargo fmt` and the listed tests pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- row wrap metadata or dirty state cannot be updated safely without first adding
  a broader `Screen` cursor/pin abstraction, and that prerequisite is identified
  precisely.

The experiment fails if:

- right-edge printing still rejects ordinary supported one-cell input when the
  terminal has another row available;
- the implementation silently scrolls or grows storage in this slice;
- pending wrap state is lost across terminal feed calls;
- old-row wrap or new-row wrap-continuation metadata is missing after a
  successful pending wrap;
- soft-wrapped rows cannot be joined by the existing unwrap formatter option;
- dirty-state tests pass only because rows were already dirty before the
  pending-wrap operation;
- bottom-row scroll-unsupported errors partially mutate cursor, pending-wrap,
  cells, or row wrap metadata before returning;
- unsupported non-ASCII or managed-cell paths regress to silent writes or silent
  drops;
- public API or ABI changes are added.

## Design Review

Codex reviewed this design before implementation.

Initial review artifacts:

- Prompt: `logs/codex-review/20260601-010746-655026-prompt.md`
- Result: `logs/codex-review/20260601-010746-655026-last-message.md`

Codex found three real design gaps:

- destination rows also need wrap-continuation metadata, and formatter tests
  must cover both default hard-line output and unwrap soft-wrap output;
- dirty-state tests must clear prior dirt before triggering pending wrap, so the
  wrap/write operation itself is proven to mark both affected rows dirty;
- bottom-row scroll-unsupported errors must preserve cursor, pending-wrap, cell,
  and row metadata state without partial mutation.

All three findings were applied. A clean design re-review will be recorded
before implementation.

Clean design re-review artifacts:

- Prompt: `logs/codex-review/20260601-010954-614212-prompt.md`
- Result: `logs/codex-review/20260601-010954-614212-last-message.md`

Codex found no remaining real design findings and approved implementation.
