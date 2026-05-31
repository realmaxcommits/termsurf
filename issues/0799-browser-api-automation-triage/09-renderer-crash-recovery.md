# Experiment 9: Add Renderer Crash Recovery UX

## Description

Experiment 1 classified renderer crash UX as `Automatable after setup`.
Experiments 2-8 have now built the harness and resolved the higher-priority
browser API surfaces ahead of it.

The current TermSurf crash behavior is too generic:

- Chromium's `TsTabObserver` handles load failures with
  `DidFailLoad(...) -> LoadingState(state="error")`.
- It does not override
  `WebContentsObserver::PrimaryMainFrameRenderProcessGone(...)`.
- Roamium forwards only the generic loading error.
- webtui clears the progress indicator for `LoadingState("error")`, but it does
  not show a tab-crashed state or prove same-tab recovery.

This experiment adds a dedicated protocol-visible renderer crash event and an
automated recovery probe. The goal is not to prevent renderer crashes; it is to
make them visible, non-confusing, and recoverable without killing Roamium or the
socket session.

Chromium already provides a deterministic renderer-crash trigger for this exact
kind of test: typed navigation to `chrome://crash/`. TermSurf's
`TsBrowserMainParts::NavigateTab(...)` uses `PAGE_TRANSITION_TYPED`, so the
existing `Navigate` message can trigger the crash without manual UI.

## Changes

1. Create a new Chromium branch.

   In `chromium/src`, fork from:

   ```text
   148.0.7778.97-issue-799-exp8
   ```

   Name the new branch:

   ```text
   148.0.7778.97-issue-799-exp9
   ```

   Add it to `chromium/README.md` with a description such as:

   ```text
   Add renderer crash recovery event.
   ```

2. Extend `termsurf.proto`.

   Add a new Chromium-to-client event after HTTP auth:

   ```text
   RendererCrashed renderer_crashed = 39;
   ```

   Add a message:

   ```text
   message RendererCrashed {
     int64 tab_id = 1;
     string termination_status = 2;
     int32 termination_status_code = 3;
     string url = 4;
     bool can_reload = 5;
   }
   ```

   `termination_status` should be a stable lowercase string derived from
   `base::TerminationStatus`, for example:
   - `normal_termination`
   - `abnormal_termination`
   - `killed`
   - `crashed`
   - `still_running`
   - `killed_by_oom`
   - `oom_protected`
   - `launch_failed`
   - `oom`
   - `integrity_failure`
   - `evicted_for_memory`
   - `unknown`

   The integer code is included for debugging but the UI and harness must prefer
   the string. Do not add an `exit_code` field in this experiment; the
   `WebContentsObserver` callback available here exposes only the termination
   status. If a future experiment needs exit codes, it can move to a
   `RenderProcessHostObserver`-based implementation.

3. Add a Chromium crash notification callback.

   In `chromium/src/content/libtermsurf_chromium/`:
   - add `TsNotifyRendererCrashed(...)` next to the existing `TsNotify*`
     callbacks;
   - add `ts_set_on_renderer_crashed(...)` to the public C API;
   - register the global callback in `libtermsurf_chromium.cc`;
   - override
     `TsTabObserver::PrimaryMainFrameRenderProcessGone(base::TerminationStatus status)`;
   - map the status to the stable string from step 2;
   - include the current visible URL from `web_contents()->GetVisibleURL()` or
     the controller's visible entry if that is safer at the callback point;
   - emit `RendererCrashed` with `can_reload=true` for all non-normal primary
     main-frame renderer exits.

   Also send `LoadingState(state="error")` or otherwise clear active loading
   progress at the same point. The dedicated crash event carries the crash
   semantics; the loading state keeps existing progress-bar behavior from
   getting stuck.

   Do not treat subframe crashes as a tab crash in this experiment. The callback
   selected here is specifically for the primary main frame.

4. Route the event through Roamium and Wezboard.

   In Roamium:
   - add the FFI callback binding;
   - register it during startup;
   - convert the callback into a `RendererCrashed` protobuf;
   - route it to connected clients just like `LoadingState`, `TitleChanged`, and
     `ConsoleMessage`.

   In Wezboard:
   - add the message name for logging;
   - route `RendererCrashed` to the pane's TUI when a pane mapping exists;
   - if no pane mapping exists, log and drop the event, as console messages do.

5. Add webtui crash state.

   In `webtui`:
   - add `CompositorMessage::RendererCrashed`;
   - parse top field 39 in the IPC reader;
   - store the latest crash state separately from console messages;
   - clear the crash state only when a new recovery load begins after the crash
     event, or when that recovery load completes;
   - render a concise viewport message when the browser is ready but the current
     tab is crashed, for example:

     ```text
     Renderer crashed
     Press Cmd+R to reload, or enter a new URL.
     ```

   The status bar should also show the latest crash status if no warning/error
   console message is newer. Do not add a modal prompt, native UI, automatic
   reload, or new keybinding in this experiment.

   Be careful not to clear the crash state on stale post-crash events from the
   crashed navigation. Chromium can send `UrlChanged(chrome://crash/)` or a
   trailing `LoadingState("done")` around the crash sequence. Those events must
   not hide the user-visible crash message. Track whether a `loading` event was
   observed after `RendererCrashed`; only that subsequent load, or its matching
   `done`, may clear the crash state. A plain `UrlChanged` by itself is not
   enough to clear it.

