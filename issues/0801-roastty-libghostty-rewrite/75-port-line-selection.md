# Experiment 75: Port Line Selection

## Description

Port upstream `Screen.selectLine` into Roastty's current PageList-centered
terminal model.

Experiment 74 added word selection. Upstream double/triple-click and
word/line-drag behavior next needs content-aware line selection. Roastty still
does not have `Screen`, but the PageList row/cell iterators, selection value
type, semantic-content cell flags, row wrap flags, and default line-whitespace
codepoint table are enough to port the pure line-selection calculation now.

This experiment should add only the line-selection helper and tests. It must not
add full `SelectionGesture`, press/release state, word/line drag wiring,
semantic-output selection, Screen, Terminal, public C ABI, renderer, parser,
app, platform input, autoscroll, deep-press, or mouse event behavior.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/Screen.zig` for:
     - `SelectLine`;
     - `selectLine`;
     - `test "Screen: selectLine"`;
     - `test "Screen: selectLine across soft-wrap"`;
     - `test "Screen: selectLine across full soft-wrap"`;
     - `test "Screen: selectLine across soft-wrap ignores blank lines"`;
     - `test "Screen: selectLine disabled whitespace trimming"`;
     - `test "Screen: selectLine with scrollback"`;
     - all `selectLine` semantic-boundary tests.
   - Use existing Roastty code:
     - `roastty/src/terminal/page_list.rs`;
     - `roastty/src/terminal/selection.rs`;
     - `roastty/src/terminal/selection_codepoints.rs`;
     - existing PageList semantic highlight tests and helpers.
   - Do not modify `vendor/ghostty/`.

2. Add a private line-selection options value.
   - Preferred shape:

     ```rust
     struct SelectLineOptions<'a> {
         pin: Pin,
         whitespace: Option<&'a [u32]>,
         semantic_prompt_boundary: bool,
     }
     ```

   - The default behavior should use
     `Some(selection_codepoints::DEFAULT_LINE_WHITESPACE)` and
     `semantic_prompt_boundary = true`, matching upstream.
   - Keep the type private. Do not add public API or C ABI.

3. Add the private PageList helper.
   - Preferred shape:

     ```rust
     fn select_line(&self, options: SelectLineOptions<'_>)
         -> Option<selection::Selection>
     ```

   - Reject invalid or garbage pins with `None`.
   - Return an untracked non-rectangular selection on success.
   - Match upstream soft-wrap behavior:
     - scan left/up through prior rows while rows are soft-wrapped;
     - scan right/down through following rows until the final row in the
       soft-wrap;
     - use row `wrap` flags as the authoritative boundary.
   - Match upstream whitespace trimming:
     - when `whitespace` is `Some`, trim leading and trailing cells whose
       codepoint is in the supplied table, skipping empty cells while searching
       for the first/last non-whitespace text;
     - return `None` if the soft-wrapped line has no non-whitespace text;
     - when `whitespace` is `None`, skip trimming and select the full
       soft-wrapped row span from column 0 through the last column within the
       semantic-bounded range. This becomes the full physical soft-wrapped span
       only when semantic boundaries are disabled or absent.
   - Match upstream semantic-boundary behavior:
     - when `semantic_prompt_boundary` is true, capture the clicked cell's
       `Cell::semantic_content()` as the required state;
     - scan backward and forward only through cells with the same semantic
       content, even when rows are soft-wrapped;
     - handle mid-row semantic transitions, first-cell-of-row transitions, and
       disabled semantic boundaries like upstream;
     - do not use row `SemanticPrompt` flags for this helper. Upstream
       `selectLine` uses cell semantic content, not prompt-zone row metadata.

4. Add upstream-equivalent tests.
   - Port ordinary line-selection cases:
     - click at start/middle/end of a line;
     - click on unwritten cells beyond line text but inside the same active row;
     - empty/unwritten rows returning `None`;
     - whitespace trimming at both ends.
   - Port soft-wrap cases:
     - line spans a partial soft-wrap;
     - line spans a full-width soft-wrap;
     - blank wrapped rows are ignored by whitespace trimming;
     - hard row boundaries stop selection.
   - Port disabled-whitespace behavior:
     - `whitespace = None` selects the full soft-wrapped row span within the
       semantic-bounded range;
     - a non-wrapped row selects columns `0..cols - 1`.
     - `whitespace = None` with a mid-row or soft-wrap semantic transition still
       honors semantic boundaries unless `semantic_prompt_boundary = false`.
   - Port scrollback-relevant behavior with PageList history fixtures if the
     current PageList test helpers can represent it without adding Screen. If
     not, document the exact missing fixture and keep active/screen coordinate
     coverage for this experiment.
   - Port semantic-boundary cases:
     - prompt-to-output boundary across a soft-wrap;
     - prompt-to-input mid-row boundary;
     - input-to-output row boundary;
     - output/prompt/input mid-row boundary;
     - soft-wrap with a mid-row semantic transition on the wrapped row;
     - semantic boundary disabled selecting the whole line;
     - first-cell-of-row semantic transition;
     - all-same semantic content across soft-wraps.
   - Add Roastty-specific edge tests:
     - invalid and garbage pins return `None`;
     - a soft-wrapped line containing only whitespace/empty cells returns `None`
       when trimming is enabled;
     - selections are untracked and non-rectangular.

5. Keep scope narrow.
   - Do not add `SelectionGesture.dragSelectionLine`,
     `SelectionGesture.pressSelection`, `LineIterator`, `selectAll`, or
     selection string extraction in this experiment.
   - Do not add parser/write support merely to construct tests. Reuse existing
     PageList fixture helpers (`set_screen_text_lines`, `set_screen_row_wrap`,
     semantic-content cell helpers) and add small local helpers if needed.
   - Do not add public API or ABI exposure.

6. Verify.
   - Run:

     ```bash
     cargo fmt
     cargo test -p roastty select_line
     cargo test -p roastty terminal::page_list
     cargo test -p roastty
     ```

   - `cargo fmt` output must be accepted as-is.

7. Independent review.
   - Before implementation, get Codex review of this experiment design.
   - Record the design-review outcome in this experiment file before
     implementation.
   - After implementation and verification, get Codex review of the completed
     result.
   - Fix all real findings before proceeding.

8. Record the result.
   - Append `## Result` and `## Conclusion` to this file.
   - Include:
     - helper names and location;
     - upstream test coverage;
     - Roastty-specific edge tests;
     - verification command output summary;
     - Codex design-review outcome;
     - Codex result-review outcome.
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- Roastty has a private PageList helper equivalent to upstream `selectLine`;
- ordinary line selection, soft-wrap selection, disabled-whitespace selection,
  and semantic-boundary selection match upstream behavior;
- whitespace trimming returns `None` for lines with no non-whitespace text;
- hard row boundaries and soft-wrap row boundaries are distinguished correctly;
- semantic boundaries are based on cell semantic content, not row prompt
  metadata;
- invalid or garbage pins return `None`;
- all new selections are untracked non-rectangular selections;
- no full `SelectionGesture`, press/release state, word/line drag wiring,
  semantic-output selection, Screen, Terminal, public ABI, renderer, parser,
  app, platform input, autoscroll, deep-press, or mouse event behavior is added;
- `cargo fmt`, targeted line-selection tests, PageList tests, and full
  `cargo test -p roastty` pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- ordinary and soft-wrap line selection match upstream, but semantic-boundary
  behavior exposes a missing PageList fixture that must be split into its own
  follow-up.

The experiment fails if:

- line selection cannot be implemented without adding Screen, Terminal, parser,
  ABI, renderer, app, or platform input behavior;
- semantic-boundary behavior is approximated with row prompt metadata instead of
  cell semantic content;
- soft-wrapped and hard-bounded rows behave the same;
- whitespace trimming selects all-whitespace lines instead of returning `None`;
- invalid pins panic instead of returning `None`;
- tests or formatting fail.

## Design Review

Codex reviewed the initial design and found one blocking ambiguity:
`whitespace = None` disables whitespace trimming, but upstream still applies
semantic-content boundaries first when `semantic_prompt_boundary` is true.

The design now states that disabled whitespace trimming selects the full
soft-wrapped span only within the semantic-bounded range, and requires a test
proving semantic boundaries still apply with `whitespace = None` unless
`semantic_prompt_boundary = false`.

Follow-up Codex review approved the updated design with no remaining blockers.
