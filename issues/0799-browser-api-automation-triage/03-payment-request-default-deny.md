# Experiment 3: Add PaymentRequest Default-Deny Binder

## Description

Experiment 2 built the browser API no-crash harness and identified the first
concrete automated implementation target:

```text
Empty binder for interface payments.mojom.PaymentRequest for the frame/document scope
```

The page-facing symptom was not a clean browser-level denial. The probe saw:

```text
UnknownError: Renderer process could not establish or lost IPC connection to the PaymentRequest service in the browser process.
```

Full native Payment Request support is out of scope for Issue 799. TermSurf does
not need to implement Chrome's payment sheet, stored cards, payment handlers, or
native payment UI here. The goal is narrower: replace Chromium Content's empty
`payments.mojom.PaymentRequest` binder with a TermSurf-owned default-deny
implementation that keeps the Mojo pipe healthy and answers the renderer with
clear unsupported/false responses.

This follows the same pattern as Issue 655's `BadgeService` stub and the PDF
work's scoped browser-service binders: implement the smallest browser-process
surface TermSurf needs, verify it automatically, and do not import broad Chrome
product stacks.

## Changes

1. Create a Chromium branch for Issue 799.

   In `chromium/src`, fork the current active TermSurf Chromium branch to:

   ```text
   148.0.7778.97-issue-799
   ```

   Add the branch to `chromium/README.md` with a description such as
   `Default-deny PaymentRequest binder`.

2. Add a TermSurf Payment Request stub.

   Add a narrow implementation under `content/libtermsurf_chromium/`, for
   example:

   ```text
   ts_payment_request.h
   ts_payment_request.cc
   ```

   The implementation should bind `payments::mojom::PaymentRequest` and own its
   receiver safely, using the local patterns already used by PDF Mojo stubs
   (`mojo::MakeSelfOwnedReceiver` or an equivalent self-owned receiver pattern).

   The stub should:
   - implement the full `payments::mojom::PaymentRequest` interface so future
     Chromium IDL additions fail at compile time instead of silently dropping
     methods;
   - store the `payments::mojom::PaymentRequestClient` remote passed to `Init`;
   - respond to `CanMakePayment()` with
     `payments::mojom::CanMakePaymentQueryResult::CANNOT_MAKE_PAYMENT`;
   - respond to `HasEnrolledInstrument()` with
     `payments::mojom::HasEnrolledInstrumentQueryResult::HAS_NO_ENROLLED_INSTRUMENT`;
   - respond to `Show(...)` with `PaymentErrorReason::NOT_SUPPORTED` and a clear
     short message such as `Payment Request is not supported in TermSurf`;
   - respond to `Abort()` with `OnAbort(false)` if the client is still bound;
   - treat `UpdateWith(...)`, `OnPaymentDetailsNotUpdated()`, `Complete(...)`,
     and `Retry(...)` as no-ops or explicit unsupported responses if Chromium's
     renderer contract requires a callback;
   - never show native UI;
   - never touch Chrome's payment app, autofill, card, profile, or native
     payment dialog code.

   If Chromium's renderer expects a different error mapping for unsupported
   embedders, document the reference evidence and use that mapping instead of
   guessing.

3. Register the binder in `TsBrowserClient`.

   In `TsBrowserClient::RegisterBrowserInterfaceBindersForFrame()`, after
   calling `ShellContentBrowserClient::RegisterBrowserInterfaceBindersForFrame`,
   add a TermSurf binder for `payments::mojom::PaymentRequest`.

   Chromium's `BinderMapWithContext::Add()` replaces earlier binders for the
   same interface, so this TermSurf binder should override Content's default
   empty binder from `content/browser/browser_interface_binders.cc`.

   Keep the registration frame-scoped. Do not add service-worker payment-app
   support, Payment Handler support, autofill integration, or a new TermSurf
   protocol message.

4. Update Chromium build deps narrowly.

   Add only the GN deps required for:
   - `payments::mojom::PaymentRequest`;
   - Mojo receiver/client types;
   - any base helpers used by the stub.

   The `payments::mojom::PaymentRequest` interface itself is generated from
   Blink's `third_party/blink/public/mojom/payments/payment_request.mojom`, so
   the expected primary dependency is in the `//third_party/blink/public/mojom`
   generated mojom targets, plus existing Mojo/base deps. The
   `//components/payments/mojom` target owns related payment data mojoms, but it
   is not sufficient by itself for the browser-side `PaymentRequest` interface.

   Do not depend on `//chrome/browser/payments`, `//components/autofill`, native
   views, or Chrome payment UI targets.

