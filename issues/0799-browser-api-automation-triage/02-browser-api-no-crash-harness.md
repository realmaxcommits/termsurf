# Experiment 2: Build the Browser API No-Crash Audit Harness

## Description

Build the automated harness that Issue 655 said TermSurf needed: a repeatable
browser API probe suite that loads local pages in Roamium, exercises selected
Web Platform APIs, and classifies each API as:

- reached and completed with expected success/denial;
- reached and failed cleanly with a JavaScript exception;
- blocked before meaningful execution by lack of user activation, permissions,
  secure-context rules, or feature availability;
- crashed or killed the renderer due to a missing browser-side Mojo binder,
  delegate, or service.

This experiment is an automation and diagnostic foundation. It must not fix any
missing binder yet. The output should make the next implementation experiment
mechanical: if a missing binder is found, the next experiment targets that
specific interface; if no missing binder is found in the initial probe set, the
next experiment expands probe coverage or moves to the next in-scope feature.

The harness should use the same fake-GUI approach that proved PDF plumbing in
Issue 792: launch the debug Roamium binary directly, accept its TermSurf socket
registration, send `CreateTab`, serve local deterministic fixtures, and classify
from protocol messages plus Chromium/Roamium logs. It should not require a real
Wezboard window, OS screenshots, Screen Recording permission, Accessibility
permission, or manual clicking.

## Changes

1. Create `scripts/test-issue-799-browser-api-audit.py`.

   Use `scripts/test-issue-792-fake-gui.py` as the starting pattern, but keep
   the new script independent so it can evolve for browser API probes without
   adding PDF-specific assumptions.

   Required behavior:
   - launch `chromium/src/out/Default/roamium` with:
     - `--ipc-socket={run_dir}/gui.sock`;
     - `--user-data-dir={run_dir}/profile`;
     - `--no-sandbox`;
     - any required local-test flags discovered during implementation, recorded
       in the result;
   - listen as a fake TermSurf GUI;
   - accept `ServerRegister`;
   - send `CreateTab` for each probe page;
   - send `Resize` after `TabReady`;
   - capture all protobuf top-level message kinds in `messages.log`;
   - capture Roamium stdout/stderr in the run directory;
   - enable Chromium logging strongly enough for bad-Mojo messages to appear in
     captured artifacts, or record exactly where Chromium emitted the relevant
     messages;
   - run without a real Wezboard instance.

   The result must prove the logging path is active. If the harness cannot
   capture a known Chromium bad-message line or demonstrate where such lines
   would appear, the experiment is at most `Partial` because missing-interface
   extraction is a main deliverable.

2. Add a local fixture server inside the harness.

   The server should bind `127.0.0.1` on an automatically selected free port,
   serve probe pages over HTTP, and collect result beacons from the pages.

   Required endpoints:
   - `/probe/{name}.html` — HTML page for one probe;
   - `/worker/{name}.js` — service-worker scripts where needed;
   - `/report` — accepts JSON or query-string probe results and appends them to
     `reports.jsonl`;
   - `/summary` — optional debugging endpoint returning collected reports.

   Use `127.0.0.1` / `localhost` only. Do not use third-party sites.

