# Experiment 5: Prove Renderer Crash Recovery

## Description

Issue 816 still needs Ghostboard-specific runtime proof for renderer crash
recovery. Issue 799 added protocol-visible `RendererCrashed` handling and a
deterministic crash trigger with `chrome://crash/`, while Issue 810 classified
current Ghostboard evidence as `Maybe`: the normal direct Roamium-to-webtui path
should work, but no current Ghostboard walkthrough proves the crash UI and
same-tab recovery behavior.

This experiment will add a focused Ghostboard runtime smoke for renderer crash
recovery. It will intentionally crash the renderer for the active tab, verify
that webtui receives and renders crash state through the direct browser socket,
then navigate the same tab to a normal local page and verify recovery.

## Changes

Planned investigation:

- Inspect the current renderer crash path in:
  - `proto/termsurf.proto`;
  - `roamium/src/dispatch.rs`;
  - `roamium/src/ipc.rs`;
  - `webtui/src/ipc.rs`;
  - `webtui/src/main.rs`;
  - `ghostboard/src/apprt/termsurf.zig`;
  - the Issue 799 renderer crash result in
    `issues/0799-browser-api-automation-triage/09-renderer-crash-recovery.md`.
- Confirm whether Roamium's stable trace needs an additional `renderer-crashed`
  line for the geometry harness. The existing stderr log is useful, but the
  harness should prefer the same stable Roamium trace file used by the prior
  Issue 816 experiments.
- Confirm whether webtui's test-only state trace needs a `renderer_crashed`
  event and crash-aware render fields so the harness can prove the TUI boundary
  without OCR. The experiment must not pass on Roamium evidence alone.

Planned harness changes:

- Add a `renderer-crash-smoke` scenario to
  `scripts/ghostboard-geometry-matrix.sh`.
- Serve a local fixture page that reports a unique ready marker and includes a
  recovery page with a unique title and console marker.
- Launch debug Ghostboard, debug webtui, and debug Roamium using the same
  no-installed-binary guarantees as the existing Issue 816 scenarios.
- Navigate the active browser tab to `chrome://crash/` using the webtui URL edit
  path or direct browser navigation, whichever gives the clearest user-boundary
  evidence.
- Capture app log, Roamium trace, webtui state trace, and screenshot evidence.
- Verify:
  - Roamium emits one `RendererCrashed` event for the active tab;
  - webtui records a durable `renderer_crashed` state-trace event for the same
    tab;
  - webtui render-state trace shows the crash state is active before recovery;
  - loading/progress state is not left stuck after the crash;
  - stale post-crash URL/loading/title events do not clear crash state before a
    new recovery load begins;
  - the same tab can navigate to the local recovery page afterward;
  - post-recovery URL/title/console evidence reaches webtui;
  - Roamium stays alive through the crash and recovery.

Planned fix policy:

- If Roamium receives the Chromium crash callback but webtui does not receive or
  render the event, fix Roamium dispatch or webtui parsing/rendering according
  to the proven owner.
- If the crash event reaches webtui but stale `UrlChanged`/`LoadingState` events
  hide the crash UI before a recovery navigation begins, fix webtui crash-state
  clearing.
- If same-tab recovery fails after the crash while the event path works, fix the
  owning navigation/recovery component.
- If the direct path passes but Ghostboard compositor fallback is the only
  missing path, record that as a lower-priority resilience finding rather than
  broadening this experiment into fallback routing.

Planned issue-doc changes:

- Add this experiment to the Issue 816 README with status `Designed`.
- Record crash event metadata, rendered/TUI crash evidence, recovery
  URL/title/console evidence, Roamium liveness, and owner.
