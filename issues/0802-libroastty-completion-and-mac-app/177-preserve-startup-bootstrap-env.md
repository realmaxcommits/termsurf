# Experiment 177: Phase C — preserve startup bootstrap env

## Description

Diagnose and fix the live smoke-marker gap found by Experiment 176.

Experiment 176 proved that the copied `Roastty.app` selects the real CoreVideo
display-link present driver and exits cleanly, but both live smoke captures
showed only the shell prompt instead of the harness marker. The harness writes a
temporary zsh bootstrap and launches each app with `ZDOTDIR` and
`XDG_CONFIG_HOME` in the app process environment. Ghostty preserves that
environment through shell-integration setup, so its zsh integration restores the
temporary `ZDOTDIR` and sources the bootstrap. Roastty currently starts termio
with only `Surface.env_vars` in `TermioSpawnOptions.env`; `PtyCommand` would
inherit the process environment, but shell-integration setup overwrites
`ZDOTDIR` before the child launches and cannot preserve the inherited value
because it is not visible in the explicit env vector. That loses the harness
bootstrap.

This experiment should make the termio child environment explicit and
upstream-shaped: start from the current app process environment, apply terminal
identity and shell-integration edits to that base, and then apply surface/config
env overrides last. That ordering lets zsh setup preserve the harness-provided
process `ZDOTDIR` as `ROASTTY_ZSH_ZDOTDIR` while preserving upstream's rule that
explicit surface env overrides win after integration. Then rerun the live A/B
smoke proof with strict evidence that the Roastty screenshot contains the
marker.

## Changes

- `roastty/src/termio.rs`
  - Treat `TermioSpawnOptions.env` as explicit env overrides, matching upstream
    embedded `env_override` behavior.
  - Add a small helper that builds the base spawn environment from the current
    process environment.
  - Apply terminal identity/features and shell-integration setup to the base
    inherited environment, then apply `TermioSpawnOptions.env` last.
  - Add focused termio tests proving inherited env reaches the child, explicit
    env overrides win after integration, and forced zsh integration preserves
    and sources an inherited bootstrap `ZDOTDIR`.

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After the run, mark it `Pass`, `Partial`, or `Fail`.
  - If the live marker renders in Roastty and the app still selects
    `present-driver=display-link reason=core-video`, update the Experiment 176
    dependency in the roadmap/experiment notes as appropriate. Do not check the
    broader render-thread item unless cursor-blink timer parity is also proven.

- `issues/0802-libroastty-completion-and-mac-app/177-preserve-startup-bootstrap-env.md`
  - Record the exact implementation, verification commands, live screenshot/log
    paths, result, conclusion, and AI completion review.

## Verification

Before implementation:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

Focused tests:

- `cargo test -p roastty termio_env -- --test-threads=1`
- `cargo test -p roastty zsh_integration -- --test-threads=1`

Regression checks:

- `cargo test -p roastty --test abi_harness`
- `cargo test -p roastty -- --test-threads=1`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/177-preserve-startup-bootstrap-env.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

Live proof:

- Rebuild the copied app:

  ```bash
  cd roastty && macos/build.nu --action build
  ```

- Run the smoke harness with display-link logging:

  ```bash
  scripts/roastty-app/stop-app.sh
  TERMSURF_AB_HOLD_SECONDS=10 \
  ROASTTY_PRESENT_DRIVER_LOG=1 \
    scripts/roastty-app/live-ab-smoke.sh \
      --recipe smoke \
      --comparison-region content \
      --max-mismatch-ratio 1 \
      --max-mean-channel-delta 255
  ```

- Assert the Roastty stderr log contains:

  ```text
  present-driver=display-link reason=core-video
  ```

