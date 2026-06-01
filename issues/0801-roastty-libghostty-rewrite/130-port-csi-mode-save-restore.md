# Experiment 130: Port CSI Mode Save and Restore

## Description

Port Ghostty's DEC-private mode save and restore commands:

- `CSI ? ... s` saves one or more DEC-private modes;
- `CSI ? ... r` restores one or more DEC-private modes.

Experiment 128 made mode set/reset reachable from stream input, and Experiment
129 made the basic insert, linefeed, and wraparound print effects observable.
Roastty already has upstream-derived `ModeState::save()` and
`ModeState::restore()` storage from the earlier mode-state experiments. This
experiment wires real stream input into that storage and applies restore side
effects through the same current-core helper used by `CSI ? ... h/l`.

This is not a mode-request experiment. Do not implement DECRQM
(`CSI ? ... $ p`), mode report replies, device reports, SGR, OSC, DCS,
alternate-screen switching, save/restore cursor side effects, DECCOLM resize,
mouse encoding, keypad behavior, public ABI, or non-macOS behavior here.

Important upstream shape:

- plain `CSI s` remains the existing ambiguous save-cursor / left-right-margin
  path in upstream Ghostty;
- plain `CSI r` remains top/bottom margin in upstream Ghostty;
- only `?` private `CSI ? ... s` and `CSI ? ... r` save/restore modes;
- save/restore mode parsing uses DEC mode numbers only, not ANSI mode numbers;
- restore first loads the saved boolean, then calls the same mode-setting path
  so restore side effects run.

