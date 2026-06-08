+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 5: Comprehensive keyboard & mouse input matrix — drive everything, map what works

## Description

Exp 4 gave us window-isolated **output** (screenshots). This experiment
establishes **input**: drive a comprehensive matrix of keyboard and mouse events
against the real Ghostty app and produce a **results table** — every input class
marked **Works / Partial / Fails**, with the failure mode and the injection
mechanism recorded. The point is the **map**: which inputs the agent can
reliably inject and which it can't (scrolling is a prime suspect — we've hit
scroll problems before). Individual input failures are _data_, not experiment
failure; a complete, honest map is the success condition.

This is the last Phase-A prerequisite before live-A/B feature testing: the A/B
harness can only exercise a feature if we can drive its input.

This experiment changes **no roastty source** and **no app source** — only
harness tooling under `scripts/ghostty-app/`. Screenshots obey the no-commit
policy (out-of-repo; only the text results table is recorded here).

## Two things can fail — the table separates them

For every input we record **(1) Injected?** — could the agent synthesize the
event at all — and **(2) Effect?** — did Ghostty actually receive and act on it.
"Injected but no effect" (e.g. wrong Space, not focused, mouse-mode off) is a
different finding than "can't inject," and the likely scroll failure is probably
the former.

## Oracles — how we know an input landed (deterministic where possible)

- **PTY byte log (keyboard + mouse-reporting):** a small raw-mode reader runs
  _inside_ Ghostty and appends every received byte (hex) to a file on disk
  (`$TS/bytes.log`). The agent reads that file directly — **no OCR**. Injecting
  `a` → `61`; `Return` → `0d`; `Ctrl-C` → `03`; `Up` → `1b 5b 41`; a mouse click
  in SGR-1006 mode → `1b 5b 3c 0 ; …`. This is the precise oracle for what bytes
  reached the PTY.
  - **Bootstrapping (shell matters):** Ghostty's default shell is **nushell**,
    whose syntax is not POSIX (`>` is a comparison, `$TS` errors — so a naive
    `echo X > $TS/f` writes nothing and would silently corrupt the
    inject-vs-effect gate). So the **first injected command switches to a clean
    POSIX shell** — `exec bash --norc --noprofile`, then `PS1='READY$ '` (fixed
    prompt) and `export TS=/tmp/ghostty-exp5` (`mkdir -p`). Only then do we
    prove text+Return by injecting `echo MARKER > $TS/marker` and reading that
    file. Every driver command (`seq`, `printf` of DECSET, the byte probe) runs
    in this bash session, so all POSIX idioms hold.
- **Pasteboard (selection/copy):** after a drag-select + `Cmd-C`, read `pbpaste`
  and assert the exact selected text.
- **Window/app state (app keybindings):** new window/tab/split → window count
  via `winid.swift --list`; font-size → window/cell dimensions change.
- **Window screenshot (visual effects):** `screenshot.sh` (Exp 4) for rendering,
  color, selection highlight, scrollback position, context menu, cursor — saved
  out-of-repo.

## Injection mechanisms (and we record which one each input needed)

- **Keyboard:** `osascript` System Events (`keystroke` for text,
  `key code N using {… down}` for keys/modifiers) first; **CGEvent**
  (`CGEventCreateKeyboardEvent`) where System Events can't express a key.
- **Mouse:** **CGEvent** (`CGEventCreateMouseEvent`, drag via
  move-with-button-down, `CGEventCreateScrollWheelEvent` for scroll) —
  `osascript` mouse support is too weak; `cliclick` is not installed.
  Coordinates come from the window's point bounds (`winid.swift`, Exp 4) →
  cell/region targeting; CGWindowBounds and CGEvent share **point** units (no
  Retina conversion, unlike screenshots).