6. Extend the Issue 799 harness.

   Add a focused probe named:

   ```text
   renderer-crash-recovery
   ```

   The probe should use the existing fake GUI connection and no manual
   screenshots:
   1. Create a normal local page that reports `ready`.
   2. After the ready report and `TabReady`, send a `Navigate` message to
      `chrome://crash/`.
   3. Wait for a `RendererCrashed` event.
   4. Verify the event includes:
      - the current tab id;
      - `termination_status` equal to `crashed` or another explicit non-normal
        status if Chromium reports a different deterministic status on macOS;
      - nonzero `termination_status_code`;
      - a nonempty URL containing `chrome://crash`;
      - `can_reload=true`.
   5. Verify a loading `error` event is also observed, so progress state clears.
   6. Send a second `Navigate` message in the same tab to a local post-crash
      page.
   7. Verify the post-crash page reports successfully.
   8. Verify Roamium stays alive, the socket stays connected until cleanup, and
      no bad-Mojo/missing-binder signature appears.

   Classification should be `renderer_crash_recovered` only when the crash event
   and the same-tab post-crash navigation both succeed. A crash event without
   successful same-tab recovery is `renderer_crash_unrecovered`, not `Pass`.

   The existing generic log scanner currently treats crash signatures as a
   failed probe. For this one probe only, expected renderer-crash log lines from
   `chrome://crash/` should be recorded as expected evidence rather than
   classified as a harness failure. Browser-process crashes, socket loss before
   recovery, bad Mojo, missing binders, or Roamium exit still fail the probe.

   Implement this separation at every harness aggregation point:
   - split raw crash matches into `expected_crash_lines` and
     `unexpected_crash_lines` for `renderer-crash-recovery`;
   - set `result["crashed"]` from unexpected crash lines only;
   - keep `expected_crash_lines` in the probe result for auditability;
   - make `classify_probe(...)` ignore expected crash lines but still fail on
     unexpected crash lines;
   - compute the full-run `status="crash"` from unexpected crashes only;
   - keep existing behavior unchanged for every other probe.

   The expected crash matcher must be narrow. It may match Chromium's
   intentional renderer crash lines for `chrome://crash/`, but it must not
   whitelist browser process crashes, bad-Mojo termination, missing binder
   termination, socket disconnects, or Roamium process exits.

7. Run formatting and builds.
   - Run `cargo fmt` after Rust edits and accept all formatter output.
   - Run Chromium `clang-format` on edited C++ headers/sources.
   - Build Chromium with `autoninja -C out/Default libtermsurf_chromium`.
   - Build debug `roamium`, `webtui`, and `wezboard`.

8. Regenerate the Chromium patch archive after a passing implementation.

   Use the standard Issue 799 patch archive:

   ```text
   chromium/patches/issue-799/
   ```

## Verification

1. Run the focused crash probe:

   ```bash
   python3 scripts/test-issue-799-browser-api-audit.py \
     --probe renderer-crash-recovery \
     --seconds 12
   ```

   Pass criteria:
   - classification is `renderer_crash_recovered`;
   - `RendererCrashed` was received exactly once for the tab;
   - termination status is explicit and non-normal;
   - the matching `LoadingState("error")` was observed;
   - post-crash same-tab navigation reports successfully;
   - Roamium remains alive until normal harness cleanup;
   - there are no bad-Mojo or missing-binder signatures.

2. Run the full Issue 799 harness:

   ```bash
   python3 scripts/test-issue-799-browser-api-audit.py
   ```

   Pass criteria:
   - all prior probes remain green;
   - `missing_interfaces` is empty;
   - `empty_interfaces` is empty;
   - the only expected crash evidence is from the `renderer-crash-recovery`
     probe;
   - the overall run status remains `completed`.

3. Inspect the generated artifacts:
   - `probe-results.json`;
   - `probes/renderer-crash-recovery/probe-result.json`;
   - `messages.log`;
   - `roamium.stderr`.

   The artifacts must show the ordered sequence:

   ```text
   initial ready report
   Navigate chrome://crash/
   RendererCrashed
   LoadingState error
   Navigate post-crash local page
   post-crash report
   ```

4. Run a compile-level webtui check by building `webtui`. Manual visual testing
   is not required for this experiment unless the protocol and harness pass but
   the crash-state render code cannot be reasoned about from the source.

## Failure Criteria

The experiment fails if:

- it only sends generic `LoadingState("error")` and does not add a dedicated
  renderer crash event;
- the event is emitted for subframe crashes as if the whole tab crashed;
- Roamium exits or the socket disconnects before the harness can recover the
  same tab;
- the harness accepts a crash event without proving same-tab post-crash
  navigation;
- browser-process crash evidence is treated as an expected renderer crash;
- webtui shows no user-visible crash state;
- the implementation adds automatic reload, a modal prompt, a native crash
  dialog, or a new keybinding;
- prior Issue 799 probes regress.

## Non-Negotiable Invariants

- Do not suppress or hide renderer crashes.
- Do not claim renderer stability; this is a recovery/visibility experiment.
- Do not change browser API probe expectations except for the new intentional
  crash probe's classification.
- Do not add native UI.
- Do not add manual verification requirements unless automation proves
  insufficient.
- Do not use `ninja`; Chromium builds must use `autoninja`.
- Run `cargo fmt` after Rust edits and accept its output.