3. Implement the initial automatically invokable probe set.

   Start with APIs that can be meaningfully invoked without native UI, hardware,
   external accounts, or human input:

   | Probe                     | JavaScript action                                                                                                          | Expected non-crash outcome                                               |
   | ------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
   | `badge`                   | `navigator.setAppBadge(1)` and `navigator.clearAppBadge()`                                                                 | resolves or rejects cleanly; no bad Mojo; confirms Issue 655 stays fixed |
   | `permissions-query`       | `navigator.permissions.query()` for `geolocation`, `notifications`, `camera`, `microphone`, and any supported stable names | resolves with a state or rejects cleanly                                 |
   | `notification-permission` | `Notification.requestPermission()` if available                                                                            | resolves/rejects cleanly; no native OS notification requirement          |
   | `geolocation-deny`        | `navigator.geolocation.getCurrentPosition()` with short timeout                                                            | denied, unavailable, timeout, or fake result; no renderer kill           |
   | `credential-get-empty`    | `navigator.credentials.get()` with benign options, if available                                                            | resolves `null` or rejects cleanly                                       |
   | `webauthn-create`         | `navigator.credentials.create({ publicKey: ... })` with a synthetic challenge, if available                                | rejects cleanly if user activation/authenticator is missing; no bad Mojo |
   | `file-system-access`      | guarded calls to `showOpenFilePicker()` / related APIs if exposed                                                          | rejects cleanly, usually due to no user activation; no bad Mojo          |
   | `payment-request`         | guarded construction and `canMakePayment()` for a deterministic test method if exposed                                     | resolves/rejects cleanly; no native payment UI                           |
   | `service-worker-basic`    | register a local service worker and have it call at least one safe API path                                                | registration succeeds/fails cleanly; no service-worker bad Mojo          |

   Important: if an API requires transient user activation and JavaScript
   rejects before reaching the browser service, classify that probe as
   `blocked_user_activation`, not as proof the browser service is safe. The
   result must clearly separate "no crash observed" from "binder actually
   exercised."

   Each probe must have its own timeout. If a renderer kill or browser crash
   prevents later probes from running in the same process, the harness should
   either restart Roamium for the remaining probes or classify the remaining
   probes as `not_run_after_crash` with the first failing probe and extracted
   interface recorded. An early crash must not look like a script-level
   automation failure.

4. Add bad-Mojo and crash classification.

   The harness must scan `roamium.stderr`, `roamium.stdout`, and any Chromium
   logs it captures for at least these patterns:
   - `Terminating render process for bad Mojo message`;
   - `No binder found for interface`;
   - `Empty binder for interface`;
   - `Received bad user message`;
   - `RenderProcessGone`;
   - `bad_message`;
   - `CHECK failed`;
   - `Received signal`;
   - renderer process exits or socket disconnects before the probe timeout.

   The classifier should extract the interface name when Chromium reports
   `No binder found for interface ...` or `Empty binder for interface ...`.
   Record that extracted interface in the machine-readable summary.

5. Produce machine-readable and human-readable artifacts.

   Each run should create a timestamped directory under:

   ```text
   logs/issue-799-browser-api-audit/
   ```

   Required artifacts:
   - `run.json` — command, binary paths, environment, Chromium version/branch if
     available, fixture base URL, start/end timestamps, and overall status;
   - `probe-results.json` — one object per probe with:
     - probe name;
     - page URL;
     - JavaScript result status;
     - whether the probe was known to require user activation;
     - whether the page sent a report;
     - loading/protocol status;
     - crash status;
     - bad-Mojo status;
     - extracted missing interface, if any;
     - classification;
   - `binder-errors.tsv` — one line per extracted missing-interface error;
   - `coverage-map.md` — human-readable table mapping probes to exercised APIs,
     result classifications, and next action;
   - `reference-coverage-map.md` — human-readable table mapping each targeted
     API family to reference and TermSurf evidence, even when no runtime crash
     occurs;
   - `reports.jsonl`, `messages.log`, `roamium.stdout`, `roamium.stderr`, and
     `http.log`.

   `reference-coverage-map.md` must include columns for:

   | Column                        | Meaning                                                                              |
   | ----------------------------- | ------------------------------------------------------------------------------------ |
   | JS API / feature              | The page-facing API or feature being considered                                      |
   | Expected browser-side surface | Mojo interface, `ContentBrowserClient` hook, delegate, or permission path if known   |
   | Reference evidence            | Chrome, headless, content shell, or Electron evidence for how the surface is handled |
   | TermSurf evidence             | Current TermSurf implementation or absence                                           |
   | Runtime probe status          | `exercised`, `blocked_user_activation`, `unsupported`, `not_run_after_crash`, etc.   |
   | Next action                   | Fix specific binder, expand harness, defer, or no action                             |

   This reference map is separate from pure runtime survival. A clean runtime
   probe only proves the tested JavaScript path survived; it does not by itself
   prove that TermSurf's binder/delegate coverage matches the reference
   embedders.

