# Experiment 3: Prove JavaScript Dialog Runtime Flow

## Description

Issue 816 still needs Ghostboard-specific runtime proof for browser interruption
flows. JavaScript dialogs are the next narrow slice because Issue 799 added
protocol-mediated `alert`, `confirm`, `prompt`, and `beforeunload` support in
Chromium/Roamium/webtui, while Issue 810 classified the current Ghostboard
evidence as only `Maybe`.

Static evidence suggests ordinary post-`BrowserReady` dialog requests should
work over webtui's direct Roamium socket. Ghostboard also ignores compositor
fallback dialog messages, and initial-load timing can lose browser events unless
the responsible component buffers or replays them. This experiment will prove
the normal direct path first, classify any missing behavior by owner, and only
fix code after the harness identifies a concrete failure.

## Changes

Planned investigation:

- Inspect the current JavaScript dialog path in:
  - `proto/termsurf.proto`;
  - `roamium/src/dispatch.rs`;
  - `roamium/src/ipc.rs`;
  - `webtui/src/ipc.rs`;
  - `webtui/src/main.rs`;
  - `ghostboard/src/apprt/termsurf.zig`;
  - the Issue 799 JavaScript dialog result in
    `issues/0799-browser-api-automation-triage/05-javascript-dialogs.md`.
- Confirm whether direct-client replay/buffering is needed for initial-load
  dialogs, separate from the loading-state replay fixed in Experiment 2.

Planned harness changes:

- Add a `javascript-dialog-smoke` scenario to
  `scripts/ghostboard-geometry-matrix.sh`.
- Serve local fixture pages that trigger:
  - delayed `alert("ISSUE816_ALERT")`;
  - delayed `confirm("ISSUE816_CONFIRM")` with accept and cancel cases;
  - delayed `prompt("ISSUE816_PROMPT", "default")` with accepted typed text and
    cancel cases;
  - an initial-load `alert("ISSUE816_INITIAL_ALERT")` before normal load
    completion;
  - `beforeunload` proceed and stay cases with explicit sticky-user activation
    evidence.
- Launch debug Ghostboard, debug webtui, and debug Roamium using the same
  no-installed-binary guarantees as `browser-state-smoke`.
- Capture app log, Roamium trace, webtui state trace, screenshots, and terminal
  input coordinates.
- Extend the test-only webtui state trace if needed so dialog request, rendered
  dialog state, key-driven reply, and restored mode are observable without
  relying on OCR.
- Drive replies with automated keyboard input in the actual Ghostboard window:
  - Enter accepts `alert`;
  - `y`/Enter accepts `confirm` and `beforeunload`;
  - `n`/Esc cancels `confirm` and `beforeunload`;
  - text editing plus Enter accepts `prompt`;
  - Esc cancels `prompt`.

Planned fix policy:

- If Roamium receives and sends a `JavaScriptDialogRequest` but webtui does not
  enter/render dialog mode or reply correctly, fix webtui.
- If Chromium/libtermsurf emits the request before any direct client can receive
  it, fix Roamium with an observed-request replay/buffer or fix the Chromium
  creation order. Do not pass by ignoring the initial-load dialog case.
- If webtui sends a reply but Roamium does not call the Chromium reply FFI, fix
  Roamium dispatch.
- If the direct path passes but Ghostboard compositor fallback is the only
  missing path, record that as a lower-priority resilience finding rather than
  broadening this experiment into fallback routing.

Planned issue-doc changes:

- Add this experiment to the Issue 816 README with status `Designed`.
- Record a per-dialog result table covering request evidence, rendered UI
  evidence, reply evidence, page-observed JavaScript result, and owner.
- Record any cases deferred for separate experiments, especially HTTP auth,
  renderer crash recovery, color scheme, and copy-current-URL.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/03-prove-javascript-dialogs.md`.
2. If Rust files change, `cargo fmt -- <changed-rust-files>`.
3. If Zig files change, `zig fmt <changed-zig-files>`.

Static/build checks:

1. `prettier --check --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/03-prove-javascript-dialogs.md`.
2. `bash -n scripts/ghostboard-geometry-matrix.sh`.
3. `cargo check -p webtui` if webtui changes.
4. `cargo build -p webtui` if webtui changes.
5. `cargo check -p roamium` if Roamium changes.
6. `./scripts/build.sh roamium` if Roamium changes.
7. `./scripts/build.sh chromium` only if Chromium changes.
8. `shellcheck scripts/ghostboard-geometry-matrix.sh` if available.
9. `git diff --check`.

Runtime checks:

1. `scripts/ghostboard-geometry-matrix.sh javascript-dialog-smoke`.
2. Confirm Roamium logs a `JavaScriptDialogRequest` for each triggered dialog
   with matching `tab_id`, `request_id`, `dialog_type`, origin, and message.
3. Confirm webtui records or renders dialog mode for each request before the
   automated reply.
4. Confirm webtui sends the expected `JavaScriptDialogReply` and Roamium logs a
   matching reply with `ok=true`.
5. Confirm the page observes the expected JavaScript return value:
   - alert resumes execution;
   - confirm accepted returns `true`;
   - confirm canceled returns `false`;
   - prompt accepted returns the typed text;
   - prompt canceled returns `null`;
   - initial-load alert resumes and normal page state becomes visible;
   - beforeunload proceed navigates away and beforeunload stay remains on the
     original page.

6. Confirm the terminal returns to the previous webtui mode after each reply and
   later browser state updates still reach webtui.

Pass criteria:

- Delayed alert, confirm accept/cancel, prompt accept/cancel, initial-load
  alert, beforeunload proceed, and beforeunload stay all pass with request,
  rendered UI, reply, and page-observed result evidence.
- Beforeunload cases include sticky-user activation evidence before navigation
  triggers the dialog.
- The harness contains durable assertions for each passing subcase.
- Any app code change is owned by the component proven responsible and is no
  broader than needed.

Partial criteria:

- The delayed dialog path passes but initial-load dialog delivery is missing and
  ownership is proven.
- Alert/confirm/prompt pass, but beforeunload needs a separate activation
  experiment.
- The owner is proven, but the fix requires Chromium branch work that cannot be
  completed in this experiment.

Fail criteria:

- The harness cannot distinguish request delivery, visible dialog state, reply
  delivery, and page-observed results.
- The scenario passes only by reading Roamium logs without proving webtui UI or
  page behavior.
- The implementation hides missing dialog delivery by weakening assertions,
  skipping initial-load dialogs, or relying on native OS dialogs.

## Design Review

Fresh-context adversarial review by Codex subagent `Locke`:

- **Initial verdict:** Changes required.
- **Required finding:** The design scoped JavaScript dialogs to include
  `beforeunload`, but allowed a full Pass if `beforeunload` was split into the
  next experiment.
- **Resolution:** Accepted. The design now requires `beforeunload` proceed and
  stay cases, including sticky-user activation evidence, for Pass. Deferring
  `beforeunload` remains only a Partial outcome.
- **Re-review verdict:** Approved. The reviewer confirmed the pass bar now
  requires beforeunload and no new required findings were introduced.
