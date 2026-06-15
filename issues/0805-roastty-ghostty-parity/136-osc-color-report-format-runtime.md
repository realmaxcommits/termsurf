# Experiment 136: OSC Color Report Format Runtime

## Description

`RUNTIME-009B2B2B3B2B2` still groups other remaining terminal behavior effects.
One concrete unproven terminal config option in that gap is
`osc-color-report-format`.

Pinned Ghostty threads `osc-color-report-format` through `termio.DerivedConfig`
into `StreamHandler.osc_color_report_format`, updates it through
`StreamHandler.changeConfig`, and uses it when answering OSC color queries:

- `none` suppresses OSC 4/10/11/12 color query replies;
- `8-bit` reports `rgb:rr/gg/bb`;
- `16-bit` reports `rgb:rrrr/gggg/bbbb`.

Roastty already parses/formats `osc-color-report-format` and already answers OSC
4 palette and OSC 10/11/12 dynamic color queries, but the terminal response
format is currently hard-coded to 16-bit and is not wired from parsed runtime
config.

This experiment will split the remaining terminal row:

- `RUNTIME-009B2B2B3B2B2A`: **Oracle complete** for `osc-color-report-format`
  runtime effects on OSC palette and dynamic color query replies, including
  startup config and live config update wiring.
- `RUNTIME-009B2B2B3B2B2B`: **Gap** for other remaining terminal behavior
  effects.

This experiment will not claim broader color/theme runtime parity or unrelated
terminal behavior.

## Changes

- `roastty/src/terminal/terminal.rs`
  - Add terminal-owned OSC color report format state.
  - Add a runtime setter for config updates.
  - Use the configured format in OSC 4 palette query replies and OSC 10/11/12
    dynamic color query replies.
  - Add focused terminal tests proving default 16-bit replies, 8-bit replies,
    `none` suppression, and runtime format updates.
- `roastty/src/termio.rs`
  - Add OSC color report format to `TermioSpawnOptions`.
  - Pass it into `TerminalInitOptions`.
  - Add a PTY-backed runtime test proving a child-visible OSC color query reply
    uses the configured format.
- `roastty/src/lib.rs`
  - Thread parsed `Config.osc_color_report_format` into initial surface Termio
    spawn options.
  - Update existing live surfaces when app config changes so OSC color query
    replies use the latest parsed format.
  - Add or extend focused app/surface config tests for startup and update
    propagation.
- `issues/0805-roastty-ghostty-parity/osc_color_report_format_runtime_parity.py`
  - Add a static guard checking pinned Ghostty markers:
    `@"osc-color-report-format"`, `osc_color_report_format`, `changeConfig`,
    `self.osc_color_report_format = config.osc_color_report_format`, `.none`,
    `."8-bit"`, `."16-bit"`, and OSC color query response formatting.
  - Check Roastty markers for parser coverage, terminal owned report-format
    state, palette and dynamic query formatting, Termio spawn wiring, app config
    startup/update wiring, focused runtime tests, and the inventory split.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split `RUNTIME-009B2B2B3B2B2` into the OSC color report-format complete row
    and the reduced remaining-terminal gap row.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Regenerate from the inventory script.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-223 summary. It must remain `Gap`.
- Existing CFG-223 static guards that hard-code current runtime row counts or
  the remaining terminal gap row
  - Update expected counts after the split: 45 runtime rows, 38 Oracle complete
    rows, 40 closed rows, and 5 remaining runtime gaps.
  - Update references from the old remaining terminal gap row to
    `RUNTIME-009B2B2B3B2B2B`.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- Pinned Ghostty evidence shows `osc-color-report-format` is a configured format
  stored on `StreamHandler`, updated through `changeConfig`, and used to
  suppress or format OSC color query replies.
- Roastty terminal core uses the configured report format for OSC 4 palette
  query replies and OSC 10/11/12 dynamic color query replies.
- Default behavior remains 16-bit.
- `none` suppresses color query replies without suppressing color set/reset
  operations.
- `8-bit` emits `rgb:rr/gg/bb`; `16-bit` emits `rgb:rrrr/gggg/bbbb`.
- PTY-backed `Termio` runtime proves a child-visible color query response using
  parsed spawn options.
- Initial app/surface config and live config updates both propagate
  `osc-color-report-format` to the active terminal runtime.
- `RUNTIME-009B2B2B3B2B2A` is Oracle complete and cites the terminal, Termio,
  app/surface, and static guard evidence.
- `RUNTIME-009B2B2B3B2B2B` remains `Gap` for other remaining terminal behavior
  effects.
- `CFG-223` remains `Gap`.

Commands:

```bash
cargo test --manifest-path roastty/Cargo.toml terminal_stream_osc_color_report_format
cargo test --manifest-path roastty/Cargo.toml termio_osc_color_report_format
cargo test --manifest-path roastty/Cargo.toml surface_osc_color_report_format
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/osc_color_report_format_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
cargo fmt --manifest-path roastty/Cargo.toml
cargo fmt --manifest-path roastty/Cargo.toml --check
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/136-osc-color-report-format-runtime.md
git diff --check
```

Fail criteria:

- OSC color report format is only proven through parser/default tests.
- PTY-backed terminals still use a hard-coded 16-bit response regardless of
  config.
- Runtime config update changes stored config but not the active terminal report
  format.
- The experiment promotes unrelated color/theme behavior or unrelated terminal
  behavior from the remaining gap.
- CFG-223 is marked complete.

## Design Review

**Reviewer:** Codex adversarial subagent with fresh context.

**Verdict:** Approved.

The reviewer found no issues. It verified that the README links Experiment 136
as `Designed`, the design has the required sections, the scope is narrow to
`osc-color-report-format`, the plan matches pinned Ghostty's
`DerivedConfig`-to-`StreamHandler` wiring plus `none`/`8-bit`/`16-bit` OSC color
query behavior, and the verification plan includes focused tests, PTY coverage,
app/surface startup and live update propagation, inventory regeneration, static
guard, fmt, Prettier, and `git diff --check` while keeping CFG-223 as `Gap`.