- Assert a stronger marker oracle than the harness JSON. Prefer a machine check
  when feasible: OCR the new
  `/Users/ryan/.cache/termsurf/shots/roastty-ab-content-<stamp>.png`, or compare
  a marker row crop against Ghostty's marker-containing crop with a much tighter
  region. If machine OCR/comparison is unavailable, record a saved inspection
  note with the exact screenshot path, marker string, and visual observation.
  The harness JSON alone is not sufficient evidence because Experiment 176
  showed permissive thresholds can pass while the marker is absent.

- Prove no debug Roastty app PID remains:

  ```bash
  if pgrep -f 'roastty/macos/build/.*Roastty.app/Contents/MacOS/roastty'; then
    exit 1
  fi
  ```

**Pass** = focused tests prove inherited env and zsh bootstrap preservation, the
copied app rebuilds, the live Roastty content screenshot visibly contains the
smoke marker, stderr proves the CoreVideo display-link driver was selected, and
no debug Roastty app PID remains.

**Partial** = env/unit behavior is fixed but live capture still cannot prove the
marker, or the marker renders but display-link selection/cleanup cannot be
proved. Record the exact blocker and artifact paths.

**Fail** = the env fix breaks termio spawning, zsh integration, app build,
launch, live rendering, or cleanup.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Euler`, fresh context.

**Initial verdict:** Changes required.

Findings and fixes:

- Required: the first design applied `TermioSpawnOptions.env` before terminal
  identity and shell-integration setup, which was not faithful to upstream.
  Upstream treats embedded surface env as a final `env_override` after shell
  integration. Fixed by splitting inherited process env from explicit overrides:
  inherited env is the base, terminal identity/features and shell integration
  mutate that base, then `TermioSpawnOptions.env` is applied last.
- Optional: marker proof still depended on subjective visual inspection. Fixed
  by requiring a stronger oracle than the harness JSON: prefer OCR or a tighter
  marker-row comparison, and if that is unavailable record an explicit saved
  inspection note with screenshot path, marker string, and observation.

**Final verdict:** Approved.

Final findings: None.

## Result

**Result:** Pass.

Implemented the upstream-shaped termio environment ordering in
`roastty/src/termio.rs`:

- `TermioSpawnOptions.env` is now treated as explicit env overrides, matching
  upstream embedded `env_override` behavior.
- `Termio::spawn_with_options` starts from the inherited process environment,
  applies terminal identity/features and shell-integration setup, then applies
  explicit overrides last.
- Added test helpers and focused tests proving inherited env reaches the child,
  explicit env overrides inherited values, explicit env overrides terminal
  identity and shell-integration edits, stale inherited terminal identity values
  are still replaced by Roastty defaults, and inherited `ZDOTDIR` is preserved
  through zsh integration so the bootstrap `.zshenv` is sourced.

Focused verification:

- `cargo test -p roastty termio_env -- --test-threads=1` — **Pass**, 5 tests
  passed.
- `cargo test -p roastty zsh_integration -- --test-threads=1` — **Pass**, 2
  tests passed.
- `cargo test -p roastty spawn_with_options_sets_fallback_terminal_identity_without_resources -- --test-threads=1`
  — **Pass**.
- `cargo test -p roastty spawn_with_options_resource_identity_overwrites_inherited_env -- --test-threads=1`
  — **Pass**.

Regression verification:

- `cargo test -p roastty --test abi_harness` — **Pass**, 1 test passed; existing
  enum-conversion warnings remained.
- `cargo test -p roastty -- --test-threads=1` initially found two stale tests
  that still expected terminal identity to override explicit env. Updated those
  tests to distinguish inherited env from explicit final overrides, then reran:
  **Pass**, 4882 passed, 0 failed, 4 ignored; ABI harness and doc-tests also
  passed.
- `cargo fmt --check -p roastty` — **Pass**.
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/177-preserve-startup-bootstrap-env.md issues/0802-libroastty-completion-and-mac-app/README.md`
  — **Pass**.
- `git diff --check` — **Pass**.

Live proof:

- `cd roastty && macos/build.nu --action build` — **Pass**. The copied app build
  completed with `** BUILD SUCCEEDED **`; only the existing Swift actor,
  retroactive Sendable, linker deployment-target, and terminfo warnings
  appeared.
