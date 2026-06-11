+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 79: Phase F — undo timeout config

## Description

Experiment 78 wired `quit-after-last-window-closed-delay`. The next unported
upstream config field after `initial-window` is:

- `undo-timeout`

Upstream declares this as `Duration = .{ .duration = 5 * std.time.ns_per_s }` in
`vendor/ghostty/src/config/Config.zig`. It controls how long macOS undo
operations remain available; Linux does not support undo operations and ignores
the setting.

This experiment adds the config parser/formatter surface only. Runtime undo
stack expiration behavior and binding/action integration are out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::undo_timeout: Duration` with upstream default `5s`.
  - Route `undo-timeout` through `Config::set`, config loading diagnostics,
    clone/equality, and formatting.
  - Parse values through the existing `Duration::parse_cli` path.
  - Reset empty values to the default `5s`.
  - Preserve the current local formatter convention by inserting the key after
    the app-lifecycle block that currently contains `initial-window`,
    `quit-after-last-window-closed`, and `quit-after-last-window-closed-delay`.

Out of scope:

- Runtime undo stack expiration.
- Keybinding/action behavior that consumes the undo timeout.
- `quick-terminal-position`, the next upstream field after `undo-timeout`.
- Any broader formatter reordering of already-ported keys.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/79-undo-timeout-config.md`
- Run targeted tests:
  - `cargo test -p roastty undo_timeout_config`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - the default formats as `undo-timeout = 5s`;
  - a valid duration such as `1m 30s` parses and formats as the normalized
    duration string;
  - `0` remains accepted by the existing duration parser and formats back to an
    empty duration payload;
  - an empty value resets the field to the default `5s`;
  - a missing value returns `ValueRequired`;
  - invalid duration values return `InvalidValue`;
  - `Config::load_str` records diagnostics for invalid neighboring
    `undo-timeout` lines while preserving valid parsed values;
  - formatter order includes the key immediately after
    `quit-after-last-window-closed-delay`;
  - clone/equality preserves the duration.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the undo timeout field is represented faithfully on `Config`,
round-trips through config loading/formatting, matches the upstream default and
duration parser behavior, and has targeted and full tests passing.

**Partial** = the parser/formatter field lands, but duration reset, diagnostics,
or formatter-order coverage requires a follow-up.

**Fail** = the key cannot be represented faithfully without first implementing
runtime undo stack expiration behavior.

## Design Review

Codex adversarial reviewer `019eb461-d4b1-7090-b260-7f79a67b506b` returned
**Approved** with no required findings. The reviewer confirmed that the README
links Experiment 79 as `Designed`, the experiment has the required sections, the
scope is narrow and limited to `undo-timeout`, the plan matches upstream
default/parser/formatter behavior, the local formatter-order choice is explicit,
and the verification plan includes the required formatting, targeted test, full
test, hygiene, and status checks.
