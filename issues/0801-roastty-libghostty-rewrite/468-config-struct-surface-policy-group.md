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

# Experiment 468: grow the Config struct with the surface-policy group

## Description

Continuing the incremental growth of the aggregating `Config` struct
(Experiments 461–467), this experiment adds the **surface-policy** group:
`confirm_close_surface`, `link_previews`, and `window_subtitle` — all
already-ported leaf enums (`ConfirmCloseSurface`, `LinkPreviews`,
`WindowSubtitle`). It adds the three fields and their upstream `Config`-field
defaults to `Config` and its `Default`. The parser and the rest of upstream
`Config` stay deferred.

## Upstream behavior

In `config/Config.zig`, the surface-policy group's field defaults:

```zig
@"confirm-close-surface": ConfirmCloseSurface = .true,
@"link-previews": LinkPreviews = .true,
@"window-subtitle": WindowSubtitle = .false,
```

`confirm-close-surface` defaults to `.true` (confirm when a command appears to
be running); `link-previews` defaults to `.true` (preview every link);
`window-subtitle` defaults to `.false` (no subtitle).

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
pub(crate) struct Config {
    // ... clipboard (461) … optional-colors (467) ...
    /// `confirm-close-surface`.
    pub confirm_close_surface: ConfirmCloseSurface,
    /// `link-previews`.
    pub link_previews: LinkPreviews,
    /// `window-subtitle`.
    pub window_subtitle: WindowSubtitle,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... earlier groups ...
            confirm_close_surface: ConfirmCloseSurface::True,
            link_previews: LinkPreviews::True,
            window_subtitle: WindowSubtitle::False,
        }
    }
}
```

The defaults are upstream's Config-field defaults: `confirm-close-surface`
`True`, `link-previews` `True`, `window-subtitle` `False`.

## Scope / faithfulness notes

- **Ported (bridged)**: the surface-policy field group of the aggregating
  `Config` struct (upstream `config.Config`) — the three fields and their
  `Default`.
- **Faithful**: the three fields use the already-ported types
  (`ConfirmCloseSurface`, `LinkPreviews`, `WindowSubtitle`); their `Default`
  values match upstream's Config-field defaults (`.true`, `.true`, `.false`).
- **Faithful adaptation**: the struct continues to grow one coherent field group
  per experiment. The derive set (`Clone`/`PartialEq`) is unchanged.
- **Deferred**: the rest of upstream `Config`'s fields (added group by group in
  later slices), the parser, the `changeConfig` machinery, and the
  conditional-config system. (Consumed by later slices; this experiment grows
  the struct with the surface-policy group.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add the three fields `confirm_close_surface: ConfirmCloseSurface`,
     `link_previews: LinkPreviews`, `window_subtitle: WindowSubtitle` to
     `Config`, and their defaults (`True`, `True`, `False`) to the `Default`
     impl.
2. Tests (in `config/mod.rs`):
   - extend the `Config::default()` assertion for the new fields:
     `confirm_close_surface == ConfirmCloseSurface::True`,
     `link_previews == LinkPreviews::True`,
     `window_subtitle == WindowSubtitle::False`; the existing group defaults
     still hold.
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

- `Config` gains the three surface-policy fields, and `Config::default()` sets
  their upstream defaults (`confirm-close-surface` `True`, `link-previews`
  `True`, `window-subtitle` `False`) while the earlier group defaults still hold
  — a faithful partial of upstream's `Config`;
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
(`confirm_close_surface = ConfirmCloseSurface::True`, `Config.zig:2499`;
`link_previews = LinkPreviews::True`, `Config.zig:1436`;
`window_subtitle = WindowSubtitle::False`, `Config.zig:2110`); the
surface-policy group is coherent (app/surface-facing policy knobs with
already-ported leaf enums); and the test plan is adequate (assert the three new
defaults and keep the existing groups covered as `Default` grows).

Review artifacts:

- Prompt: `logs/codex-review/20260604-123046-d468-prompt.md` (design)
- Result: `logs/codex-review/20260604-123046-d468-last-message.md` (design)

## Result

**Result:** Pass

The `Config` struct now carries the surface-policy field group.

- `roastty/src/config/mod.rs`: `Config` gains
  `confirm_close_surface: ConfirmCloseSurface`, `link_previews: LinkPreviews`,
  and `window_subtitle: WindowSubtitle`; `Config::default()` sets their upstream
  Config-field defaults — `ConfirmCloseSurface::True`, `LinkPreviews::True`,
  `WindowSubtitle::False`.

Test (in `config/mod.rs`): `config_default_clipboard_group` extended to assert
the three new surface-policy defaults (`True` / `True` / `False`) alongside the
seven prior groups' defaults; the modified-config inequality and the
`Clone`/`PartialEq` round-trip remain.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2952 passed, 0 failed (no regressions; the existing
  `config_default` test was extended).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The aggregating `Config` struct now holds eight field groups — clipboard (461),
mouse/click (462), shell-integration (463), notification (464),
renderer-appearance (465), background-image (466), optional-colors (467), and
surface-policy — twenty-six fields total, drawing on the leaf enums and color
value types ported earlier this issue. The parser, the `changeConfig` machinery,
the conditional-config system, and the remaining upstream `Config` fields stay
deferred.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the surface-policy fields were added with faithful
defaults (`ConfirmCloseSurface::True`, `LinkPreviews::True`,
`WindowSubtitle::False`); the group remains properly scoped to aggregate
`Config` defaults only; and extending the existing `Config::default()` test is
adequate and keeps all prior defaults covered. No public C ABI/header impact;
nothing needed to change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-123228-r468-prompt.md` (result)
- Result: `logs/codex-review/20260604-123228-r468-last-message.md` (result)
