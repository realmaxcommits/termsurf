+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 721: Binding Action Tab Window Forwarding

## Description

Experiment 720 added title binding actions. Upstream Ghostty's nearby
surface-scoped actions also forward tab and window commands to the app runtime:

- `new_tab`
- `close_tab[:this|other|right]`
- `goto_window:previous|next`
- `toggle_split_zoom`
- `reset_window_size`
- `toggle_maximize`
- `toggle_fullscreen`

Roastty already has the generic runtime action callback and split forwarding.
This experiment adds the next small app-runtime forwarding slice for tab/window
commands. It does not implement the tab model, window UI, fullscreen policy, or
Swift frontend behavior; it only parses binding actions and forwards the
upstream-shaped action tags/storage through the existing callback ABI.

## Changes

- `roastty/include/roastty.h`
  - Add action tags matching upstream `ghostty_action_tag_e` values:
    - `ROASTTY_ACTION_NEW_TAB = 2`
    - `ROASTTY_ACTION_CLOSE_TAB = 3`
    - `ROASTTY_ACTION_TOGGLE_MAXIMIZE = 6`
    - `ROASTTY_ACTION_TOGGLE_FULLSCREEN = 7`
    - `ROASTTY_ACTION_GOTO_WINDOW = 17`
    - `ROASTTY_ACTION_TOGGLE_SPLIT_ZOOM = 20`
    - `ROASTTY_ACTION_RESET_WINDOW_SIZE = 23`
  - Add close-tab mode constants matching upstream
    `ghostty_action_close_tab_mode_e`:
    - `ROASTTY_CLOSE_TAB_THIS = 0`
    - `ROASTTY_CLOSE_TAB_OTHER = 1`
    - `ROASTTY_CLOSE_TAB_RIGHT = 2`
  - Add goto-window constants matching upstream `ghostty_goto_window_e`:
    - `ROASTTY_GOTO_WINDOW_PREVIOUS = 0`
    - `ROASTTY_GOTO_WINDOW_NEXT = 1`
  - Add fullscreen constants matching upstream `ghostty_fullscreen_e`:
    - `ROASTTY_FULLSCREEN_NATIVE = 0`
    - `ROASTTY_FULLSCREEN_MACOS_NON_NATIVE = 1`
    - `ROASTTY_FULLSCREEN_MACOS_NON_NATIVE_VISIBLE_MENU = 2`
    - `ROASTTY_FULLSCREEN_MACOS_NON_NATIVE_PADDED_NOTCH = 3`
  - Document storage conventions:
    - close tab: `storage[0] = roastty_close_tab_e`
    - goto window: `storage[0] = roastty_goto_window_e`
    - toggle fullscreen: `storage[0] = roastty_fullscreen_e`
    - no-storage actions leave storage zeroed.

- `roastty/src/lib.rs`
  - Add matching constants.
  - Reuse the existing generic runtime action parsed-binding variant for the new
    forwarded actions.
  - Extend `parse_binding_action` to accept:
    - `new_tab`
    - `close_tab` as the upstream default `this`
    - `close_tab:this`
    - `close_tab:other`
    - `close_tab:right`
    - `goto_window:previous`
    - `goto_window:next`
    - `toggle_split_zoom`
    - `reset_window_size`
    - `toggle_maximize`
    - `toggle_fullscreen`
  - Reject missing, empty, whitespace-padded, unknown, and extra-colon
    parameters where applicable; reject any parameter for no-parameter actions.
  - Forward actions through `action_cb`, returning `false` for null, detached,
    and no-callback surfaces and otherwise returning the callback result.
  - Forward `toggle_fullscreen` with `ROASTTY_FULLSCREEN_NATIVE` because Roastty
    does not yet expose Ghostty's macOS non-native fullscreen config.
  - Keep title, clipboard, font-size, split, close-surface, text/CSI/ESC, reset,
    clear-screen, scroll, prompt-jump, select-all, and adjust-selection
    semantics unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage for new action constants and enum constants.
  - Add malformed tab/window action rejection checks.
  - Add no-callback coverage that valid tab/window forwarding actions return
    `false` without crashing.

- Tests in `roastty/src/lib.rs`
  - Cover constant values matching upstream.
  - Cover parser false paths for invalid close-tab, goto-window, and
    no-parameter action forms.
  - Cover null, detached, and no-callback surfaces returning `false`.
  - Cover valid tab/window actions forwarding expected action tags, target,
    storage, and callback result.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty tab_window -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 721 design and found no findings. The review
approved the scope as parser plus app-runtime forwarding only, with action tags,
storage conventions, close-tab and goto-window modes, no-storage actions,
malformed-form tests, and callback-result tests all covered.

The review also accepted the explicit `toggle_fullscreen` scope decision:
Roastty forwards `ROASTTY_FULLSCREEN_NATIVE` for now because the macOS
non-native fullscreen configuration is not exposed yet.

## Result

**Result:** Pass

Implemented tab/window binding-action forwarding through the existing runtime
action callback path. Roastty now exposes upstream-matching action tags for new
tab, close tab, goto window, split zoom, reset window size, maximize, and
fullscreen actions, plus close-tab, goto-window, and fullscreen storage enums in
`roastty/include/roastty.h`.

`parse_binding_action` now accepts:

- `new_tab`
- `close_tab`, `close_tab:this`, `close_tab:other`, `close_tab:right`
- `goto_window:previous`, `goto_window:next`
- `toggle_split_zoom`
- `reset_window_size`
- `toggle_maximize`
- `toggle_fullscreen`

Invalid empty, unknown, whitespace-padded, and extra-colon forms are rejected.
No-argument actions reject parameters. `close_tab` defaults to `this`, matching
upstream, and `toggle_fullscreen` forwards `ROASTTY_FULLSCREEN_NATIVE` until
Roastty exposes the macOS non-native fullscreen policy.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty tab_window -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Tab/window binding actions now reach the app runtime with stable upstream-shaped
action tags and storage. The remaining work is the frontend/tab model behavior
that consumes these callbacks, plus the later fullscreen-policy configuration
that can choose non-native macOS modes.

## Completion Review

Codex reviewed the completed Experiment 721 result and found no implementation
blockers. The review approved the action constants, header enums, storage
conventions, parser behavior, callback forwarding, `close_tab` default mode, and
native fullscreen scope decision.

The review found one workflow blocker: result-review provenance was missing from
the experiment frontmatter and README tuple. This section, the `[review.result]`
frontmatter, and the README tuple now record the completion review.

The review also noted a non-blocking test coverage gap for empty-colon
no-argument actions. The Rust false-path test and C ABI harness now cover empty
colon forms for `new_tab`, `toggle_split_zoom`, `reset_window_size`,
`toggle_maximize`, and `toggle_fullscreen`.

Codex re-reviewed the revised result and found no remaining findings. The
completion review approved the result for commit.
