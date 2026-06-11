+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 77: Phase F — title report and image limit config

## Description

Experiment 76 wired `focus-follows-mouse`. The next unported upstream fields
after the already-ported clipboard group are:

- `title-report`
- `image-storage-limit`

Upstream declares `title-report` as `bool = false` and `image-storage-limit` as
`u32 = 320 * 1000 * 1000` in `vendor/ghostty/src/config/Config.zig`, after the
clipboard paste fields and before `copy-on-select`.

This experiment adds the config parser/formatter surface only. Runtime CSI 21 t
title reporting behavior and Kitty image storage enforcement are out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::title_report = false`.
  - Add `Config::image_storage_limit = 320_000_000`.
  - Route both keys through defaults, `Config::set`, `format_config`,
    diagnostics, clone/equality, and formatter-order tests.
  - Preserve upstream order after `clipboard-paste-bracketed-safe` and before
    `copy-on-select`:
    - `title-report`
    - `image-storage-limit`

Out of scope:

- Runtime CSI 21 t title-report handling.
- Runtime image protocol storage accounting/enforcement.
- `copy-on-select`.
- Mouse action fields after `copy-on-select`.
- `keybind` and `key-remap`.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/77-title-report-image-limit-config.md`
- Run targeted tests:
  - `cargo test -p roastty title_report_image_limit_config`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - defaults are `title-report = false` and `image-storage-limit = 320000000`;
  - `title-report` follows existing bool field semantics, including bare values,
    empty reset, and invalid diagnostics;
  - `image-storage-limit` parses decimal and Zig-style scalar integer prefixes,
    accepts zero, resets on empty values, returns `ValueRequired` on missing
    values, and returns `InvalidValue` on invalid or overflowing values;
  - `Config::load_str` records diagnostics for invalid neighboring bool/scalar
    lines while preserving valid values;
  - formatter order matches the upstream sequence around these fields;
  - clone/equality preserves both values.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = both fields are represented faithfully on `Config`, round-trip
through config loading/formatting, match upstream defaults and parser behavior,
and have targeted and full tests passing.

**Partial** = one field lands faithfully but a parser, diagnostic, or
formatter-order edge requires a follow-up.

**Fail** = these fields cannot be represented faithfully without first porting
runtime title-report or image storage behavior.

## Design Review

Codex adversarial reviewer `019eb449-50ef-7561-b62c-126b6363f4d8` returned
**Approved** with no required findings. The reviewer confirmed that the README
links Experiment 77 as `Designed`, the experiment has the required sections, the
scope is limited to the config parser/formatter surface, runtime title-report
and image storage behavior stay out of scope, and the plan matches upstream
defaults and order for `title-report` and `image-storage-limit`. The reviewer
also confirmed the verification plan covers defaults, parser/reset semantics,
diagnostics, formatter order, clone/equality, targeted tests, full
`cargo test -p roastty`, `cargo fmt --check`, and `git diff --check`.

## Result

**Result:** Pass

Implemented `title-report` and `image-storage-limit` in
`roastty/src/config/mod.rs`. `title-report` now uses the existing bool parser
and formatter path with upstream default `false`; `image-storage-limit` uses the
existing `u32` scalar parser/formatter path with upstream default `320000000`.
Both fields format after `clipboard-paste-bracketed-safe` and before
`copy-on-select`.

Verification passed:

- `cargo fmt`
- `cargo test -p roastty title_report_image_limit_config`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty`
  - 4514 unit tests passed
  - ABI harness passed with the existing enum-conversion warnings
  - doc tests passed
- `cargo fmt --check`
- `git diff --check`

## Conclusion

The title-report and image-storage-limit config surface now matches the upstream
defaults, parser behavior, diagnostics, formatter order, and clone/equality
expectations for this slice. The next upstream fields after this point are
already represented by the earlier copy/mouse behavior config slices, so the
next experiment should inspect the next unported field beyond that already
ported group.

## Completion Review

Codex adversarial reviewer `019eb44f-6f1e-7ac2-a8a9-2b48587f6e26` returned
**Approved** with no required findings. The reviewer confirmed that the
implementation is limited to `title-report` and `image-storage-limit`, matches
upstream defaults and order, uses the existing bool and `u32` scalar
parser/reset paths, and adds coverage for formatting, diagnostics,
clone/equality, and reset semantics. The reviewer also confirmed that the README
status and result docs match the current state and no result commit exists after
the plan commit.

The reviewer independently verified `cargo fmt --check`, `git diff --check`,
targeted tests, and full `cargo test -p roastty`.
