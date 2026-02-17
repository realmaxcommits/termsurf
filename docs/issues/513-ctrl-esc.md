# Issue 513: Ctrl+Esc

## Background

The `web` TUI has two modes: Browse and Control. Pressing Esc (or Ctrl+Esc)
switches from Browse to Control mode. The status bar displays
`[ctrl+esc] force exit browse mode` as a hint.

## Problem

Ctrl+Esc works in WezTerm but not in TermSurf (Ghostty fork). Bare Esc works in
both. The `web` code hasn't changed — the issue is in how the terminal encodes
Ctrl+Esc and how crossterm parses it.

### What Ghostty sends

Ghostty's `function_keys.zig` (line 226) encodes Ctrl+Escape as:

```
\x1b[27;5;27~
```

This is the xterm "modify-other-keys" format (`CSI 27 ; modifier ; keycode ~`).
The entry has `modify_other_keys: .any`, meaning Ghostty sends this sequence
regardless of whether the application has requested modify-other-keys mode.

Bare Escape sends `\x1b` — a single byte that crossterm handles fine.

### What crossterm expects

The `web` TUI uses crossterm 0.28.1 **without enabling keyboard enhancement
flags**. Without `PushKeyboardEnhancementFlags`, crossterm's legacy parser
handles standard CSI sequences (arrows, function keys, etc.) but does not
recognize the `CSI 27 ; 5 ; 27 ~` format — key number 27 is not in the standard
function key table. The sequence is either silently dropped or misinterpreted.

### Why WezTerm works

WezTerm likely sends a different encoding for Ctrl+Esc — either the same bare
`\x1b` as unmodified Escape, or a sequence that crossterm's legacy parser
recognizes. The exact encoding WezTerm uses has not been verified.

### The `web` code

In `web/src/main.rs` (line 117):

```rust
if key.code == KeyCode::Esc {
    mode = Mode::Control;
}
```

This checks only `key.code` without inspecting modifiers. It would match
Ctrl+Esc if crossterm delivered it as `KeyCode::Esc` with
`KeyModifiers::CONTROL` — but crossterm never produces that event because it
can't parse the sequence Ghostty sends.

## Options

### 1. Enable keyboard enhancement in `web`

Crossterm supports the kitty keyboard protocol via
`PushKeyboardEnhancementFlags`. When enabled, the terminal sends key events in a
format crossterm can fully parse, including modifier information for all keys.

```rust
use crossterm::event::{
    PushKeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    KeyboardEnhancementFlags,
};

// On startup:
execute!(stdout, PushKeyboardEnhancementFlags(
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
))?;

// On exit:
execute!(stdout, PopKeyboardEnhancementFlags)?;
```

With `DISAMBIGUATE_ESCAPE_CODES`, Ghostty would send Ctrl+Esc using the kitty
protocol (`CSI 27 ; 5 u` or similar) instead of the xterm modify-other-keys
format, and crossterm would parse it correctly.

Pros: Proper solution. Works for all modified keys, not just Ctrl+Esc. Cons:
Changes the keyboard protocol for the entire TUI. Need to verify this doesn't
break other keybindings. Only works in terminals that support the kitty keyboard
protocol (Ghostty does, WezTerm does, many others do not).

### 2. Fall back to bare Esc

Remove the Ctrl+ requirement entirely. The code already matches bare Esc. Just
update the status bar hint from `[ctrl+esc]` to `[esc]`.

Pros: Trivial one-line change. Works everywhere. Cons: Loses the ability to
distinguish Esc from Ctrl+Esc. In the future, when browser input forwarding is
implemented, bare Esc may conflict with webpage key handling (e.g., closing a
dropdown). Ctrl+Esc would provide an unambiguous "force exit" that the browser
would never intercept.

### 3. Parse the raw sequence manually

Add a custom parser that recognizes `\x1b[27;5;27~` and emits the appropriate
key event. Crossterm supports raw event reading and custom parsing.

Pros: Targeted fix without changing the keyboard protocol. Cons: Fragile — only
handles this one sequence. Doesn't scale to other modified keys we may need
later.

## Recommendation

Option 1 (keyboard enhancement) is the correct long-term solution. The `web` TUI
will eventually need full keyboard support for browser input forwarding —
enabling the kitty protocol now sets the right foundation. Option 2 is a
reasonable fallback if keyboard enhancement causes unexpected issues.