Current Roastty does not yet implement the upstream plain `CSI s/r` margin and
cursor paths. This experiment must preserve current Roastty behavior for those
plain forms: unsupported/no-dispatch with no final-byte leak. Porting DECSTBM,
DECSLRM, and save-cursor ambiguity belongs to a later cursor/margin experiment.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/stream.zig` for `CSI ? ... s/r` parsing.
   - Use `vendor/ghostty/src/terminal/stream_terminal.zig` for execution
     ordering:
     - save: `terminal.modes.save(mode)`;
     - restore: `const v = terminal.modes.restore(mode); setMode(mode, v)`.
   - Use `vendor/ghostty/src/terminal/modes.zig` and Roastty's
     `roastty/src/terminal/modes.rs` table for DEC-private mode mapping.
   - Do not modify `vendor/ghostty/`.

2. Extend private stream actions.
   - Add `Action::SaveMode { mode: modes::Mode }`.
   - Add `Action::RestoreMode { mode: modes::Mode }`.
   - Keep the actions internal to the terminal module.
   - Do not add public API or ABI surface.

3. Extend CSI dispatch for final `s`.
   - Preserve current Roastty plain `CSI s` behavior exactly:
     unsupported/no-dispatch with no final-byte leak.
   - Do not port upstream's plain `CSI s` ambiguous save-cursor /
     left-right-margin behavior here.
   - For `CSI ? ... s`, parse params as DEC-private mode numbers
     (`ansi = false`) and dispatch `SaveMode` actions in parameter order.
   - Unknown DEC-private mode numbers dispatch no action and do not block later
     known modes in the same sequence.
   - Invalid private/intermediate forms dispatch no action and must not leak the
     final `s` byte as printable text.
   - Multi-param save commands may dispatch up to `CSI_PARAM_CAPACITY` ordered
     actions through the existing fixed-capacity multi-action machinery.

4. Extend CSI dispatch for final `r`.
   - Preserve current Roastty plain `CSI r` behavior exactly:
     unsupported/no-dispatch with no final-byte leak.
   - Do not port upstream's plain `CSI r` top/bottom-margin behavior here.
   - For `CSI ? ... r`, parse params as DEC-private mode numbers
     (`ansi = false`) and dispatch `RestoreMode` actions in parameter order.
   - Unknown DEC-private mode numbers dispatch no action and do not block later
     known modes in the same sequence.
   - Invalid private/intermediate forms dispatch no action and must not leak the
     final `r` byte as printable text.
   - Multi-param restore commands may dispatch up to `CSI_PARAM_CAPACITY`
     ordered actions through the existing fixed-capacity multi-action machinery.

5. Route terminal save/restore mode actions.
   - `SaveMode` calls `ModeState::save(mode)` and performs no immediate side
     effects.
   - `RestoreMode` must match upstream ordering:
     1. call `ModeState::restore(mode)` to load the saved boolean into current
        mode state;
     2. call the same current-core mode side-effect path used by set/reset with
        that restored boolean.
   - If the helper currently always writes `ModeState`, it may write the same
     restored value again. That is acceptable as long as side effects run once
     and the final mode state is the restored value.
   - Current-core restore side effects must include:
     - `Mode::Origin`: move the cursor to the restored origin-home position and
       clear pending wrap through the cursor move;
     - `Mode::EnableLeftAndRightMargin` restored to `false`: clear horizontal
       margins to full width;
     - `Mode::Wraparound`: make the restored state observable through the
       pending-wrap print behavior from Experiment 129.
   - Current-core restore side effects must not fake deferred subsystems:
     alternate-screen switching, save/restore cursor, DECCOLM resize, mouse
     encoding, keypad behavior, renderer callbacks, and non-macOS behavior stay
     out of scope.

6. Add stream parser tests.
   - `CSI ? 7 s` dispatches `SaveMode(Wraparound)`.
   - `CSI ? 7 r` dispatches `RestoreMode(Wraparound)`.
   - `CSI ? 1 ; 7 ; 2004 s` dispatches three save actions in order.
   - `CSI ? 1 ; 7 ; 2004 r` dispatches three restore actions in order.
   - Unknown modes mixed with known modes skip only the unknown entries.
   - Empty params dispatch no action if DEC mode `0` remains unknown.
   - Exactly 24 known DEC-private params dispatch 24 ordered save actions and 24
     ordered restore actions.
   - Over-capacity save/restore params do not panic, do not leak final bytes,
     and follow the existing over-capacity invalid/no-dispatch behavior.
   - Plain `CSI s` keeps current Roastty unsupported/no-dispatch/no-leak
     behavior.
   - Plain `CSI r`, `CSI 2 r`, and `CSI 1 ; 3 r` keep current Roastty
     unsupported/no-dispatch/no-leak behavior.
   - Invalid private/intermediate forms for `s/r` dispatch no action and do not
     leak the final byte as printable text.
   - Split-feed save/restore commands dispatch correctly.
   - Pending invalid UTF-8 emits `U+FFFD` before same-slice and split-feed
     save/restore commands.
   - Direct C1 CSI byte `0x9b` followed by `?7s` or `?7r` remains out of scope
     and follows current raw-C1 behavior instead of entering CSI mode.
   - Handler errors from save/restore leave the parser in ground state.
   - Multi-action dispatch stops after the first failing save/restore action.

7. Add terminal tests.
   - Saving `?7` while wraparound is enabled, resetting it, then restoring it
     re-enables the pending-wrap behavior from Experiment 129.
   - Saving `?7` while wraparound is disabled, setting it, then restoring it
     disables pending-wrap movement again.
   - Saving/restoring `?6` origin mode restores the saved boolean and moves the
     cursor to the correct restored origin-home position.
   - Restoring `?69` to `false` clears horizontal margins to full width.
   - Saving/restoring `?2004` mutates bracketed-paste mode state.
   - Do not test `?4` or `?20` as insert/linefeed mode save/restore. Insert
     (`4`) and linefeed (`20`) are ANSI modes, while upstream save/restore mode
     parsing for `CSI ? ... s/r` is DEC-private. DEC `?4` maps to slow scroll,
     and DEC `?20` is currently unknown.
   - Multi-param save and restore apply in order.
   - Unknown modes mixed with known modes skip only unknown modes.
   - Save performs no side effect until a later restore. Cover this explicitly
     with side-effect modes:
     - `CSI ? 6 s` must not move the cursor;
     - `CSI ? 69 s` must not clear horizontal margins.
   - Restoring a never-saved mode uses the existing `ModeState` saved default
     storage value and documents that behavior. In current Roastty this means
     `false`, matching the current `ModeState` data model.
   - Formatter, mode-state, basic print-mode, margin, cursor, parser, and ABI
     tests keep passing.

8. Verify.
   - Run:

     ```bash
     cargo fmt
     cargo test -p roastty stream_csi_mode
     cargo test -p roastty terminal_stream_csi_mode
     cargo test -p roastty terminal_stream_pending_wrap
     cargo test -p roastty terminal_stream_lf
     cargo test -p roastty terminal::modes
     cargo test -p roastty terminal::terminal
     cargo test -p roastty stream
     cargo test -p roastty terminal_formatter
     cargo test -p roastty
     ```

   - `cargo fmt` output must be accepted as-is.

9. Independent review.
   - Before implementation, get Codex review of this experiment design.
   - Fix all real design findings before implementation.
   - Record the design-review outcome in this experiment file before
     implementation.
   - Commit the approved design before implementation.
   - After implementation and verification, get Codex review of the completed
     result.
   - Fix all real result findings before proceeding.
   - Commit the approved result separately from the design commit.

10. Record the result.
    - Append `## Result` and `## Conclusion` to this file.
    - Include:
      - exact parser behavior for `CSI ? ... s/r`;
      - confirmation that plain `CSI s/r` behavior was preserved;
      - exact restore ordering and side effects;
      - intentionally deferred DECRQM, alternate-screen, cursor-save, DECCOLM,
        mouse, keypad, public ABI, and non-macOS behavior;
      - verification command output summary;
      - Codex design-review outcome;
      - Codex result-review outcome.
    - Update the Issue 801 README experiment index from `Designed` to `Pass`,
      `Partial`, or `Fail`.

