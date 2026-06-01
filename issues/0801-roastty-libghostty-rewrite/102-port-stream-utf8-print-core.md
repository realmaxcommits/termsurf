# Experiment 102: Port Stream UTF-8 Print Core

## Description

Start the terminal runtime input path by porting the first narrow slice of
upstream Ghostty's `terminal/stream.zig`: UTF-8 print decoding and action
dispatch.

Experiments 90-101 completed formatter-side terminal state serialization for the
active screen and terminal-level extras. The next major subsystem is runtime
mutation: bytes from a PTY must be decoded into stream actions, then applied to
terminal state. Upstream's `stream.zig` is large, so this experiment only builds
the foundational stream/handler shape and the ground-state UTF-8 print path. It
must not implement CSI, OSC, DCS, APC, ESC, terminal mutation, PTY IO, public
API, public ABI, renderer behavior, app behavior, or UI behavior.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/stream.zig` for:
     - `Action.print`;
     - `Stream(H)` handler dispatch shape;
     - `nextSlice()` / incremental input behavior;
     - UTF-8 decoding behavior;
     - invalid UTF-8 replacement behavior.
   - Use `vendor/ghostty/src/terminal/UTF8Decoder.zig` for decoder semantics.
   - Do not modify `vendor/ghostty/`.

2. Add a private stream module.
   - Add `roastty/src/terminal/stream.rs`.
   - Export it only inside `terminal/mod.rs`.
   - Keep the module private to the crate/subsystem. Do not expose public API or
     ABI.
   - Add a private `Action` enum with at least:

     ```rust
     Action::Print { cp: char }
     ```

   - Use Rust `char` for decoded Unicode scalar values. Invalid UTF-8 must
     dispatch `U+FFFD` as a print action, matching upstream's replacement
     behavior.

3. Add a handler trait or equivalent private dispatch shape.
   - Preserve upstream's conceptual boundary: stream parsing emits actions; a
     handler receives those actions.
   - The first implementation may use a trait, callback, or small generic type,
     but it must keep parsing independent from terminal mutation.
   - Do not call into `Terminal` yet.
   - Do not add `Terminal::vt_stream()` yet.

4. Implement incremental UTF-8 print decoding.
   - `Stream::next_slice(&[u8])` processes byte slices.
   - Complete valid UTF-8 sequences dispatch one `Action::Print` per scalar.
   - ASCII printable bytes dispatch directly as print actions.
   - Split multi-byte UTF-8 sequences across calls are buffered until complete.
   - Invalid UTF-8 dispatches `U+FFFD` with upstream retry semantics:
     - if the rejecting byte is part of the invalid sequence, consume it;
     - if the rejecting byte is a new possible starter byte, emit `U+FFFD` for
       the invalid pending sequence but retry that same byte as the start of the
       next decode attempt.
   - At end-of-input, incomplete UTF-8 is not dispatched until another call
     either completes it or proves it invalid.
   - This experiment may use Rust's standard UTF-8 validation primitives rather
     than porting Ghostty's exact SIMD path. Do not add SIMD in this slice.

5. Explicitly defer escape and control handling.
   - C0/C1 control bytes other than ESC are ignored in this slice.
   - ESC (`0x1b`) starts a minimal unsupported-escape state and must not leak
     subsequent escape bytes as printable text.
   - For unsupported CSI-looking input such as `ESC [ C`, consume through the
     final byte and return to ground state without dispatching print actions.
   - For direct unsupported ESC final-byte input such as `ESC c`, consume the
     final byte and return to ground state without dispatching print actions.
   - This minimal state exists only to keep unsupported escape/control syntax
     from being misclassified as text. It is not a CSI/OSC/DCS/APC
     implementation.
   - Do not implement CSI, OSC, DCS, APC, parser state machine, modes, cursor
     movement, tab mutation, PWD mutation, keyboard mutation, screen writes, or
     terminal writes.
   - If the stream sees an unsupported escape/control byte, behavior must be
     documented in tests so later parser slices can replace it deliberately.

6. Add upstream-equivalent tests.
   - Add stream tests for:
     - ASCII text dispatches one print action per character;
     - Unicode scalar values dispatch correctly;
     - a multi-byte scalar split across `next_slice()` calls dispatches only
       after the final byte arrives;
     - invalid UTF-8 dispatches `U+FFFD`;
     - partial-invalid UTF-8 retries a rejecting starter byte instead of
       dropping it, matching upstream `UTF8Decoder.zig` behavior;
     - incomplete UTF-8 held at a slice boundary completes correctly on the next
       slice;
     - unsupported C0/C1 control bytes do not masquerade as printable text;
     - unsupported direct ESC final-byte sequences do not leak their final byte
       as printable text;
     - unsupported CSI-shaped escape sequences such as `ESC [ C` consume the
       whole unsupported sequence and do not leak `[` or `C` as printable text;
     - parser state remains usable after invalid UTF-8 and after ignored
       unsupported bytes.
   - Keep existing formatter, tabstops, modes, ScreenFormatter, PageList
     formatter, and PageList tests passing.

7. Verify.
   - Run:

     ```bash
     cargo fmt
     cargo test -p roastty stream
     cargo test -p roastty terminal_formatter
     cargo test -p roastty modes
     cargo test -p roastty tabstops
     cargo test -p roastty screen_formatter
     cargo test -p roastty styled_pin_map
     cargo test -p roastty pin_map
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
     - stream module visibility;
     - action/handler shape;
     - UTF-8 decoding and replacement behavior;
     - unsupported escape/control behavior;
     - why terminal mutation, CSI, OSC, PTY, public API, and ABI remain
       deferred;
     - verification command output summary;
     - Codex design-review outcome;
     - Codex result-review outcome.
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- `roastty/src/terminal/stream.rs` exists and is private to the terminal
  subsystem;
- stream parsing emits print actions through a private handler boundary;
- ASCII and Unicode text dispatch as print actions;
- split UTF-8 sequences are buffered across `next_slice()` calls;
- invalid UTF-8 emits `U+FFFD` with upstream retry semantics for rejecting
  starter bytes;
- unsupported control and escape behavior is explicit and tested;
- no terminal mutation, CSI parser, OSC parser, DCS parser, APC parser, PTY IO,
  public API, public ABI, renderer behavior, app behavior, or UI behavior is
  added;
- `cargo fmt`, stream tests, formatter tests, tabstops tests, modes tests,
  PageList formatter tests, PageList tests, and full `cargo test -p roastty`
  pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- UTF-8 decoding cannot be represented without first porting Ghostty's parser
  state machine or UTF-8 decoder module, and that prerequisite is identified
  precisely.

The experiment fails if:

- stream parsing is coupled directly to `Terminal` mutation in this slice;
- invalid UTF-8 is silently dropped or panics;
- split valid UTF-8 emits replacement characters before enough bytes arrive;
- invalid UTF-8 consumes a rejecting starter byte that upstream would retry;
- unsupported ESC/control bytes or bytes inside unsupported escape sequences are
  treated as normal printable text;
- public API or ABI changes are added;
- formatter or existing terminal storage behavior regresses.

## Design Review

Codex reviewed this design before implementation.

Initial review artifacts:

- Prompt: `logs/codex-review/20260601-003456-369584-prompt.md`
- Result: `logs/codex-review/20260601-003456-369584-last-message.md`

Codex found two real design gaps:

- invalid UTF-8 behavior had to specify Ghostty's retry-the-rejecting-byte
  semantics instead of only saying "resume at the next valid boundary";
- unsupported ESC/control behavior had to be concrete enough to prevent
  unsupported sequences such as `ESC [ C` from leaking `[` or `C` as printable
  text.

Both findings were applied.

Re-review artifacts:

- Prompt: `logs/codex-review/20260601-003712-140875-prompt.md`
- Result: `logs/codex-review/20260601-003712-140875-last-message.md`

Codex found no remaining blocking findings and approved implementation.