- Record remaining Issue 816 gaps for later experiments, especially color scheme
  and copy-current-URL.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/05-prove-renderer-crash-recovery.md`.
2. If Rust files change, `cargo fmt -- <changed-rust-files>`.
3. If Zig files change, `zig fmt <changed-zig-files>`.

Static/build checks:

1. `prettier --check --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/05-prove-renderer-crash-recovery.md`.
2. `bash -n scripts/ghostboard-geometry-matrix.sh`.
3. `cargo check -p webtui` if webtui changes.
4. `cargo build -p webtui` if webtui changes.
5. `cargo check -p roamium` if Roamium changes.
6. `./scripts/build.sh roamium` if Roamium changes.
7. `./scripts/build.sh chromium` only if Chromium changes.
8. If Ghostboard Zig or non-`macos/` Ghostboard files change, run
   `cd ghostboard && zig build -Demit-macos-app=false`.
9. If Ghostboard app files change or a Ghostboard rebuild is needed, run
   `cd ghostboard && macos/build.nu --configuration Debug --action build`.
10. `shellcheck scripts/ghostboard-geometry-matrix.sh` if available.
11. `git diff --check`.

Design gate:

- This experiment file is plan-only until a fresh-context design review approves
  it.
- Record design review findings and fixes in this file.
- Commit the approved experiment plan before implementation begins.

Runtime checks:

1. `scripts/ghostboard-geometry-matrix.sh renderer-crash-smoke`.
2. Confirm the initial local page loads and reports its ready marker.
3. Confirm navigation to `chrome://crash/` is sent for the active tab.
4. Confirm Roamium records a `RendererCrashed` event with:
   - matching `tab_id`;
   - explicit non-normal termination status;
   - nonempty URL containing `chrome://crash`;
   - `can_reload=true`.
5. Confirm webtui records a `renderer_crashed` state-trace event for the same
   tab and render-state trace shows crash state active.
6. Confirm stale post-crash events do not clear the webtui crash state before a
   new recovery load begins.
7. Confirm loading/progress is not left stuck after the crash.
8. Confirm same-tab recovery navigation reaches the local recovery page and
   webtui receives the recovery URL/title/console marker.
9. Confirm Roamium stays alive until normal harness cleanup.

Pass criteria:

- Renderer crash event, webtui crash state, same-tab recovery navigation, and
  Roamium liveness all pass under debug Ghostboard.
- The harness contains durable assertions for the Roamium crash event, webtui
  crash event, webtui crash render state, stale-event non-clear behavior, and
  recovery.
- Expected intentional `chrome://crash/` renderer crash evidence is separated
  from unexpected browser-process crash, socket-loss, bad-Mojo, or
  missing-binder signatures.
- Any app code change is owned by the component proven responsible and is no
  broader than needed.

Partial criteria:

- The crash event reaches webtui, but same-tab recovery fails and ownership is
  proven.
- Roamium emits the event, but webtui crash UI or trace evidence is incomplete.
- The owner is proven, but the fix requires Chromium branch work that cannot be
  completed in this experiment.

Fail criteria:

- The harness cannot distinguish crash event delivery, visible/TUI crash state,
  recovery navigation, and Roamium liveness.
- The scenario passes only by reading Chromium/Roamium logs without proving
  webtui behavior.
- The implementation hides the crash by weakening assertions, skipping recovery,
  or treating browser-process crashes as expected renderer-crash evidence.

## Design Review

Fresh-context adversarial review by Codex subagent `Lovelace`:

- **Initial verdict:** Changes required.
- **Required finding:** The original design did not require strict webtui crash
  trace/render-state proof and could still pass from Roamium evidence plus weak
  rendering evidence.
- **Required finding:** The original design did not explicitly record the
  design-review and plan-commit gate before implementation.
- **Resolution:** Accepted both findings. The design now requires a durable
  `renderer_crashed` state-trace event, crash-aware render-state evidence,
  stale-event non-clear proof, and explicitly forbids passing on Roamium
  evidence alone. It also includes a design gate requiring fresh-context design
  approval, recorded findings/fixes, and a plan commit before implementation.
- **Re-review verdict:** Approved. The reviewer confirmed the prior findings
  were resolved and no new required findings were introduced.

## Result

**Result:** Pass

Implemented a focused renderer crash recovery smoke for debug Ghostboard.

Code changes:

- `roamium/src/dispatch.rs` now emits a stable `renderer-crashed` trace line
  when the existing Chromium renderer-crash callback fires.
- `webtui/src/main.rs` now records a test-only `renderer_crashed` state-trace
  event and includes `loading_bar_active`, `renderer_crash_active`,
  `renderer_crash_tab_id`, and `renderer_crash_status` in `render_state` trace
  lines.