5. Extend the Issue 799 harness to verify the default-deny contract.

   `scripts/test-issue-799-browser-api-audit.py` already detects the current
   Payment Request failure, but it only constructs `PaymentRequest` and calls
   `canMakePayment()`. That is not enough to prove the stub methods are correct.

   Tighten the `payment-request` probe so that a pass specifically requires:
   - no `payments.mojom.PaymentRequest` empty binder line;
   - no `UnknownError` / IPC-loss message;
   - `canMakePayment()` resolves to `false`;
   - `hasEnrolledInstrument()` resolves to `false`;
   - `show()` rejects cleanly with the chosen explicit unsupported error
     (`NotSupportedError` is expected if the stub uses
     `PaymentErrorReason::NOT_SUPPORTED`);
   - classification is no longer `empty_binder`.

   This is still fully automated. Do not add native payment UI testing. Do not
   treat `show()` as a real payment-flow test; it is only a contract check that
   Blink receives a coherent unsupported response instead of an IPC failure.

6. Build Chromium with the project Chromium workflow.

   Use `autoninja`, never `ninja` directly:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the Roamium binary also needs rebuilding for the local test workflow, use
   the existing project build script rather than installing anything:

   ```bash
   ./scripts/build.sh chromium
   ```

7. Run the automated browser API harness.

   First run the narrow probe:

   ```bash
   scripts/test-issue-799-browser-api-audit.py --probe payment-request --seconds 8
   ```

   Then run the full suite:

   ```bash
   scripts/test-issue-799-browser-api-audit.py --seconds 8
   ```

   Record the run directories and the relevant `coverage-map.md`,
   `reference-coverage-map.md`, `probe-results.json`, and `binder-errors.tsv`
   findings in this experiment.

8. Archive Chromium patches only after a successful implementation.

   If the experiment passes, commit the Chromium branch, regenerate the Issue
   799 Chromium patch archive, update `chromium/README.md`, and record the main
   repo patch/docs changes.

   If the experiment is partial or fails, record the exact blocker and do not
   silently expand into Chrome's full payment stack.

9. Run Codex review before completion is accepted.

   After implementation and verification, run `codex-review` against:
   - this experiment file;
   - the Chromium diff;
   - the main repo diff;
   - the narrow Payment Request harness run;
   - the full browser API harness run.

   Ask Codex to verify that the stub is safe, narrow, correctly denies Payment
   Request, does not overclaim full payment support, and that the next action
   follows from the evidence. Fix real findings before marking this experiment
   `Pass`, `Partial`, or `Fail`.

## Verification

This experiment passes if:

- Chromium builds successfully with
  `autoninja -C out/Default libtermsurf_chromium`;
- `payment-request` no longer logs
  `Empty binder for interface payments.mojom.PaymentRequest`;
- `payment-request` no longer reports the IPC-loss `UnknownError`;
- `payment-request` resolves `canMakePayment()` to `false`;
- `payment-request` resolves `hasEnrolledInstrument()` to `false`;
- `payment-request` rejects `show()` with the explicitly chosen unsupported
  error, without opening native UI;
- the full Issue 799 harness still runs without manual input;
- other probes do not regress from Experiment 2's final classifications except
  for harmless log noise changes that are explained in the result;
- no native payment UI appears;
- no Chrome payment UI/autofill/payment-handler product stack is imported;
- Codex reviews the completed experiment and has no blocking findings.

This experiment is partial if:

- the binder compiles and removes the empty-binder log, but the renderer still
  reports a confusing error;
- `canMakePayment()` is fixed but `show()` / `abort()` expose a second missing
  method contract that needs a follow-up;
- GN dependencies pull in more payment infrastructure than expected and need a
  scope decision before landing;
- the harness needs a small follow-up to distinguish the exact unsupported
  error.

This experiment fails if:

- it implements or imports Chrome's full Payment Request product stack;
- it opens native payment UI;
- it changes `termsurf.proto`, Roamium FFI, Wezboard, or webtui for Payment
  Request;
- it suppresses or ignores the empty binder without giving Blink a coherent
  response;
- it breaks the existing `BadgeService`, permissions, notification, geolocation,
  file-system-access, service-worker, or PDF-related harness paths.

## Expected Outcome

The expected outcome is not "Payment Request works." The expected outcome is
"Payment Request fails cleanly and deterministically in TermSurf."

After this experiment, the Payment Request row in the Issue 799 harness should
move from:

```text
empty_binder / UnknownError IPC loss
```