6. Keep the result diagnostic-only.

   Do not add Chromium binders, protocol messages, Roamium FFI, Wezboard UI, or
   webtui behavior in this experiment. If the harness finds a concrete missing
   binder, record it as the next experiment target instead of fixing it inside
   Experiment 2.

7. Add usage instructions to this experiment's result.

   The result should include the exact command that was run, for example:

   ```bash
   scripts/test-issue-799-browser-api-audit.py
   ```

   If the script supports useful flags such as `--roamium`, `--seconds`,
   `--probe`, or `--log-dir`, document them in the result.

8. Run Codex review before implementation is considered ready.

   After implementing the harness and before recording the final result, run
   `codex-review` against:
   - this experiment file;
   - the script diff;
   - one real run's `coverage-map.md`;
   - `probe-results.json`;
   - any extracted bad-Mojo lines.

   Ask Codex to check whether the harness classification is honest, whether any
   probe claims more than it proves, and whether the next experiment target
   follows from the evidence. Fix real findings before marking the experiment
   `Pass`, `Partial`, or `Fail`.

9. Update the issue README.

   When the result is recorded, update this experiment's README index line from
   `Designed` to `Pass`, `Partial`, or `Fail`.

## Verification

This experiment passes if:

- `scripts/test-issue-799-browser-api-audit.py` runs end-to-end without manual
  input on the current debug Roamium binary;
- the harness launches Roamium through a fake TermSurf GUI socket, not through a
  real Wezboard window;
- at least the initial probe set from Changes step 3 runs or is honestly
  classified as unsupported/unavailable;
- every probe receives one explicit classification;
- bad-Mojo and renderer-crash patterns are scanned and surfaced in
  `probe-results.json`;
- the run proves Chromium bad-message logging is captured strongly enough to
  extract `No binder found for interface ...` or
  `Empty binder for interface ...`, or records that interface extraction remains
  partial;
- if a missing or empty interface is reported, the interface name is extracted
  into `binder-errors.tsv`;
- `coverage-map.md` clearly states which probes exercised browser service
  plumbing and which only proved early JavaScript rejection;
- `reference-coverage-map.md` maps each targeted API family against
  Chrome/headless/content shell/Electron and current TermSurf evidence;
- every probe has a timeout, and a crash in one probe either triggers a Roamium
  restart for later probes or records later probes as `not_run_after_crash`;
- the result identifies the next experiment based on the harness output;
- Codex reviews the completed harness and no blocking findings remain.

This experiment is partial if:

- the fake-GUI harness works but only a subset of the initial probes can run;
- the probes run but cannot yet distinguish early JavaScript rejection from
  browser-service exercise for some APIs;
- Chromium/Roamium launches and logs are captured, but the classifier needs a
  follow-up to extract missing-interface names reliably;
- runtime probes work, but the reference coverage map is incomplete enough that
  "no missing binder found" cannot be interpreted as meaningful coverage;
- the harness exposes an immediate missing binder that prevents later probes
  from running in the same browser process, requiring the next experiment to fix
  that first.

This experiment fails if:

- it requires manual clicking, a real Wezboard window, native OS permissions, or
  screenshots to classify the initial probe set;
- it silently suppresses bad-Mojo failures;
- it implements browser API behavior instead of building the audit harness;
- it marks user-activation-blocked probes as safe browser-service coverage;
- it cannot produce durable artifacts under `logs/issue-799-browser-api-audit/`;
- Codex finds unresolved blocking flaws in the harness or result.

## Expected Outcome

The most useful outcome is a coverage map, not necessarily a green test run. A
`Pass` can still say "no missing binder found in the initial probes" if the
classification is honest and the next expansion is clear. A `Partial` can be
more valuable than a broad false pass if it identifies the first concrete
missing interface to fix.

The next experiment should be selected mechanically from the result:

- If a missing interface is found, design Experiment 3 to add the narrow
  TermSurf-owned stub/delegate for that interface and re-run this harness.
