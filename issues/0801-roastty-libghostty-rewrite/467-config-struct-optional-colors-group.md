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

# Experiment 467: grow the Config struct with the optional-colors group

## Description

Continuing the incremental growth of the aggregating `Config` struct
(Experiments 461–466), this experiment adds the **optional-colors** group:
`cursor_color`, `cursor_text`, `selection_foreground`, `selection_background`
(each `Option<TerminalColor>`), and `bold_color` (`Option<BoldColor>`). These
reuse the already-ported color value types (`TerminalColor` from Experiment 446,
`BoldColor` from Experiment 447) — the first time the `Config` aggregate uses
the color value types it was built toward. All five default to `None` (upstream
`null`). The parser and the rest of upstream `Config` stay deferred.

## Upstream behavior

In `config/Config.zig`, the optional-colors group's field defaults (all `null`):

```zig
@"selection-foreground": ?TerminalColor = null,
@"selection-background": ?TerminalColor = null,
@"cursor-color": ?TerminalColor = null,
@"cursor-text": ?TerminalColor = null,
@"bold-color": ?BoldColor = null,
```

Each is an optional color that defaults to `null` (unset): when `null`, the
consumer uses a fallback (e.g. the cursor falls back to the inverse cell color,
the selection to the theme's selection colors). The renderer's `DerivedConfig`
reads these (`cursor_color`, `cursor_text`, `selection_*`, `bold_color`) and
resolves each via `toTerminalRGB` / `toTerminal` when present.

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
pub(crate) struct Config {
    // ... clipboard (461) … background-image (466) ...
    /// `cursor-color`.
    pub cursor_color: Option<TerminalColor>,
    /// `cursor-text`.
    pub cursor_text: Option<TerminalColor>,
    /// `selection-foreground`.
    pub selection_foreground: Option<TerminalColor>,
    /// `selection-background`.
    pub selection_background: Option<TerminalColor>,
    /// `bold-color`.
    pub bold_color: Option<BoldColor>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... earlier groups ...
            cursor_color: None,
            cursor_text: None,
            selection_foreground: None,
            selection_background: None,
            bold_color: None,
        }
    }
}
```

The defaults are upstream's Config-field defaults: all five are `None` (upstream
`null`). The fields use the already-ported `TerminalColor` / `BoldColor` value
types, wrapped in `Option` (upstream's `?`).

## Scope / faithfulness notes

- **Ported (bridged)**: the optional-colors field group of the aggregating
  `Config` struct (upstream `config.Config`) — the five fields and their
  `Default`.
- **Faithful**: the four `?TerminalColor` fields and the one `?BoldColor` field
  use the already-ported value types wrapped in `Option`; all five `Default`
  values are `None` (upstream `null`).
- **Faithful adaptation**: upstream's `?T` maps to `Option<T>`; the `null`
  default maps to `None`. `Option<TerminalColor>` / `Option<BoldColor>` are
  `Clone`/`PartialEq` (the value types are, and `TerminalColor` is `Copy` so its
  `Option` is too; `BoldColor` is `Copy`). The struct continues to grow one
  coherent field group per experiment; the derive set is unchanged.
- **Deferred**: the rest of upstream `Config`'s fields (added group by group in
  later slices), the parser, the `changeConfig` machinery, the
  conditional-config system, and the renderer `DerivedConfig` resolution of
  these optional colors (`toTerminalRGB` / `toTerminal` when present, the `None`
  fallback). (Consumed by later slices; this experiment grows the struct with
  the optional-colors group.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add the five fields `cursor_color: Option<TerminalColor>`,
     `cursor_text: Option<TerminalColor>`,
     `selection_foreground: Option<TerminalColor>`,
     `selection_background: Option<TerminalColor>`,
     `bold_color: Option<BoldColor>` to `Config`, and their defaults (all
     `None`) to the `Default` impl.
2. Tests (in `config/mod.rs`):
   - extend the `Config::default()` assertion for the new fields: all five are
     `None` (`cursor_color`, `cursor_text`, `selection_foreground`,
     `selection_background`, `bold_color`); the existing group defaults still
     hold.
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

- `Config` gains the five optional-colors fields, and `Config::default()` sets
  them all to `None` (upstream `null`) while the earlier group defaults still
  hold — a faithful partial of upstream's `Config`;
- the tests pass (the new `None` defaults; the existing defaults), and the
  existing tests still pass;
- the rest of upstream `Config`, the parser, and the renderer resolution stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a default is not `None`, a field uses the wrong type
(e.g. not wrapped in `Option`), an unrelated item changes, or any public C
API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: all five fields are
optional and default to `null`, so `Option<...> = None` is the right Rust
mapping (`selection_foreground` `Config.zig:707`; `selection_background` `:708`;
`cursor_color` `:851`; `cursor_text` `:902`; `bold_color` `:3709`); the types
are correct (the four cursor/selection fields `Option<TerminalColor>`,
bold-color `Option<BoldColor>`); reusing the already-ported `TerminalColor` /
`BoldColor` value types is the right boundary (renderer/`DerivedConfig`
resolution belongs in a later slice); the optional-colors group is coherent; and
the test plan is adequate (assert all five new defaults are `None` and keep the
existing `Config` defaults covered).

Review artifacts:

- Prompt: `logs/codex-review/20260604-122706-d467-prompt.md` (design)
- Result: `logs/codex-review/20260604-122706-d467-last-message.md` (design)

## Result

**Result:** Pass

The `Config` struct now carries the optional-colors field group.

- `roastty/src/config/mod.rs`: `Config` gains `cursor_color`, `cursor_text`,
  `selection_foreground`, `selection_background` (each `Option<TerminalColor>`),
  and `bold_color` (`Option<BoldColor>`); `Config::default()` sets all five to
  `None` (upstream `null`). This is the first time the aggregate uses the
  already-ported color value types (`TerminalColor`, `BoldColor`).

Test (in `config/mod.rs`): `config_default_clipboard_group` extended to assert
all five new optional-color fields are `None` alongside the six prior groups'
defaults; the modified-config inequality and the `Clone`/`PartialEq` round-trip
remain.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2952 passed, 0 failed (no regressions; the existing
  `config_default` test was extended).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The aggregating `Config` struct now holds seven field groups — clipboard (461),
mouse/click (462), shell-integration (463), notification (464),
renderer-appearance (465), background-image (466), and optional-colors —
twenty-three fields total. The optional-colors group is the first to use the
`TerminalColor` / `BoldColor` value types (`Color` → `TerminalColor` /
`BoldColor`, Experiments 445–447) that the `Config` aggregate was built toward,
each wrapped in `Option` for upstream's `?T = null`. The parser, the
`changeConfig` machinery, the conditional-config system, the renderer
`DerivedConfig` resolution of these colors, and the remaining upstream `Config`
fields stay deferred.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the four `?TerminalColor` fields and one
`?BoldColor` field are faithfully `Option<TerminalColor>` / `Option<BoldColor>`;
all five defaults are correctly `None` (upstream `null`); reusing the existing
value types is appropriate (renderer/`DerivedConfig` resolution remains
deferred); and extending the existing `Config::default()` test is adequate and
keeps the prior groups covered. No public C ABI/header impact; nothing needed to
change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-122904-r467-prompt.md` (result)
- Result: `logs/codex-review/20260604-122904-r467-last-message.md` (result)
