+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"
+++

# Experiment 462: grow the Config struct with the mouse/click group

## Description

Experiment 461 began the aggregating `Config` struct with the clipboard group.
This experiment grows it with the next coherent field group: the **mouse / click
config** — `mouse_shift_capture`, `right_click_action`, `middle_click_action` —
all already-ported leaf enums (`MouseShiftCapture`, `RightClickAction`,
`MiddleClickAction`). It adds the three fields and their upstream `Config`-field
defaults to `Config` and its `Default`. The parser and the rest of upstream
`Config` stay deferred.

## Upstream behavior

In `config/Config.zig`, the mouse / click group's field defaults:

```zig
@"mouse-shift-capture": MouseShiftCapture = .false,
@"right-click-action": RightClickAction = .@"context-menu",
@"middle-click-action": MiddleClickAction = .@"primary-paste",
```

`mouse-shift-capture` defaults to `.false`; `right-click-action` defaults to
`.context-menu`; `middle-click-action` defaults to `.primary-paste`.

## Rust mapping (`roastty/src/config/mod.rs`)

The three fields are added to `Config`, and their defaults to
`Config::default()`:

```rust
pub(crate) struct Config {
    // ... clipboard group (Experiment 461) ...
    /// `mouse-shift-capture`.
    pub mouse_shift_capture: MouseShiftCapture,
    /// `right-click-action`.
    pub right_click_action: RightClickAction,
    /// `middle-click-action`.
    pub middle_click_action: MiddleClickAction,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... clipboard group ...
            mouse_shift_capture: MouseShiftCapture::False,
            right_click_action: RightClickAction::ContextMenu,
            middle_click_action: MiddleClickAction::PrimaryPaste,
        }
    }
}
```

The defaults are upstream's Config-field defaults for these three keys:
`mouse-shift-capture` `False`, `right-click-action` `ContextMenu`,
`middle-click-action` `PrimaryPaste`.

## Scope / faithfulness notes

- **Ported (bridged)**: the mouse / click field group of the aggregating
  `Config` struct (upstream `config.Config`) — the three fields and their
  `Default`.
- **Faithful**: the three fields use the already-ported types
  (`MouseShiftCapture`, `RightClickAction`, `MiddleClickAction`); their
  `Default` values match upstream's Config-field defaults (`.false`,
  `.context-menu`, `.primary-paste`).
- **Faithful adaptation**: the struct continues to grow one coherent field group
  per experiment (Experiment 461 added the clipboard group); this slice adds the
  mouse / click group, a faithful partial of upstream's `Config`. The derive set
  (`Clone`/`PartialEq`, not `Copy`/`Eq`) is unchanged.
- **Deferred**: the rest of upstream `Config`'s fields (added group by group in
  later slices), the parser, the `changeConfig` machinery, and the
  conditional-config system. (Consumed by later slices; this experiment grows
  the struct with the mouse / click group.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add the three fields `mouse_shift_capture: MouseShiftCapture`,
     `right_click_action: RightClickAction`,
     `middle_click_action: MiddleClickAction` to `Config`, and their defaults
     (`False`, `ContextMenu`, `PrimaryPaste`) to the `Default` impl.
2. Tests (in `config/mod.rs`):
   - extend / add a `Config::default()` assertion for the new fields:
     `mouse_shift_capture == MouseShiftCapture::False`,
     `right_click_action == RightClickAction::ContextMenu`,
     `middle_click_action == MiddleClickAction::PrimaryPaste`; the existing
     clipboard-group defaults still hold.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty config_default
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Config` gains the three mouse / click fields, and `Config::default()` sets
  their upstream defaults (`mouse-shift-capture` `False`, `right-click-action`
  `ContextMenu`, `middle-click-action` `PrimaryPaste`) while the clipboard-group
  defaults still hold — a faithful partial of upstream's `Config`;
- the tests pass (the new defaults; the existing defaults), and the existing
  tests still pass;
- the rest of upstream `Config` and the parser stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a default is wrong, a field uses the wrong type, an
unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: the defaults are correct
(`mouse_shift_capture = False`, `Config.zig:965`;
`right_click_action = ContextMenu`, `Config.zig:2433`;
`middle_click_action = PrimaryPaste`, `Config.zig:2443`); mouse/click is a
coherent next `Config` group (adjacent input-surface policy fields whose leaf
enums already exist); and the test plan is adequate (asserting the new defaults
plus the existing clipboard defaults protects both the new fields and the prior
default initializer as the aggregate grows).

Review artifacts:

- Prompt: `logs/codex-review/20260604-120818-d462-prompt.md` (design)
- Result: `logs/codex-review/20260604-120818-d462-last-message.md` (design)

## Result

**Result:** Pass

The `Config` struct now carries the mouse / click field group.

- `roastty/src/config/mod.rs`: `Config` gains
  `mouse_shift_capture: MouseShiftCapture`,
  `right_click_action: RightClickAction`, and
  `middle_click_action: MiddleClickAction`; `Config::default()` sets their
  upstream Config-field defaults — `MouseShiftCapture::False`,
  `RightClickAction::ContextMenu`, `MiddleClickAction::PrimaryPaste`.

Test (in `config/mod.rs`): `config_default_clipboard_group` extended to assert
the new mouse / click defaults (`False` / `ContextMenu` / `PrimaryPaste`)
alongside the existing clipboard-group defaults; the modified-config inequality
and the `Clone`/`PartialEq` round-trip remain.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2952 passed, 0 failed (no regressions; the existing
  `config_default` test was extended).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The aggregating `Config` struct now holds two field groups — the clipboard group
(Experiment 461) and the mouse / click group — demonstrating the incremental
growth pattern: each experiment adds a coherent group of already-ported leaf
fields with their upstream defaults, and the `config_default` test grows to
guard the whole initializer. The parser (`loadCli` / file loading / per-field
`parseCLI`), the `changeConfig` machinery, the conditional-config system, and
the remaining upstream `Config` fields stay deferred.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the mouse / click fields were added to the
aggregate `Config` with the correct upstream defaults (`False`, `ContextMenu`,
`PrimaryPaste`); keeping the existing clipboard default assertions in the same
test is adequate while `Config::default()` grows; the `Clone`/`PartialEq`
coverage still fits the aggregate shape; and the deferred parser /
`changeConfig` / conditional-config work remains properly scoped. No public C
ABI/header impact; nothing needed to change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-121006-r462-prompt.md` (result)
- Result: `logs/codex-review/20260604-121006-r462-last-message.md` (result)
