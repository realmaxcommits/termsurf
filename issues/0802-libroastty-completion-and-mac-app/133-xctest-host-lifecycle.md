# Experiment 133: Phase G — XCTest host lifecycle

## Description

Make the copied Roastty macOS app's hosted unit-test gate deterministic after
Experiment 132 fixed the old config/menu assertion cluster. The remaining
failure mode is not a known Swift assertion: focused `RoasttyTests/ConfigTests`
hangs before an individual test frame, with the host process sampled in XCTest
IDE-session preparation.

This experiment should isolate and fix the CLI test-host lifecycle/session setup
path. It must not reopen the config/menu semantics solved in Experiment 132, and
it must not broaden into UI-test permissions, screenshots, or visual automation.

## Changes

- Inspect the current Xcode test setup:
  - `roastty/macos/Roastty.xcodeproj/xcshareddata/xcschemes/Roastty.xcscheme`
  - `roastty/macos/Roastty.xctestplan`
  - `roastty/macos/Roastty.xcodeproj/project.pbxproj`
  - `roastty/macos/build.nu`
- Add a deterministic non-UI unit-test runner path for `RoasttyTests`. Prefer a
  mechanical project/scheme/test-runner change over copied app logic changes,
  for example:
  - an explicit unit-test-only shared scheme or test plan that includes
    `RoasttyTests` but not `RoasttyUITests`;
  - CLI runner flags that remove avoidable IDE/session ambiguity, such as an
    explicit macOS destination, disabled parallel/concurrent testing for hosted
    unit tests, bounded result-bundle paths under `logs/`, and xcodebuild test
    timeouts where supported;
  - a `build.nu` option or default test-path adjustment that keeps UI tests
    opt-in via `--ui-tests`.
- Preserve the copied app's source behavior. App source edits are out of scope
  unless the investigation proves a test-only lifecycle hook is required; any
  such hook must be compile-time/test-only, minimal, and documented in the
  result.
- Keep generated or diagnostic artifacts out of the repo. If `.xcresult`,
  samples, spindumps, or xcodebuild logs are needed, write them under
  `logs/issue-0802/exp-133/` or `/tmp` and reference their paths in the result.
- Update this experiment's result, Issue 802 operating notes, and the Issue 802
  roadmap/checklist after verification.

## Verification

Pass criteria:

- `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/133-xctest-host-lifecycle.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `cargo fmt --check`
- `cargo build -p roastty`
- `cargo test -p roastty config_get_ -- --test-threads=1`
- `cargo test -p roastty config_trigger_ -- --test-threads=1`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/ConfigTests`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/MenuShortcutManagerTests`
- `cd roastty && macos/build.nu --action test`
- `git diff --check`

The focused `ConfigTests` and `MenuShortcutManagerTests` commands must produce
finalized xcodebuild results instead of hanging. The full non-UI macOS unit-test
gate must either pass or fail with concrete post-Experiment-132 assertions that
identify the next libroastty/app gap. If a hang remains, the result must include
fresh process, sample/spindump, and xcodebuild-result evidence proving where the
new blocker is.

Every `xcodebuild`, spawned `Roastty.app`, and helper process started by this
experiment must be cleaned up by exact PID or exact build-output path, never by
broad process-name matching. The result must record the cleanup method and a
post-run process check showing no experiment-spawned processes remain.

## Design Review

**Reviewer:** Codex-native adversarial subagents (`multi_agent_v1.spawn_agent`,
fresh context, `Darwin` then `Banach`)

**Verdict:** Approved after fixes

The initial design review returned **Changes Required** with two workflow
findings:

- The result-step maintenance list mentioned operating notes but omitted the
  Issue 802 roadmap/checklist update required by the issue process.
- The verification section omitted an explicit scoped
  cleanup/no-dangling-process requirement for hang-prone `xcodebuild` and hosted
  app processes.

Both findings were fixed in the design. The final re-review approved the plan
with no remaining required findings.

## Result

**Result:** Partial

Implemented the deterministic non-UI hosted XCTest runner path:

- `macos/build.nu` now refreshes `RoasttyKit.xcframework` from the current
  `roastty` Rust build before `Roastty` app build/test actions. This prevents
  the Swift app tests from linking stale `libroastty.a` after Rust ABI changes.
- CLI-driven test actions now pass an explicit native macOS destination
  (`platform=macOS,arch=arm64`) and disable parallel testing. This removes the
  previous destination/session ambiguity and makes hosted unit-test runs finish
  instead of hanging in XCTest IDE-session preparation.
- UI tests remain opt-in through `--ui-tests`; normal CLI test runs still skip
  `RoasttyUITests`.

Verification passed:

- `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/133-xctest-host-lifecycle.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `cargo fmt --check`
- `cargo build -p roastty`
- `cargo test -p roastty config_get_ -- --test-threads=1` — 36 passed.
- `cargo test -p roastty config_trigger_ -- --test-threads=1` — 12 passed.
- `git diff --check`