- **Posting + the Spaces/focus problem (the suspected scroll culprit):** the
  agent's Wezboard is fullscreen on another Space; Ghostty is elsewhere.
  Keyboard reaches only the **frontmost** app
  (`osascript keystroke`/`key code`), and mouse-by-coordinate hit-tests only the
  **active display Space**. So the committed approach is **activate-first**:
  `osascript … to activate` Ghostty (which switches the active Space to
  Ghostty's), then post **global** keyboard/`CGEventPost` mouse events. This
  costs nothing for the oracles — screenshots are window-id-based (Exp 4), the
  byte log is a file, `pbpaste` is global — so they survive the Space switch.
  `CGEventPostToPid` is kept only as a recorded fallback (it does **not**
  reliably hit-test an off-Space window). The harness **records what each input
  required** to actually land — itself a key result (and the likely explanation
  if scroll fails).
- **Fallback noted:** XCUITest (the app ships
  `Ghostty.xctestplan`/`GhosttyUITests`) is the most robust injector but needs
  the test target built/run with special permissions; recorded as the escalation
  path for any input the lighter mechanisms can't drive.

## The input matrix (planned; result columns filled at run)

Each row is scored **Inject** (Y/N), **Effect** (Y/N), **Result**
(Works/Partial/Fails), with the mechanism + notes.

### A. Keyboard → PTY (oracle: byte log)

| Group                | Inputs                                                                                                                                                                                   |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Printable ASCII      | a–z, A–Z (Shift), 0–9, symbol row + punctuation                                                                                                                                          |
| Whitespace/edit      | Space, Tab, Return, Backspace, Forward-Delete                                                                                                                                            |
| Escape               | Esc (and `Ctrl-[` = ESC)                                                                                                                                                                 |
| Arrows               | Up/Down/Left/Right (normal + application-cursor-keys mode)                                                                                                                               |
| Navigation           | Home, End, PageUp, PageDown, Insert, Delete                                                                                                                                              |
| Function keys        | F1–F12                                                                                                                                                                                   |
| Control combos       | Ctrl-A/E/B/F (nav), Ctrl-U/K/W (kill), Ctrl-L (clear), Ctrl-C (SIGINT), Ctrl-D (EOF — guarded), Ctrl-Z (SIGTSTP — guarded)                                                               |
| Option/Alt as Meta   | Alt+letter (ESC-prefixed / per `macos-option-as-alt`)                                                                                                                                    |
| Chorded arrows       | Shift+Arrow, Option+Arrow (word), Cmd+Arrow (line)                                                                                                                                       |
| Unicode/IME          | Option-e e → `é`; one emoji/CJK entry (expected hard — recorded honestly)                                                                                                                |
| Terminal-mode events | bracketed paste (DECSET 2004 → `ESC[200~`…`201~` around Cmd-V), focus-in/out (DECSET 1004 — the harness's own Space/focus switching triggers these; measure them), key-repeat (held key) |

### B. Keyboard → app keybindings (oracle: window/app state or screenshot)

| Inputs                                                                                     |
| ------------------------------------------------------------------------------------------ |
| Cmd-N (new window), Cmd-T (new tab), Cmd-W (close surface)                                 |
| Cmd-D / Cmd-Shift-D (split), Cmd-] / Cmd-[ (focus split)                                   |
| Cmd-+ / Cmd-- / Cmd-0 (font size), Cmd-K (clear scrollback)                                |
| Cmd-C / Cmd-V (copy/paste), Cmd-F (find), Cmd-A (select all)                               |
| Cmd-Ctrl-F (fullscreen), Cmd-, (settings) — Cmd-Q (quit) **excluded** (would kill the run) |

### C. Mouse (oracle: pasteboard / screenshot / byte log in mouse-report mode)

| Input                               | Primary oracle                                        |
| ----------------------------------- | ----------------------------------------------------- |
| Move + left click                   | screenshot (focus); byte log if mouse-report on       |
| Left click-drag → selection         | screenshot (highlight) + `pbpaste` after Cmd-C        |
| Double-click → word select          | `pbpaste`                                             |
| Triple-click → line select          | `pbpaste`                                             |
| Right-click → context menu          | screenshot                                            |
| Middle-click → paste/report         | screenshot / byte log                                 |
| **Scroll up/down → scrollback**     | screenshot (earlier lines appear) — **prime suspect** |
| Scroll in mouse-report mode         | byte log (encoded scroll events)                      |
| Cmd-click URL (OSC 8)               | observe browser open / app behavior                   |
| Mouse reporting 1000/1002/1003/1006 | byte log (encoded click/drag/motion sequences)        |

## Changes / Deliverables

- `scripts/ghostty-app/inject.swift` — CGEvent injector with subcommands:
  `text <s>`, `key <keycode> [cmd,ctrl,opt,shift]`, `move <x> <y>`,
  `click <x> <y> [left|right|middle] [n]`, `drag <x1> <y1> <x2> <y2>`,
  `scroll <x> <y> <dy>`. Posts globally and/or to a pid.
- `scripts/ghostty-app/byteprobe.py` — raw-mode byte logger (run inside
  Ghostty). Must set the tty to raw with **ISIG disabled** (so Ctrl-C/Z/D arrive
  as bytes `03`/`1a`/`04`, not signals/EOF that would kill the probe),
  **`VMIN=1`**, and **append+flush per byte** (line-buffering would hide
  arrow/control/mouse sequences until newline). Optionally `printf`s the
  mouse-reporting / focus / bracketed-paste DECSETs first.
- `scripts/ghostty-app/input-matrix.sh` — orchestrator: focus Ghostty, set up
  the byte probe + scrollback content (`seq 1 200`), drive every matrix row,
  collect the oracle for each, and emit a **markdown results table** to stdout
  (and the agent transcribes it into this doc's Result). Uses `osascript` for
  keyboard, `inject.swift` for mouse, `screenshot.sh`/`pbpaste`/`winid.swift` as
  oracles.
- This experiment's **## Result** = the filled results table
  (Works/Partial/Fails per row) + a short narrative of the failure modes and any
  permission grants required.

## Verification

1. Launch the Exp-3 Ghostty app and **`activate`** it (front + active Space).
   Switch to a clean POSIX shell — inject `exec bash --norc --noprofile`, then
   `PS1='READY$ '` and `export TS=/tmp/ghostty-exp5; mkdir -p $TS` — since the
   default shell is nushell.
2. Bootstrap gate: inject `echo MARKER > $TS/marker && <Return>` (now valid
   bash); confirm the marker file appears (basic text + Return inject **and**
   take effect) — else that's the first, blocking result.
3. Start `byteprobe.py` in Ghostty; drive group **A**, reading `$TS/bytes.log`
   after each key; record the bytes received vs expected.
4. Drive group **B**; check window/app state (`winid.swift --list`, dimensions)
   or screenshot per row.
5. Drive group **C** with `inject.swift`; selection via `pbpaste`,
   scrollback/menu via screenshot, mouse-report via byte log. **Explicitly test
   scroll** with ≥1 screen of scrollback and record live-vs-no-effect.
6. Assemble the results table; document every failure mode and the exact
   permission(s) relied on (expected: **Accessibility** for keyboard/CGEvent
   posting, in addition to Exp 4's Screen Recording).

**Pass** = the **entire matrix is driven and classified** — every row attempted,
marked Works/Partial/Fails, with mechanism + failure mode + required permissions
documented — i.e. a complete, trustworthy map (even if several inputs, e.g.
scroll, fail).

**Partial** = a mechanism or permission gap blocks an entire class from being
_attempted_ (not merely failing), and that gap is documented with the
remediation.

**Fail** = no input can be injected or no oracle can observe effects (the whole
approach is blocked) — documented precisely.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only + cheap probes). **Verdict: CHANGES REQUIRED → addressed.** The
reviewer **verified the core mechanisms on this machine**: Accessibility is
granted (`AXIsProcessTrusted=true`; the agent's shell is parented to
`TermSurf Wezboard.app`, the responsible app); the raw-mode byte oracle works
(`printf 'ab\x03\x1b[A' | python3` raw read → `61 62 03 1b 5b 41`, i.e.
`a b Ctrl-C Up` as bytes); and the point-unit claim holds (CGWindowBounds and
CGEvent share global points). So permission, the oracle, and coordinate math are
sound.

Findings and fixes:

- **Required — wrong shell.** The bootstrap/driver commands were POSIX, but the
  running Ghostty's default shell is **nushell** (`nu`), where
  `echo MARKER > $TS/marker` writes nothing (`>` is comparison, `$TS` errors) —
  which would make the inject-vs-effect gate misfire even when injection works.
  **Fixed:** the first injected command now `exec bash --norc --noprofile`
  (fixed `PS1`, `TS=/tmp/ghostty-exp5`), so every driver runs in POSIX bash;
  called out in the oracle bootstrap and Verification.
- **Optional — byteprobe correctness.** **Fixed:** the deliverable now specifies
  raw mode with **ISIG disabled** (Ctrl-C/Z/D arrive as bytes, not signals),
  `VMIN=1`, and append+flush per byte.
- **Optional — off-Space mouse.** `CGEventPostToPid` doesn't reliably hit-test a
  window on another Space. **Fixed:** committed to **activate-first + global
  `CGEventPost`** (oracles survive the Space switch); `CGEventPostToPid` kept
  only as a recorded fallback. Noted that keyboard likewise needs activate
  (reaches frontmost only).
- **Optional — missing classes.** **Fixed:** added bracketed paste (DECSET
  2004), focus-in/out (DECSET 1004 — the harness's own Space switching triggers
  these), and key-repeat to the matrix.
- **Nit — XCUITest as escalation-only** confirmed reasonable (building the
  UI-test target would relaunch the app and is heavier than CGEvent for a
  one-shot map). No change.

## Result

_(to be added after the run — the filled results table.)_

## Conclusion

_(to be added after the run.)_
