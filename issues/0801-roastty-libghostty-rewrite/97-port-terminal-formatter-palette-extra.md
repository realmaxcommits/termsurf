# Experiment 97: Port Terminal Formatter Palette Extra

## Description

Port the first terminal-level formatter extra: palette emission.

Experiment 96 added the nested `TerminalFormatterExtra.screen` forwarding bridge
without implementing terminal-level extras. Upstream Ghostty emits palette state
before screen content when `TerminalFormatter.Extra.palette` is enabled:

- VT output emits OSC 4 entries for all 256 palette indexes;
- HTML output emits a `<style>:root{...}</style>` block containing CSS palette
  variables;
- plain output ignores palette extras.

Roastty already has `color::Palette`, `color::Rgb`, and default palette tests,
but `Terminal` does not yet own terminal color state. This experiment adds
private terminal palette state and opt-in palette formatting only. It must not
add OSC 4 parser/runtime mutation, render-state color API, config color loading,
foreground/background/cursor dynamic color state, terminal modes, scrolling
region, tabstops, pwd, or keyboard extras.

## Changes

1. Re-read the upstream source of truth.
   - Use `vendor/ghostty/src/terminal/formatter.zig` for:
     - `TerminalFormatter.Extra.palette`;
     - VT OSC 4 output shape;
     - HTML CSS variable output shape;
     - palette-before-content ordering;
     - palette pin-map handling.
   - Use `vendor/ghostty/src/terminal/color.zig` for:
     - palette length;
     - default/current palette distinction;
     - dynamic palette concept.
   - Do not modify `vendor/ghostty/`.

2. Add private terminal color state.
   - In `roastty/src/terminal/terminal.rs`, add a private color state owned by
     `Terminal`, for example:

     ```rust
     struct TerminalColors {
         palette: color::Palette,
     }
     ```

   - Initialize the palette from `color::DEFAULT_PALETTE`.
   - Add `#[cfg(test)] pub(super)` helpers to set palette entries for formatter
     tests.
   - Add a narrow private helper that lets `TerminalFormatter` obtain the active
     screen top-left pin for terminal-level extra pin maps. Prefer changing
     `Screen::top_left_pin()` to `pub(super)` rather than inferring the pin from
     formatted content. Palette bytes must map to the true top-left pin even
     when the formatted content starts elsewhere.
   - Keep foreground/background/cursor dynamic color state deferred. The
     formatter palette extra only needs the current 256-entry palette.
   - Keep this state private. Do not expose public API or ABI.

3. Extend `TerminalFormatterExtra`.
   - Add `palette: bool`.
   - Extend `none()`.
   - Add a `palette(bool)` builder.
   - Keep `TerminalFormatter::init()` defaulting to
     `TerminalFormatterExtra::none()` so existing default output remains
     unchanged.
   - This is an intentional temporary divergence from upstream
     `TerminalFormatter.init()`, which defaults to `.styles`. Roastty cannot
     honestly expose upstream-style presets until the remaining terminal extras
     exist, so this experiment preserves the current no-extra default.
   - Do not add `styles()` or `all()` presets in this experiment. Upstream has
     them, but adding presets before all terminal extras exist would create
     misleading partial semantics.

4. Emit palette before screen content.
   - In `TerminalFormatter::format()`:
     - if `extra.palette` is true and output is VT, prepend all 256 OSC 4
       sequences before screen content:

       ```text
       \x1b]4;{index};rgb:{rr}/{gg}/{bb}\x1b\
       ```

       with two-digit lowercase hex for each channel.

     - if `extra.palette` is true and output is HTML, prepend:

       ```text
       <style>:root{--vt-palette-{index}: #{rr}{gg}{bb};...}</style>
       ```

       for all 256 palette entries, with two-digit lowercase hex channels.

     - if output is plain, emit no palette bytes.

   - Preserve current content, trim, unwrap, codepoint-map, selection, and
     forwarded screen-extra behavior.

5. Preserve pin-map semantics.
   - In `TerminalFormatter::format_with_pin_map()`, palette bytes must be
     byte-indexed.
   - Because palette bytes are terminal state rather than content bytes, map all
     emitted palette bytes to the active screen top-left pin, matching the
     upstream strategy for terminal-level extras.
   - For VT/HTML with content, palette pin-map entries come before content
     entries.
   - For `Content::None`, palette pin-map entries still map to top-left.
   - Plain output emits no palette bytes and therefore adds no palette pin-map
     entries.

