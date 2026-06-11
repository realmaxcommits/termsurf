+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 86: Phase F — VT KAM config and key gate

## Description

Experiment 85 completed `command-palette-entry`. The next upstream config field
after the already-ported `osc-color-report-format` is `vt-kam-allowed`.

Upstream declares `vt-kam-allowed: bool = false`. This field is intentionally
more than a parser/formatter toggle: `Surface.zig` copies it into the derived
surface config and, during key handling, checks it after keybinding dispatch but
before writing encoded key input. If `vt-kam-allowed` is true and ANSI mode 2
(`disable_keyboard`, KAM) is enabled, ordinary keyboard input is consumed
without being written to the terminal. Keybindings still run first.

This experiment ports the config field and the embedded surface key gate. It is
narrowly scoped to the `roastty_surface_key` / `roastty_surface_key_handle`
path; broader app UI preferences and documentation are out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::vt_kam_allowed: bool` in upstream declaration order after
    `osc-color-report-format` and before the custom-shader fields.
  - Default it to `false`, matching upstream.
  - Format it as `vt-kam-allowed = false` / `true` immediately after
    `osc-color-report-format`.
  - Route `vt-kam-allowed` through `Config::set`, `load_str`, diagnostics,
    clone/equality, and `format_config` using the existing bool-field semantics:
    bare flag means true, empty value resets to default, invalid bool is an
    invalid value.
  - Extend the config format-order test so the field lands between
    `osc-color-report-format` and `custom-shader-animation` until
    `custom-shader` is ported.
  - Add focused config tests for default, explicit true/false, bare flag, empty
    reset, invalid value diagnostics, and `load_str` preservation around
    neighboring valid lines.
- `roastty/src/lib.rs`
  - Store the effective `vt_kam_allowed` value on `Surface` when a surface is
    created.
  - Refresh the stored surface value in `Surface::apply_config`, matching
    upstream `Surface.updateConfig` rebuilding `DerivedConfig` from the updated
    config.
  - Add a small terminal-mode query path for the surface to ask whether ANSI KAM
    / `Mode::DisableKeyboard` is currently enabled.
  - In `Surface::key`, keep the upstream ordering:
    1. consume tracked keybinding releases;
    2. dispatch configured keybindings;
    3. dispatch default keybindings;
    4. if `vt_kam_allowed` and terminal KAM mode are both enabled, return
       consumed without writing encoded key input;
    5. otherwise write the encoded key input.
  - Add embedded-surface key tests proving the gate consumes ordinary key input
    only when both the config and terminal mode are enabled.
  - Add an update-path test proving `roastty_app_update_config` or
    `roastty_surface_update_config` changes the KAM gate for an existing
    surface.
  - Add a test proving keybindings still win before the KAM gate.

Out of scope:

- The `custom-shader` field that follows `vt-kam-allowed` upstream.
- Any command-palette runtime behavior from Experiment 85.
- App settings UI, documentation, or C ABI config accessors beyond the existing
  surface config path.
- Changing terminal mode parsing itself; ANSI mode 2 already exists locally as
  `Mode::DisableKeyboard`.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/86-vt-kam-config-key-gate.md`
- Run targeted tests:
  - `cargo test -p roastty vt_kam`
  - `cargo test -p roastty config_format_config`
- Add concrete tests proving:
  - `Config::default().vt_kam_allowed == false`;
  - `vt-kam-allowed = true` and `false` parse as booleans;
  - a bare `vt-kam-allowed` flag parses as true;
  - `vt-kam-allowed =` resets to the default false value;
  - invalid bool values are reported as diagnostics by `load_str`;
  - `format_config` emits `vt-kam-allowed` immediately after
    `osc-color-report-format`;
  - with `vt_kam_allowed = false`, enabling terminal KAM does not consume normal
    key input;
  - with `vt_kam_allowed = true` and terminal KAM enabled, normal key input is
    consumed and not written;
  - with `vt_kam_allowed = true` and terminal KAM disabled, normal key input is
    written;
  - updating config on an existing surface toggles the KAM gate without
    recreating the surface;
  - configured or default keybindings still consume before the KAM gate.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = `vt-kam-allowed` is represented faithfully on `Config`, round-trips
through config loading/formatting, and the surface key path matches upstream KAM
ordering with targeted and full tests passing.

**Partial** = the config field lands, but the runtime key gate requires a
follow-up.

**Fail** = the key gate cannot be implemented without first replacing the
surface input or terminal-mode plumbing.

## Design Review

Codex adversarial reviewer `019eb4e0-0d81-7ba0-8ea9-11bd21cdb717` returned
**Changes Required** with one required finding:

- The original design stored `vt_kam_allowed` only when a surface was created,
  but upstream rebuilds derived surface config during config updates. Accepted:
  this design now requires `Surface::apply_config` to refresh the stored KAM
  policy and requires an update-path test for an existing surface.

Codex adversarial reviewer `019eb4e1-e792-7c63-93c1-3463fe28388e` re-reviewed
the fix and returned **Approved** with no remaining findings. The reviewer
confirmed the design now requires `Surface::apply_config` refresh behavior and
an existing-surface update-path test.
