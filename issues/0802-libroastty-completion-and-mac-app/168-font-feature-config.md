# Experiment 168: Phase F — font feature config

## Description

Remove `font-feature` from the remaining Phase F public-config tail.

Upstream defines `font-feature` as a repeatable string field immediately after
`font-synthetic-style` and before `font-size`. The syntax is intentionally
loose: the config layer stores feature-setting strings, while deeper font code
may later interpret or ignore invalid feature settings. This experiment wires
parser/formatter/storage behavior only.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config.font_feature: RepeatableString`.
  - Use the upstream default empty repeatable list.
  - Format `font-feature` in upstream declaration order after
    `font-synthetic-style` and before `font-size`.
  - Route `Config::set("font-feature", ...)` through the existing
    `RepeatableString` parser semantics: missing values report `ValueRequired`,
    empty values clear the list, non-empty values append, and CLI values append
    to file-loaded values.
  - Update config field-order/default tests and add a focused
    `font_feature_config_*` parse/format/reset/load/CLI-append/clone test.

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Mark Experiment 168 as `Designed`.
  - After result, update the Phase F remaining-public-options count from 21 to
    20 and remove `font-feature` from the remaining-tail wording if this passes.

## Verification

Before implementation:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

After implementation:

- `cargo test -p roastty font_feature_config`
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
- `cargo test -p roastty`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/168-font-feature-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = `font-feature` parses, formats, resets, loads, CLI-appends, clones,
and reports diagnostics with upstream default/order/repeatable-string semantics,
and the full roastty test suite passes.

**Partial** = the direct parser/formatter field lands, but ordering, replay
behavior, diagnostics, or full-suite verification remains incomplete.

**Fail** = the field cannot be added without conflicting with existing font
config storage, formatter ordering, or repeatable-string semantics.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Avicenna`, fresh
context.

**Verdict:** Approved with no findings.

The reviewer verified that the README links Experiment 168 as `Designed`, the
experiment has the required sections, the scope is bounded to the single
`font-feature` public config field, upstream type/order match
`RepeatableString = .{}` between `font-synthetic-style` and `font-size`, and the
planned tests cover repeatable-string parser/formatter/storage semantics plus
hygiene checks.

## Result

**Result:** Pass

Roastty now stores, parses, and formats `font-feature` as an upstream
repeatable-string config field. The default is an empty repeatable list,
formatting emits the void `font-feature = ` line when unset, non-empty values
append in order, empty values clear the list, and missing values report
`ValueRequired`.

The formatter places `font-feature` after `font-synthetic-style` and before
`font-size`, matching upstream declaration order. CLI-provided feature settings
append to file-loaded feature settings, matching upstream's repeatable-string
behavior for `font-feature`; only the font-family repeatables get the special
first-CLI-value overwrite behavior.

The Phase F public-config tail is now 20 keys: font variation/metric/freetype
knobs, `input`, and `keybind`.

Verification:

- `cargo test -p roastty font_feature_config` passed 1 filtered unit test plus
  the ABI harness filter.
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
  passed 1 filtered unit test plus the ABI harness filter.
- `cargo test -p roastty` passed 4,864 Rust unit tests, 0 failed, 4 ignored; the
  C ABI harness passed with the existing enum-conversion warnings; doc tests
  passed with 0 tests.
- `cargo fmt --check -p roastty` passed.
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/168-font-feature-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
  passed.
- `git diff --check` passed.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Raman`, fresh context.

**Verdict:** Approved after two required upstream-fidelity fixes.

The first completion review found that the initial implementation incorrectly
gave `font-feature` the special CLI overwrite behavior that upstream applies
only to `font-family`, `font-family-bold`, `font-family-italic`, and
`font-family-bold-italic`. The review also found that the direct CLI path set
`font_feature.overwrite_next` without clearing it, which could leak into a later
parse.

The fix removed `font-feature` from both CLI overwrite paths, updated the
focused test to assert upstream append semantics and no stale overwrite flag,
and corrected the experiment/README wording from CLI overwrite to CLI append.
The re-review approved those fixes with no remaining required findings.

## Conclusion

The `font-feature` public config surface is complete at the
parser/formatter/storage layer. Applying feature strings to actual font shaping
remains font/text runtime behavior, not a missing config field.