The macOS hosted XCTest commands now produce finalized xcodebuild results rather
than hanging:

- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/ConfigTests`
  — finished with a finalized `.xcresult` and 2 assertion issues in
  `uppercasedLetterShouldBeNormalized`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/MenuShortcutManagerTests`
  — finished with a finalized `.xcresult` and 4 assertion issues in the two menu
  shortcut tests.
- `cd roastty && macos/build.nu --action test` — ran 201 tests across 18 suites,
  then failed with the same 6 assertion issues.

The first post-fix probe proved the stale-kit problem: before refreshing
`RoasttyKit.xcframework`, `ConfigTests` still reported Exp-132-fixed keys such
as `resize-overlay`, `scrollbar`, `maximize`, and `focus-follows-mouse` as
`UnknownField`. After the runner refreshed the kit, those assertions passed.

The remaining failing assertions all share one concrete gap: `keybind` entries
loaded from config files still produce `UnknownField`, so
`TemporaryConfig("keybind=...")` does not populate the keybind store. The Rust
CLI/config-argument keybind path is covered by `config_trigger_`, but
`roastty_config_load_file` currently delegates to the parsed config loader and
does not route `keybind` file entries through `parse_config_keybind_entry` /
`store_keybind_entry`.

Diagnostic artifacts were written under `logs/issue-0802/exp-133/`:

- `probe-003-buildnu/` — focused `ConfigTests` via updated `build.nu`, finalized
  with 2 keybind assertions.
- `probe-004-menu-buildnu/` — focused `MenuShortcutManagerTests` via updated
  `build.nu`, finalized with 4 keybind assertions.
- `probe-005-full-buildnu/` — full non-UI macOS test gate via updated
  `build.nu`, finalized with 201 tests and 6 keybind assertions.
- `process-check-after-probes.txt` — exact process check that records the
  `ps -axo pid=,ppid=,command=` source, exact project/SYMROOT/app-path match
  criteria, and `NO MATCHES` for remaining experiment `xcodebuild` or exact-path
  hosted `roastty` processes.

No broad process-name cleanup was used.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, `Ptolemy`)

**Verdict:** Approved after fixes

The initial result review returned **Changes Required** because the cleanup
artifact recorded only a timestamp and did not prove the process-check criteria
or no-match result. I reran the post-run check through a separate temporary
script so the captured process table could not match the check command itself,
then rewrote `process-check-after-probes.txt` with the `ps` source, exact
project/SYMROOT/app-path match criteria, and explicit `NO MATCHES` output. The
re-review approved the completed result with no remaining required findings.

## Conclusion

The XCTest host lifecycle/session blocker is reduced to a deterministic
non-hanging runner path, and stale app-linked `RoasttyKit` artifacts are now
rebuilt before app build/test actions. The copied macOS unit-test gate now
reports real post-Experiment-132 failures: file-loaded `keybind` entries are not
wired into the app-facing config store. The next experiment should route
`keybind` lines loaded from config files through the existing keybind parser and
storage path, without changing copied Swift app behavior.
