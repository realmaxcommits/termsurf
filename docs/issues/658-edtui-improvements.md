# Issue 658: EdTUI Improvements

Better modes and keybindings for the TUI URL bar editor.

## Problem

Two issues with the current URL bar editing:

1. **No submode concept.** The TUI has `Mode::UrlEdit` but treats it as a single
   flat mode. The editor internally has Normal, Insert, Visual, and Search
   submodes, but the TUI doesn't model this hierarchy. There's no visual
   indicator in the URL bar showing which editor submode is active.

2. **Limited keybindings.** Only `i` enters the editor (always in Insert mode at
   end of URL). The editor state is recreated each time, losing cursor position.
   There's no way to enter Normal or Visual mode directly from Control mode.

## Solution

### Mode hierarchy

Rename `Mode::UrlEdit` to `Mode::Edit`. The TUI now has three top-level modes:

| TUI Mode | Description                                                |
| -------- | ---------------------------------------------------------- |
| Browse   | Viewport is active, keys go to Chromium                    |
| Control  | URL bar is focused, keys are TUI commands (q, i, A, Enter) |
| Edit     | URL bar is being edited, keys go to edtui editor           |

When the TUI is in **Edit mode**, all keypresses route to edtui, which manages
its own submodes: Normal, Insert, Visual, Search. The TUI stays in Edit mode
regardless of which editor submode is active. The only way back to Control is
Esc while the editor is in Normal mode (Esc in Insert/Visual/Search first
returns the editor to Normal, then a second Esc exits Edit → Control).

### Submode indicator

Add a mode label in the top-right corner of the URL bar block, matching the
pattern used for the profile name in the viewport container
(`tui/src/main.rs:363`). Shows NORMAL, INSERT, VISUAL, or SEARCH in purple.

### Persistent editor state

Stop recreating `EditorState` on every `i` press. Initialize it once (and when
the URL changes externally via navigation). The cursor position persists across
Control ↔ Edit transitions.

### New keybindings from Control mode

| Key | Action                                     |
| --- | ------------------------------------------ |
| `i` | Enter Edit/Insert, cursor at last position |
| `A` | Enter Edit/Insert, cursor at end of line   |
| `I` | Enter Edit/Insert, cursor at start of line |
| `n` | Enter Edit/Normal, cursor at last position |
| `v` | Enter Edit/Visual, cursor at last position |
| `V` | Enter Edit/Visual, entire line selected    |

All six are supported by the edtui API:

- **Mode setting**: `state.mode = EditorMode::Insert` (or Normal, Visual)
- **Cursor at end**: `state.cursor.col = state.lines.len_col(0).unwrap_or(0)`
  (Insert mode allows past-end)
- **Cursor at start**: `state.cursor = Index2::new(0, 0)`
- **Line selection**: `SelectLine.execute(&mut state)` — sets Visual mode with
  `line_mode = true`
- **Visual init**: `SwitchMode(EditorMode::Visual).execute(&mut state)` —
  creates empty selection at cursor

### Changes

In `tui/src/main.rs`:

1. **Rename mode.** `Mode::UrlEdit` → `Mode::Edit` throughout.

2. **Persistent editor state.** Remove the `EditorState::new(...)` call from the
   `i` keypress handler. Instead, sync editor content from URL only when the URL
   changes (external navigation, initial load).

3. **Esc routing in Edit mode.** Intercept Esc before passing to edtui: if the
   editor is already in Normal mode, exit Edit → Control instead of forwarding
   to the editor.

4. **New keybindings.** Add `A`, `I`, `n`, `v`, `V` handlers in the
   `Mode::Control` match arm, each setting the appropriate editor mode/cursor
   and switching to `Mode::Edit`.

5. **Submode indicator.** In the Edit rendering branch, add a
   `.title_top(mode_label.alignment(Alignment::Right))` to the URL bar block,
   showing the current `EditorMode` as a colored label.

## Experiment 1: Submodes, persistent state, new keybindings

### Hypothesis

Persistent editor state with six entry keybindings, an inline submode indicator,
and proper Esc routing will make URL editing feel like a proper vim buffer.

### Test

1. Launch TUI, press `Esc` to Control, press `i` — Edit/Insert, cursor at end
2. Type some text, press `Esc` — editor goes to Normal (still in Edit mode)
3. Press `Esc` again — exits Edit → Control
4. Press `i` — Edit/Insert, cursor where you left it (not reset)
5. Press `Esc` twice to Control, press `A` — Edit/Insert, cursor at end of line
6. Press `Esc` twice to Control, press `I` — Edit/Insert, cursor at start
7. Press `n` — Edit/Normal, cursor at last position
8. Press `v` — Edit/Visual, empty selection at cursor
9. Press `V` — Edit/Visual, entire line selected
10. In all Edit submodes, top-right of URL bar shows NORMAL/INSERT/VISUAL/SEARCH