to a clean unsupported/default-deny result, probably:

```text
exercised / canMakePayment false
```

If that happens, the next experiment should return to the Issue 799 queue from
Experiment 1. The likely next automated target is JavaScript dialogs or generic
downloads, unless the full harness exposes a more urgent crash or missing binder
after Payment Request is fixed.

## Result

**Result:** Pass

Implemented a TermSurf-owned `payments::mojom::PaymentRequest` default-deny stub
on Chromium branch `148.0.7778.97-issue-799`.

The Chromium implementation:

- adds `ts_payment_request.h` / `ts_payment_request.cc` under
  `content/libtermsurf_chromium/`;
- implements the full current `payments::mojom::PaymentRequest` interface;
- stores the `PaymentRequestClient` remote from `Init`;
- returns `CANNOT_MAKE_PAYMENT` from `CanMakePayment()`;
- returns `HAS_NO_ENROLLED_INSTRUMENT` from `HasEnrolledInstrument()`;
- rejects `Show()` with `PaymentErrorReason::NOT_SUPPORTED` and the message
  `Payment Request is not supported in TermSurf`;
- registers the frame-scoped binder in `TsBrowserClient`, after Content Shell
  and PDF binders, so it replaces Content's default empty binder;
- does not import Chrome payment UI, autofill, payment-handler, native dialog,
  or product payment-stack code.

The harness was extended so the `payment-request` probe now checks
`canMakePayment()`, `hasEnrolledInstrument()`, and `show()`, rather than only
checking that construction and `canMakePayment()` do not crash.

Verification:

```text
autoninja -C out/Default libtermsurf_chromium
```

Result: build succeeded.

Narrow probe:

```text
scripts/test-issue-799-browser-api-audit.py --probe payment-request --seconds 8
```

Run directory:

```text
logs/issue-799-browser-api-audit/20260530-230438
```

Observed result:

```json
{
  "classification": "exercised",
  "missing_interfaces": [],
  "empty_interfaces": ["blink.mojom.LCPCriticalPathPredictorHost"],
  "report": {
    "status": "resolved",
    "canMakePayment": false,
    "hasEnrolledInstrument": false,
    "show": {
      "status": "rejected",
      "errorName": "NotSupportedError",
      "error": "NotSupportedError: Payment Request is not supported in TermSurf"
    }
  }
}
```

The `binder-errors.tsv` for the narrow run contains no
`payments.mojom.PaymentRequest` row. The remaining
`blink.mojom.LCPCriticalPathPredictorHost` empty-binder line is unrelated
ambient page-load noise and was already present outside the Payment Request
failure.

Full suite:

```text
scripts/test-issue-799-browser-api-audit.py --seconds 8
```

Run directory:

```text
logs/issue-799-browser-api-audit/20260530-230453
```

Observed classifications:

```json
{
  "badge": "exercised",
  "permissions-query": "exercised",
  "notification-permission": "exercised",
  "geolocation-deny": "exercised",
  "credential-get-empty": "exercised",
  "webauthn-create": "blocked_needs_virtual_authenticator",
  "file-system-access": "blocked_user_activation",
  "payment-request": "exercised",
  "service-worker-basic": "exercised"
}
```

The full-suite `binder-errors.tsv` contains no `payments.mojom.PaymentRequest`
row. It still records `blink.mojom.CredentialManager` for the credential probe
and `blink.mojom.LCPCriticalPathPredictorHost` as ambient page-load noise,
matching the Experiment 2 baseline.

Chromium archival:

- Chromium branch committed as `2e628c059bb07`: `Default-deny PaymentRequest`.
- Patch archive regenerated under `chromium/patches/issue-799/`.
- `chromium/README.md` now records `148.0.7778.97-issue-799` as the current
  branch and includes it in the branch table.

Codex completion review:

- First completion review found no implementation defect, but correctly blocked
  completion until the experiment result, README status, Chromium commit, and
  patch archive were recorded.
- Those process blockers were resolved before finalizing this result.

## Conclusion

Payment Request no longer fails through Chromium Content's empty
`payments.mojom.PaymentRequest` binder in TermSurf. It now fails cleanly and
deterministically as unsupported: pages receive `false` capability answers and
an explicit `NotSupportedError` from `show()`, without native UI or Chrome
payment-product plumbing.

The next Issue 799 experiment should return to the automated triage queue and
select the next browser API surface from the Experiment 1/2 evidence. The most
likely candidates remain JavaScript dialogs, generic downloads, or a more
targeted follow-up for the remaining `CredentialManager` empty-binder evidence.
