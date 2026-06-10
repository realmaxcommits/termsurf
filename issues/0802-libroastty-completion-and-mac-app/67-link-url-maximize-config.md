+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 67: Phase F — link URL and maximize config

## Description

Experiment 66 added the `scrollbar` config surface. The next upstream config
fields that can land as one small parser/formatter slice are:

- `link-url`
- `maximize`

Upstream declares `link-url: bool = true` immediately after the still-TODO
repeatable `link` field, and `maximize: bool = false` immediately after
`link-previews` in `vendor/ghostty/src/config/Config.zig`.

This experiment ports those two boolean config surfaces only: fields, defaults,
parsing/reset behavior, formatting, diagnostics, and focused tests. Runtime URL
hover/link activation behavior and startup window maximization are intentionally
out of scope because they depend on later link/action and app-window wiring.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::link_url: bool = true`.
  - Add `Config::maximize: bool = false`.
  - Route both keys through defaults, `Config::set`, `format_config`,
    clone/equality, and diagnostics.
  - Preserve local formatter order around the upstream sequence:
    - `scrollbar`
    - `link-url`
    - `link-previews`
    - `maximize`
    - `fullscreen`
  - Leave upstream `link` out of scope because upstream still marks it
    `TODO: This can't currently be set!` and a faithful port needs the
    repeatable link/action parser rather than a placeholder.

Out of scope:

- The repeatable `link` config surface and URL/action parser.
- Runtime URL matching, hover previews, and open-link action dispatch.
- Applying `maximize` to app/window creation.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/config/mod.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/67-link-url-maximize-config.md`
- Run targeted tests:
  - `cargo test -p roastty link_url_maximize_config`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - defaults are `link-url = true` and `maximize = false`;
  - explicit `true` and `false` values parse and format for both keys;
  - bare/missing CLI-style values set both bools to `true`;
  - empty values reset to their upstream defaults;
  - invalid values return `InvalidValue`;
  - `Config::load_str` records `ConfigDiagnostic` line/key/error entries for
    invalid `link-url` and `maximize` lines while keeping valid neighboring
    lines;
  - formatter order places `link-url` after `scrollbar`, `link-previews` after
    `link-url`, `maximize` after `link-previews`, and `fullscreen` after
    `maximize`;
  - clone/equality preserves both values.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = `link-url` and `maximize` are represented faithfully on `Config`,
round-trip through config loading/formatting, match upstream boolean parser
behavior, and have targeted and full tests passing.

**Partial** = one field lands faithfully but the other needs a follow-up, or a
parser/diagnostic/formatter-order edge remains before runtime use.

**Fail** = either field cannot be represented faithfully without first porting
broader link/action or app-window infrastructure.

## Design Review

Codex adversarial reviewer `019eb3d0-f897-73a2-846d-44d8a3565cd0` returned
**Approved** with no findings.

The reviewer verified that the README links Exp67 as `Designed`, the experiment
has the required sections, the scope is narrow, the planned `link-url` and
`maximize` defaults and ordering match upstream, and the verification plan
includes markdown/Rust formatting, targeted tests, full `cargo test -p roastty`,
`git diff --check`, and clean-status inspection.

## Result

**Result:** Pass

Experiment 67 added the config-only `link-url` and `maximize` surfaces to
`roastty/src/config/mod.rs`. `Config` now carries `link_url` with upstream
default `true` and `maximize` with upstream default `false`, routes both keys
through `Config::set`, and emits them in `format_config` in the local upstream
sequence:

- `scrollbar`
- `link-url`
- `link-previews`
- `maximize`
- `fullscreen`

The shared bool parser accepts explicit `true` and `false`, treats a missing
CLI-style value as `true`, resets empty values to each field's upstream default,
and reports `InvalidValue` for invalid strings. `Config::load_str` records
line/key diagnostics for invalid `link-url` and `maximize` lines while applying
valid neighboring lines.

Runtime URL matching/open-link behavior and startup window maximization remain
out of scope; this experiment does not alter link/action dispatch or app-window
creation.

Verification run:

- `cargo fmt -- roastty/src/config/mod.rs`
- `cargo test -p roastty link_url_maximize_config`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty`
- `cargo fmt --check`
- `git diff --check`
- `git status --short`

`cargo test -p roastty` passed with 4,502 unit tests, the C ABI harness, and doc
tests. The C ABI harness still emits existing enum-conversion warnings unrelated
to this config change.

## Conclusion

`link-url` and `maximize` now have faithful parser/formatter config surfaces
with defaults, reset behavior, diagnostics, formatter-order coverage, and
clone/equality coverage. The remaining repeatable `link` parser and the runtime
application of both URL matching and window maximization should stay separate,
larger follow-up work.

## Completion Review

Codex-native adversarial reviewer `019eb3d9-5fac-72b0-afd8-d96ae1160856`
returned **Approved** with no findings.

The reviewer checked the completed experiment with fresh context, including the
workflow contract, issue README, experiment file, implementation diff since the
plan commit, `roastty/src/config/mod.rs`, and upstream
`vendor/ghostty/src/config/Config.zig`. The reviewer independently verified
`cargo fmt --check`, `git diff --check`, both targeted test commands, and full
`cargo test -p roastty`, which passed with 4,502 unit tests, the C ABI harness,
and doc tests.