6. Add upstream-equivalent tests.
   - Add TerminalFormatter tests for:
     - default formatting does not emit palette bytes;
     - VT palette output emits OSC 4 entries before content;
     - VT palette output includes customized entries for at least indexes 0, 1,
       and 255;
     - VT palette output contains exactly 256 OSC 4 entries;
     - VT palette output uses two-digit lowercase hex channels;
     - HTML palette output emits the `<style>:root{...}</style>` block before
       content;
     - HTML palette output includes customized entries for at least indexes 0,
       1, and 255;
     - HTML palette output contains exactly 256 `--vt-palette-` variables;
     - plain output ignores palette extras;
     - `Content::None` can emit only palette bytes for VT and HTML;
     - palette pin maps are byte-indexed and map palette bytes to top-left;
     - palette pin maps precede content pin-map entries for VT and HTML;
     - VT and HTML palette pin-map tests use content that starts away from
       top-left, such as a row-1 selection, so the test proves palette bytes map
       to the true top-left pin rather than the first content pin;
     - palette output can combine with forwarded screen extras in VT, with
       ordering `palette -> content -> screen extras`.
   - Keep existing TerminalFormatter, ScreenFormatter, and PageList formatter
     tests passing.

7. Verify.
   - Run:

     ```bash
     cargo fmt
     cargo test -p roastty terminal_formatter
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
     - terminal color state names and visibility;
     - palette extra field and default behavior;
     - exact VT OSC 4 sequence shape;
     - exact HTML CSS wrapper shape;
     - plain-output behavior;
     - ordering relative to content and forwarded screen extras;
     - pin-map behavior for palette bytes;
     - why OSC 4 parser/runtime color mutation and other terminal extras remain
       deferred;
     - verification command output summary;
     - Codex design-review outcome;
     - Codex result-review outcome.
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- `Terminal` has private current palette state initialized from
  `color::DEFAULT_PALETTE`;
- `TerminalFormatterExtra` has an opt-in palette flag;
- default TerminalFormatter output and pin maps remain unchanged;
- VT palette output emits all 256 OSC 4 entries before screen content;
- HTML palette output emits all 256 CSS palette variables before screen content;
- plain output ignores palette extras;
- palette bytes are byte-indexed in pin maps and map to active-screen top-left;
- forwarded screen extras still work and emit after content when combined with
  palette output;
- no OSC 4 parser/runtime mutation, foreground/background/cursor dynamic color
  state, render-state color API, config color loading, modes, scrolling region,
  tabstops, pwd, keyboard, public API, public ABI, app behavior, renderer
  behavior, PTY behavior, clipboard behavior, or UI behavior is added;
- `cargo fmt`, targeted formatter tests, PageList formatter tests, PageList
  tests, and full `cargo test -p roastty` pass;
- Codex design and result reviews approve the experiment, or all real findings
  are fixed before proceeding.

The experiment is partial if:

- palette emission cannot be represented honestly without first porting broader
  terminal color state.

The experiment fails if:

- default TerminalFormatter output changes;
- palette bytes emit without explicit `TerminalFormatter::with_extra()`;
- VT or HTML palette output omits entries or formats hex incorrectly;
- plain output emits palette bytes;
- palette bytes are emitted after content;
- palette pin maps become character-indexed, shorter than output bytes, or map
  to content pins instead of top-left;
- unrelated terminal extras or runtime parser behavior are added.

## Design Review

Codex reviewed this design before implementation.

Review artifacts:

- Prompt: `logs/codex-review/20260531-235104-664252-prompt.md`
- Result: `logs/codex-review/20260531-235104-664252-last-message.md`

Codex approved the overall scope and upstream fidelity, with four required
design fixes:

- specify a narrow private top-left pin helper instead of inferring the pin from
  content;
- require VT and HTML pin-map tests where content starts away from top-left;
- require exact 256-entry count assertions for VT OSC 4 entries and HTML CSS
  variables;
- explicitly record that preserving Roastty's no-extra default is an intentional
  temporary divergence from upstream `.styles` initialization.

All four findings were applied before implementation.