## Verification

The experiment passes if:

- real `CSI ? ... s` input saves DEC-private modes into `ModeState`;
- real `CSI ? ... r` input restores DEC-private modes from `ModeState`;
- restore applies the same current-core side effects as set/reset for the
  restored value;
- `Mode::Wraparound` restore behavior is observable through existing
  pending-wrap print tests;
- origin-mode restore moves the cursor to the restored origin-home position;
- restoring left/right margin mode to false clears horizontal margins;
- plain `CSI s` and plain `CSI r` current unsupported/no-dispatch/no-leak
  behavior is unchanged;
- unknown modes are skipped without blocking later known params;
- over-capacity and invalid private/intermediate forms do not leak final bytes;
- raw C1 `0x9b` inputs for the new final bytes are not treated as CSI;
- no DECRQM, response generation, alternate-screen, cursor-save, DECCOLM, mouse,
  keypad, public ABI, or non-macOS behavior is added;
- `cargo fmt` and the listed tests pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- parser support lands but terminal restore side effects need a helper split
  before they can be implemented safely;
- save/restore works for mode state but an existing current-core side effect is
  too broad and needs a preparatory refactor;
- upstream save-default behavior proves different from Roastty's current
  `ModeState` data model and must be resolved in a separate mode-storage
  experiment.

The experiment fails if:

- `CSI ? ... s/r` is parsed as ANSI mode save/restore instead of DEC-private
  mode save/restore;
- plain `CSI s` or plain `CSI r` regresses from current Roastty
  unsupported/no-dispatch/no-leak behavior;
- restore changes stored mode state without applying required current-core side
  effects;
- save performs immediate terminal side effects;
- invalid sequences leak final `s/r` bytes as printable text;
- the implementation adds deferred response, alternate-screen, cursor-save,
  DECCOLM, mouse, keypad, public ABI, or non-macOS behavior.

## Design Review

Codex reviewed the initial design and found two blocking issues and two medium
coverage gaps: `logs/codex-review/20260601-070052-824452-last-message.md`.

The design was updated to:

- preserve current Roastty plain `CSI s/r` behavior as
  unsupported/no-dispatch/no-leak instead of assuming upstream's plain
  save-cursor and margin commands already exist;
- remove ANSI insert (`4`) and linefeed (`20`) from this DEC-private
  save/restore experiment, and document that DEC `?4` is slow scroll while DEC
  `?20` is currently unknown;
- require explicit no-side-effect save tests for origin mode and left/right
  margin mode;
- require raw C1 `0x9b` coverage for the new `s/r` final bytes.

Codex re-reviewed the updated design and found no remaining blocking design
issues: `logs/codex-review/20260601-070435-251268-last-message.md`.

The design is approved for implementation.
