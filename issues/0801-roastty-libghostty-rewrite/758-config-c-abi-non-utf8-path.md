+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 758: Config C ABI Non-UTF-8 Path

## Description

Add a regression test for the lossless C path conversion introduced in
Experiment 757. `roastty_config_load_file` accepts a null-terminated C path and
converts it with Unix `OsStrExt::from_bytes`, matching upstream's byte-slice
path behavior instead of assuming UTF-8. The implementation exists, but the
completion review noted that a Unix-only non-UTF-8 path test would pin that
behavior.

This experiment is test-only. It does not change the ABI implementation, add
diagnostic formatting, change default-file loading, or add C harness coverage.

## Changes

- `roastty/src/lib.rs`
  - Add a unit test that creates a temporary config file with a filename
    containing invalid UTF-8 bytes.
  - Pass that path to `roastty_config_load_file` using the same C path byte
    helper as the existing ABI file-load tests.
  - Assert that the file loads successfully, produces no diagnostics, and syncs
    `confirm-close-surface = always` into app/surface close-confirm behavior.

## Verification

- `cargo test -p roastty non_utf8 -- --nocapture --test-threads=1`
- `cargo test -p roastty config_load_file -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

The experiment passes if a non-UTF-8 Unix path reaches the config loader
losslessly through the C ABI and the existing file-load behavior remains intact.

## Design Review

Codex reviewed the design and approved it for the plan commit with no blocking
findings. The review confirmed that the experiment is appropriately narrow,
test-only, Unix-only in practice, and directly covers the Experiment 757
follow-up: create a config file with invalid UTF-8 bytes in the filename, pass
its raw bytes through the C path helper, call `roastty_config_load_file`, and
verify the parsed setting is applied with no diagnostics.