- `scripts/roastty-app/stop-app.sh && TERMSURF_AB_HOLD_SECONDS=10 ROASTTY_PRESENT_DRIVER_LOG=1 scripts/roastty-app/live-ab-smoke.sh --recipe smoke --comparison-region content --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  — **Pass**. The harness launched Ghostty PID `93530` and Roastty PID `93538`
  with marker `ISSUE802_AB_SMOKE_20260613-012013` and returned JSON verdict
  `PASS`.
- The new Roastty content screenshot
  `/Users/ryan/.cache/termsurf/shots/roastty-ab-content-20260613-012013.png`
  visibly contains `ISSUE802_AB_SMOKE_20260613-012013`, unlike Experiment 176's
  prompt-only captures.
- A targeted pixel oracle on the marker band passed:

  ```text
  region=(360, 760, 0, 70) new_bright=840 old_bright=0
  ```

  This counted bright text pixels in the right-hand part of the marker row for
  the fixed screenshot and compared it against the old prompt-only
  `/Users/ryan/.cache/termsurf/shots/roastty-ab-content-20260613-010655.png`.
  Full-image OCR and enlarged-row OCR were attempted first but were unreliable
  for the terminal font, so the pixel-band oracle plus saved screenshot
  observation is the recorded marker proof.

- `grep -n 'present-driver' /Users/ryan/.cache/termsurf/shots/roastty-ab-stderr-20260613-012013.log`
  returned `1:[roastty] present-driver=display-link reason=core-video`.
- The explicit cleanup check printed `no debug Roastty app PID remains`.
- Captured artifacts:
  - `/Users/ryan/.cache/termsurf/shots/ghostty-ab-content-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/ghostty-ab-crop-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/ghostty-ab-full-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/ghostty-ab-stderr-20260613-012013.log`
  - `/Users/ryan/.cache/termsurf/shots/roastty-ab-content-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/roastty-ab-crop-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/roastty-ab-full-20260613-012013.png`
  - `/Users/ryan/.cache/termsurf/shots/roastty-ab-stderr-20260613-012013.log`

## Conclusion

The missing live smoke marker was caused by Roastty's termio spawn path making
only `Surface.env_vars` explicit before shell integration. `PtyCommand` would
inherit process env, but zsh setup overwrote inherited `ZDOTDIR` without seeing
it, so the harness bootstrap `.zshenv` was lost. Building the explicit child env
from the process env first, then applying shell integration, then applying final
surface/config overrides matches upstream's base-env plus `env_override` shape
and restores the startup bootstrap.

The copied app now rebuilds, launches, selects the real CoreVideo display-link
driver, renders the live smoke marker, and exits cleanly. The Phase C
render-thread checkbox remains open because cursor-blink timer parity is still
unproven, but the Experiment 176 blocker is resolved.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Lovelace`, fresh
context.

**Initial verdict:** Changes required.

Findings and fixes:

- Required: `inherited_env()` originally used `std::env::vars()`, which can
  panic if an inherited environment key or value is not valid Unicode. Fixed by
  reading with `std::env::vars_os()` and filtering entries that cannot be
  represented in Roastty's current string-based explicit env vector. The process
  child still inherits non-Unicode entries normally through `Command` unless
  Roastty overrides a specific key. Added
  `termio_env_spawn_with_options_tolerates_non_unicode_inherited_environment` to
  prove a non-Unicode inherited value no longer panics the termio spawn path.

Follow-up verification after the fix:

- `cargo test -p roastty termio_env -- --test-threads=1` — **Pass**, 6 tests
  passed.
- `cargo test -p roastty zsh_integration -- --test-threads=1` — **Pass**, 2
  tests passed.

**Final verdict:** Approved.

Final findings: None. The reviewer confirmed that `vars_os()` plus fallible
conversion resolves the non-Unicode inherited-env panic risk, and that the new
regression test covers the failure mode.
