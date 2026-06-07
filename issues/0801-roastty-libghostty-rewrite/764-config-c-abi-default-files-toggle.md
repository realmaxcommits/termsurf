+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 764: Config C ABI Default Files Toggle

## Description

Implement upstream's `config-default-files=false` discard behavior for the
public C ABI load sequence.

Upstream `Config.load` loads default files, then CLI args, then recursive files.
Inside `loadCliArgs`, Ghostty resets `config-default-files` to `true`, records
the replay boundary, applies CLI args, and if the CLI set
`config-default-files=false`, rebuilds the config from the replay steps after
the default-file layer. The practical effect is that default-file settings are
discarded while CLI-provided config still applies.

Roastty now has the individual ABI functions for default files, CLI args, and
recursive files, but the C ABI does not yet discard settings loaded by
`roastty_config_load_default_files` when a later CLI arg disables default files.
This experiment adds that behavior without implementing the full upstream replay
system.

## Changes

- `roastty/src/lib.rs`
  - Add a private parsed-config snapshot field to the C ABI `Config` wrapper.
  - Add a private ABI diagnostics length snapshot field to the C ABI `Config`
    wrapper.
  - In `roastty_config_load_default_files`, snapshot the parsed config state and
    diagnostics length before loading default files.
  - In `roastty_config_load_cli_args`, continue parsing keybind args once, then
    reset `config.parsed.config_default_files` to `true` before applying valid
    UTF-8 config-looking CLI args through `Config::set_cli_args`.
  - If CLI config args leave `config.parsed.config_default_files == false` and a
    default-file snapshot exists, restore the parsed snapshot, truncate ABI
    diagnostics to the pre-default-file diagnostics length, reset
    `config_default_files` to `true` again, and replay the valid UTF-8
    config-looking CLI args into that restored parsed config.
  - Surface diagnostics from the effective parsed CLI pass, preserving invalid
    UTF-8 and keybind diagnostics from the outer ABI scan.
  - Keep manual `roastty_config_load_file` state outside this discard behavior
    unless it existed before `roastty_config_load_default_files` was called.
  - Clone the snapshot state and diagnostics length in `roastty_config_clone`.
- Tests in `roastty/src/lib.rs`
  - default-file state is discarded when CLI contains
    `--config-default-files=false`;
  - CLI config state still applies after the default-file discard;
  - CLI `config-file` entries still feed recursive loading after the discard;
  - default-file diagnostics are removed after the default-file layer is
    discarded;
  - repeated CLI loads do not reuse stale `config-default-files=false` state
    when the later CLI pass omits the toggle;
  - invalid UTF-8, unknown config keys, and keybind diagnostics remain emitted
    once from the effective CLI/default-discard path;
  - prior manually loaded state that existed before default-file loading remains
    after the discard.

## Verification

- `cargo test -p roastty config_c_abi_default_files -- --nocapture --test-threads=1`
- `cargo test -p roastty config_c_abi_cli_config -- --nocapture --test-threads=1`
- `cargo test -p roastty config_ -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

The experiment passes if `config-default-files=false` removes settings loaded by
the default-file ABI step while preserving CLI config args, keybind behavior,
recursive CLI config files, diagnostics from the effective CLI pass, and any
explicit pre-default state.

## Design Review

Codex reviewed the initial design and found two blocking issues:

- restoring only the parsed config would leave stale default-file diagnostics in
  the ABI diagnostics list after the default-file layer is discarded;
- `config_default_files` must be reset to `true` before each effective CLI parse
  so stale `false` from a prior load or clone does not trigger an unintended
  discard.

The design was updated to snapshot/truncate the ABI diagnostics boundary around
default-file loading, reset `config_default_files` before the initial CLI parse
and before replay after discard, and add tests for diagnostic rollback and stale
toggle behavior. The replay input is defined as valid UTF-8 config-looking args,
including args that may produce parsed config diagnostics.

Codex re-reviewed the revised design and approved it with no blocking findings.
The review confirmed that the diagnostic rollback and `config_default_files`
reset blockers were resolved. Non-blocking implementation guidance: keep keybind
and invalid UTF-8 diagnostics outside parsed-config replay so they are not
duplicated, and use one path for surfacing the effective parsed CLI diagnostics.

## Result

**Result:** Pass

Implemented default-file discard behavior for the public C ABI config load
sequence. `roastty_config_load_default_files` now snapshots the parsed config
state and ABI diagnostics boundary before loading default files. The CLI loader
resets `config_default_files` before parsing config-looking CLI args, restores
the pre-default snapshot and removes default-file diagnostics when CLI sets
`--config-default-files=false`, then replays the valid UTF-8 config-looking CLI
args into the restored parsed config.

The rollback boundary is consumed by the first effective CLI parse after default
files are loaded, whether or not that CLI parse disables default files. This
prevents a later CLI load from reusing an old pre-default snapshot.

Verification passed:

- `cargo test -p roastty config_c_abi_default_files -- --nocapture --test-threads=1`
- `cargo test -p roastty config_c_abi_cli_config -- --nocapture --test-threads=1`
- `cargo test -p roastty config_ -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Completion Review

Codex reviewed the completed implementation and found one blocking issue in the
first pass: rollback state remained armed after a non-discarding CLI load, so a
later CLI load with `--config-default-files=false` could restore an old
pre-default snapshot and discard already-accepted CLI state/diagnostics.

The implementation was fixed by clearing the default-file snapshot and
diagnostics boundary on the non-discarding path too, making the rollback
boundary one-shot after default files are loaded. A regression test now covers
the sequence: load default files, run a CLI load that omits
`--config-default-files=false`, then run a later CLI load that includes it.

Codex re-reviewed the fixed implementation and approved it with no blocking
findings. Non-blocking follow-ups from the review: add
clone-while-snapshot-armed coverage, and add a focused test that loading default
files again establishes a fresh rollback boundary after the first one was
consumed.

## Conclusion

The C ABI config load sequence now handles the major upstream
`config-default-files=false` behavior without a full replay engine. Default-file
settings and diagnostics can be discarded by CLI while CLI config args,
recursive CLI config files, keybind diagnostics, invalid UTF-8 diagnostics, and
pre-default manual state remain intact.
