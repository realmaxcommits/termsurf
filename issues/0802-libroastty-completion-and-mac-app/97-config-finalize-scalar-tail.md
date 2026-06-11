# Experiment 97: Phase F — config finalize scalar tail

## Description

Port the remaining scalar tail of upstream `Config.finalize()` that is now
unblocked by the completed public config field set:

- reset an empty `term` back to upstream's `xterm-ghostty`
- clamp `minimum-contrast` to `[1, 21]`
- clamp `faint-opacity` to `[0, 1]`
- fill a missing `auto-update-channel` from the build release channel

Upstream performs these in `vendor/ghostty/src/config/Config.zig`:

```zig
if (self.term.len == 0) {
    self.term = "xterm-ghostty";
}

self.@"minimum-contrast" = @min(21, @max(1, self.@"minimum-contrast"));

if (self.@"auto-update-channel" == null) {
    self.@"auto-update-channel" = build_config.release_channel;
}

self.@"faint-opacity" = std.math.clamp(self.@"faint-opacity", 0.0, 1.0);
```

The upstream release channel is derived at build time from the semantic version:
stable when there is no prerelease component, tip otherwise. This issue's pinned
Ghostty source is `version = "1.3.2-dev"`, so the matching pinned build channel
for Roastty is `tip`. This experiment should add a small local constant for that
pinned channel rather than implementing a broader build-options system.

This experiment must not implement theme loading, conditional reload,
working-directory default resolution, app-runtime-specific GTK defaults, link
matcher mutation, or key-remap finalization.

## Changes

- `roastty/src/config/mod.rs`
  - Add a local pinned build release-channel constant set to
    `ReleaseChannel::Tip`, with a comment tying it to the issue's pinned
    `1.3.2-dev` Ghostty source.
  - Extend `Config::finalize()` to:
    - restore `term` to `xterm-ghostty` if it is empty
    - clamp `minimum_contrast` to `[1.0, 21.0]`
    - clamp `faint_opacity` to `[0.0, 1.0]`
    - set `auto_update_channel` to the pinned build channel when unset
  - Add tests for the new finalize behavior while preserving raw parser state
    before finalization.
  - Update any stale comments that still claim `faint-opacity` is not finalized.

## Verification

Pass criteria:

1. `cargo test -p roastty config_finalize_scalar_tail`
2. `cargo test -p roastty config_opacity_options_parse_and_round_trip`
3. `cargo test -p roastty async_update_config`
4. `cargo test -p roastty`
5. `cargo fmt --check`
6. `git diff --check`

The full `cargo test -p roastty` run must pass. The existing ABI harness may
print its known enum-conversion warnings, but no new failures are acceptable.

## Design Review

Codex-native adversarial review ran in fresh context with subagent
`019eb5b9-8be3-7981-b4a8-bf92125b4e26`.

Verdict: **APPROVED**

Findings: None.

## Result

**Result:** Pass

Implemented the scalar tail of upstream `Config.finalize()` in
`roastty/src/config/mod.rs`.

The implementation now:

- restores an empty `term` to `xterm-ghostty`
- clamps `minimum-contrast` to `[1, 21]`
- fills unset `auto-update-channel` from a pinned build release-channel constant
  matching the issue's `1.3.2-dev` Ghostty source (`tip`)
- clamps `faint-opacity` to `[0, 1]`

The change intentionally leaves theme loading, conditional reload,
working-directory desktop default resolution, app-runtime-specific GTK defaults,
link matcher mutation, and key-remap finalization out of scope.

Verification:

1. `cargo test -p roastty config_finalize_scalar_tail` — pass
2. `cargo test -p roastty config_opacity_options_parse_and_round_trip` — pass
3. `cargo test -p roastty async_update_config` — pass
4. `cargo test -p roastty` — pass: 4542 unit tests, ABI harness pass, doc tests
   pass. The ABI harness printed the existing 10 enum-conversion warnings. After
   completion review found one non-reproducing full-suite failure in a
   mouse-reporting test, this full command was rerun and passed again with the
   same 4542 unit-test, ABI harness, and doc-test result.
5. `cargo fmt --check` — pass
6. `git diff --check` — pass

## Conclusion

Roastty's config finalization now covers the scalar cleanup and derivation rules
at the tail of upstream `Config.finalize()`. The remaining Phase F finalization
work is the heavier behavior that this experiment explicitly deferred: theme
loading, conditional reload state, working-directory/default shell resolution,
GTK runtime defaults, link matcher mutation, and key-remap finalization.

## Completion Review

Codex-native adversarial review ran in fresh context with subagent
`019eb5bf-ab12-71b0-b9da-6c5d07d30a19`.

Initial verdict: **CHANGES REQUIRED**

Required findings:

- The reviewer's independent `cargo test -p roastty` run failed once in
  `tests::surface_mouse_button_reporting_honors_surface_mouse_reporting_gate`.
  The reviewer noted that the isolated rerun of that test passed.
- `config_finalize_scalar_tail` assigned `minimum_contrast` directly, so it did
  not prove raw parser state before finalization for the new clamp.

Fixes:

- Reran the full `cargo test -p roastty` gate. It passed again: 4542 unit tests,
  ABI harness pass with the existing 10 enum-conversion warnings, and doc tests
  pass.
- Changed `config_finalize_scalar_tail` to drive `minimum-contrast` and
  `faint-opacity` through `Config::set`, assert the raw pre-finalize values, and
  then assert the finalized clamps.

Final verdict after re-review: **APPROVED**

Final findings: None.