- If no missing interface is found but many probes are
  `blocked_user_activation`, design Experiment 3 to add a contained synthetic
  user-activation/input path for the harness.
- If the initial probe set is stable and meaningful, design Experiment 3 to
  expand coverage to downloads, dialogs, auth, console capture, or crash UX in
  the order established by Experiment 1.

## Result

**Result:** Pass

Experiment 2 built `scripts/test-issue-799-browser-api-audit.py`, an automated
fake-GUI browser API audit harness. The harness launches the debug Roamium
binary directly, serves local probe pages, accepts Roamium's TermSurf
`ServerRegister`, sends `CreateTab` / `Resize`, records page reports, scans
Chromium/Roamium logs for bad-Mojo and crash signatures, and writes durable
artifacts under `logs/issue-799-browser-api-audit/`.

No manual action, real Wezboard window, native macOS permission, screenshot, or
clicking was required.

The final verification run was:

```bash
scripts/test-issue-799-browser-api-audit.py --seconds 8
```

Artifacts:

```text
logs/issue-799-browser-api-audit/20260530-224921/
├── run.json
├── probe-results.json
├── binder-errors.tsv
├── coverage-map.md
├── reference-coverage-map.md
├── reports.jsonl
├── messages.log
├── roamium.stdout
└── roamium.stderr
```

Final classifications:

| Probe                     | Classification                        | Key evidence                                                                                             |
| ------------------------- | ------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `badge`                   | `exercised`                           | `navigator.setAppBadge()` / `clearAppBadge()` resolved; Issue 655 remains covered                        |
| `permissions-query`       | `exercised`                           | permissions query resolved for geolocation/notifications/camera/microphone                               |
| `notification-permission` | `exercised`                           | request resolved to `denied` without native UI                                                           |
| `geolocation-deny`        | `exercised`                           | geolocation returned a timeout rejection without renderer kill                                           |
| `credential-get-empty`    | `exercised`                           | rejected cleanly with `NotSupportedError`                                                                |
| `webauthn-create`         | `blocked_needs_virtual_authenticator` | probe reached WebAuthn but needs contained DevTools virtual-authenticator setup before claiming coverage |
| `file-system-access`      | `blocked_user_activation`             | rejected before browser-service coverage because no user gesture was present                             |
| `payment-request`         | `empty_binder`                        | JS saw browser-process IPC loss; Chromium logged `payments.mojom.PaymentRequest` as an empty binder      |
| `service-worker-basic`    | `exercised`                           | scoped local service-worker registration activated and unregistered cleanly                              |

Important log finding:

```text
Empty binder for interface payments.mojom.PaymentRequest for the frame/document scope
```

The harness also observed `blink.mojom.LCPCriticalPathPredictorHost` during
ordinary page loads and `blink.mojom.CredentialManager` during the credential
probe. Those are recorded in `binder-errors.tsv`, but they did not produce the
same page-facing IPC-loss failure in this run. The actionable failure is
`payments.mojom.PaymentRequest` because the page-facing Payment Request probe
produced:

```text
UnknownError: Renderer process could not establish or lost IPC connection to the PaymentRequest service in the browser process.
```

## Conclusion

The automated browser API audit foundation is working. It gives Issue 799 a
repeatable way to distinguish:

- working/cleanly rejected APIs;
- APIs blocked before real browser-service coverage by missing user activation;
- probes that need a better contained timeout or activation strategy;
- concrete browser-side binder gaps.

The first concrete automated implementation target is Payment Request. Full
native payment support remains out of scope, but TermSurf should not leave the
renderer talking to an empty `payments.mojom.PaymentRequest` binder that
surfaces as an IPC-loss `UnknownError`. Experiment 3 should add the narrowest
TermSurf-owned default-deny behavior for Payment Request, or prove from Chromium
reference code that a different contained denial path is the correct fix, then
rerun this harness.

The WebAuthn probe is not yet an implementation target. It needs contained
DevTools virtual-authenticator setup before it can prove browser-service
behavior. The service-worker probe now exercises cleanly.