- `scripts/ghostboard-geometry-matrix.sh` now has a `renderer-crash-smoke`
  scenario that:
  - serves an initial local page and a same-tab recovery page;
  - navigates the active tab to `chrome://crash/` through the webtui URL editor;
  - asserts Roamium's `renderer-crashed` trace for the active tab;
  - asserts webtui's `renderer_crashed` event and active crash render state;
  - asserts stale post-crash events do not clear crash state or restart loading
    before recovery;
  - navigates the same tab to the local recovery page;
  - asserts recovery URL/title/console/render-state evidence and Roamium
    liveness through recovery.

Verification:

- `prettier --write --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/05-prove-renderer-crash-recovery.md`
  — pass.
- `cargo fmt -- webtui/src/main.rs roamium/src/dispatch.rs` — pass.
- `bash -n scripts/ghostboard-geometry-matrix.sh` — pass.
- `cargo check -p webtui` — pass.
- `cargo build -p webtui` — pass.
- `cargo check -p roamium` — pass.
- `./scripts/build.sh roamium` — pass.
- `shellcheck scripts/ghostboard-geometry-matrix.sh` — not run; `shellcheck` is
  not installed on this VM.
- `git diff --check` — pass.
- `scripts/ghostboard-geometry-matrix.sh renderer-crash-smoke` — pass on the
  final run after tightening the status assertions.

Passing runtime evidence:

- Harness log:
  `logs/ghostboard-geometry-renderer-crash-smoke-harness-20260617-233913.log`.
- App log:
  `logs/ghostboard-geometry-renderer-crash-smoke-app-20260617-233913.log`.
- Roamium trace:
  `logs/ghostboard-geometry-renderer-crash-smoke-roamium-20260617-233913.log`.
- webtui state trace:
  `logs/ghostboard-geometry-renderer-crash-smoke-webtui-20260617-233913.log`.
- Screenshot:
  `logs/ghostboard-geometry-renderer-crash-smoke-screenshot-20260617-233913.png`.

Key passing assertions from the runtime smoke:

- Roamium recorded:
  `renderer-crashed tab=1 ... status=crashed code=3 url=chrome://crash/ can_reload=true`.
- webtui recorded `event=renderer_crashed` for the same tab.
- webtui render state showed `loading_bar_active=false` and
  `renderer_crash_active=true` with `renderer_crash_status=crashed` after the
  crash.
- No stale post-crash render state cleared `renderer_crash_active` before the
  recovery navigation.
- No stale post-crash `loading_state state=loading` appeared before recovery.
- The same tab recovered to the local recovery page and webtui recorded the
  recovery URL, title, console marker, and final render state with
  `loading_bar_active=false` and `renderer_crash_active=false`.
- Roamium recorded the recovery page title after the crash, proving browser
  liveness through same-tab recovery.

The first runtime attempt used an overly strict render-state regex that expected
`renderer_crash_active=false` before `title=Issue 816 Crash Recovery`, while the
trace writes `title` before crash fields. The trace already proved correct
behavior, so the assertion was fixed to match the field order and to require the
final recovery console marker and inactive loading state.

## Completion Review

Fresh-context adversarial completion review by Codex subagent `Huygens`:

- **Initial verdict:** Changes required.
- **Required finding:** The harness accepted any non-space renderer crash status
  (`status=[^ ]+`) even though the approved design required explicit non-normal
  termination status evidence.
- **Resolution:** Accepted. The harness now requires `status=crashed` in the
  Roamium `renderer-crashed` trace assertion, the webtui `renderer_crashed`
  event assertion, and the webtui active-crash `render_state` assertion.
- **Re-run evidence:**
  `scripts/ghostboard-geometry-matrix.sh renderer-crash-smoke` passed with the
  stricter assertions at timestamp `20260617-233913`.
- **Re-review verdict:** Approved. The reviewer confirmed the prior finding was
  resolved and no required findings remained.

## Conclusion

Renderer crash recovery is now proven under debug Ghostboard for the direct
Roamium-to-webtui state path. The active tab can be crashed with
`chrome://crash/`, webtui records and renders crash state without being cleared
by stale events, and the same tab can recover to a normal local page with
Roamium still alive.

This experiment did not require Ghostboard compositor changes or Chromium
changes. The remaining Issue 816 gaps should move to color scheme and
copy-current-URL coverage.
