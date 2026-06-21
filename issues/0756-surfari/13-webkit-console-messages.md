# Experiment 13: Implement WebKit console messages

## Description

Experiment 12 finished cursor callbacks, leaving DevTools, renderer crash
reporting, and console messages unsupported in `libtermsurf_webkit`. Console
messages are the next narrow API gap: the public C ABI already has
`ts_set_on_console_message`, and the issue background records the WKWebView
limitation that there is no native console-capture API.

This experiment should implement console message callbacks using the established
WKWebView-compatible approach from earlier TermSurf work: inject a
document-start `WKUserScript` that wraps `console.log`, `console.info`,
`console.warn`, and `console.error`, serializes the message payload, captures
useful source/line metadata, and posts it to a `WKScriptMessageHandler` owned by
`libtermsurf_webkit`.

This experiment should not implement DevTools, renderer crash reporting, the
Surfari Rust binary, Ghostboard integration, protocol changes, or new WebKit
source patches.

## Changes

- Study local WebKit script-message references:
  - `Tools/TestWebKitAPI/Helpers/cocoa/TestScriptMessageHandler.h`;
  - `Tools/TestWebKitAPI/Helpers/cocoa/TestScriptMessageHandler.mm`;
  - `Tools/TestWebKitAPI/Helpers/cocoa/TestWKWebView.mm`;
  - `Tools/MiniBrowser/mac/AppDelegate.m`.
- Add a Surfari-owned `WKScriptMessageHandler` implementation in
  `surfari/libtermsurf_webkit/src/libtermsurf_webkit.mm`.
- Configure each `WKWebViewConfiguration.userContentController` with:
  - a document-start `WKUserScript`;
  - a script message handler with a TermSurf-specific handler name, for example
    `termsurfConsole`.
- The injected script should:
  - preserve the original console methods and call them after reporting;
  - report `log`, `info`, `warn`, and `error`;
  - serialize arguments deterministically enough for smoke testing, including
    strings, numbers, booleans, null, arrays, and plain objects;
  - avoid throwing into page code if serialization or message posting fails;
  - include best-effort `line_number` and `source` fields from an
    `Error().stack` frame when available.
- The native handler should:
  - validate that the message body is a dictionary;
  - ignore malformed script messages without crashing;
  - call `g_callbacks.on_console_message` only when a callback is registered;
  - pass level, message, line number, and source through the existing C callback
    ABI.
- Extend the deterministic navigation smoke page with console calls that prove:
  - all four supported levels are reported;
  - multiple arguments are serialized into a stable message string;
  - object/array values do not become useless Objective-C descriptions;
  - source and line fields are present enough for the ABI, with line number
    greater than zero when WebKit exposes stack metadata.
- Extend the C smoke harness to register `ts_set_on_console_message`, capture
  the expected ordered console sequence, and fail on missing, duplicate, or
  mismatched messages.
- Keep Experiment 6-12 smoke coverage intact: lifecycle, navigation, resize,
  focus, mouse, scroll, keyboard, color scheme, JavaScript dialogs, HTTP auth,
  target URL hover, and cursor callbacks must still pass.
- Update `surfari/libtermsurf_webkit/README.md` so console messages move from
  unsupported to implemented only if the smoke proof passes.

## Verification

Start from a clean TermSurf repo root:

```bash
git status --short
git -C webkit/src status --short
git -C webkit/src rev-parse HEAD
git -C webkit/src rev-parse --abbrev-ref HEAD
git -C webkit/src rev-parse --is-shallow-repository
```

Build and run the smoke test:

```bash
surfari/libtermsurf_webkit/build.sh

mkdir -p logs
DYLD_FRAMEWORK_PATH="$PWD/webkit/src/WebKitBuild/Debug" \
surfari/libtermsurf_webkit/build/smoke-test \
  "$PWD/surfari/libtermsurf_webkit/test-content/index.html" \
  "$PWD/surfari/libtermsurf_webkit/test-content/navigation.html" \
  > logs/issue756-exp13-console-messages.log 2>&1
rc=$?
echo "SMOKE_EXIT_STATUS=$rc" >> logs/issue756-exp13-console-messages.log
exit $rc
```

The smoke log must prove:

- Experiment 6-12 evidence still passes.
- `ts_set_on_console_message` receives exactly the expected console sequence for
  `log`, `info`, `warn`, and `error`.
- The message string preserves multiple arguments in deterministic order.
- A structured object/array argument is serialized as JSON-like content useful
  to callers, not as an opaque Cocoa object description.
- The source string is non-empty for the deterministic smoke page.
- The line number is greater than zero when stack metadata is available. If the
  implementation cannot obtain line numbers reliably, the result must record the
  exact WebKit/stack limitation and keep line-number behavior marked partial.
- The smoke harness fails, rather than merely logging, if the callback count,
  level order, message content, source, or line-number expectations are not met.

Verify symbols/linkage and checkout state:

```bash
nm -gU surfari/libtermsurf_webkit/build/libtermsurf_webkit.dylib | rg ' _ts_|_ts_webkit_test' | sort
otool -L surfari/libtermsurf_webkit/build/libtermsurf_webkit.dylib | rg 'WebKit|JavaScriptCore|libtermsurf'
otool -L surfari/libtermsurf_webkit/build/smoke-test | rg 'WebKit|JavaScriptCore|libtermsurf'
git diff --check
prettier --check --prose-wrap always --print-width 80 \
  issues/0756-surfari/README.md \
  issues/0756-surfari/13-webkit-console-messages.md
git -C webkit/src status --short
git -C webkit/src rev-parse HEAD
git -C webkit/src rev-parse --abbrev-ref HEAD
git -C webkit/src rev-parse --is-shallow-repository
```

There is no project-configured formatter for Objective-C++ or C in
`surfari/libtermsurf_webkit`; keep those edits local-style consistent and use
`git diff --check` as the whitespace guard.

**Pass** = console callbacks work through the injected WebKit script-message
bridge, the smoke test exits 0, all prior evidence still passes, the README
reflects support, and `webkit/src` remains unchanged.

**Partial** = console callbacks work but line/source metadata is weaker than the
ABI expects, or robust value serialization needs a follow-up experiment. The
result must record the exact limitation and whether console support should stay
listed as unsupported or partially supported.

**Fail** = the implementation regresses prior lifecycle/input/browser-state
coverage, breaks page console behavior by not preserving original methods,
requires WebKit source changes without prior design, or cannot identify a
concrete next step.

## Design Review

Adversarial subagent review, fresh context, read-only.

Verdict: **Approved**. No findings.

The reviewer checked that the README links Experiment 13 as `Designed`, the
experiment has Description, Changes, Verification, and pass/partial/fail
criteria, the scope stays within Surfari console callbacks, the plan uses a
WKWebView-compatible `WKUserScript` plus `WKScriptMessageHandler` bridge without
requiring WebKit source patches, verification is concrete and non-vacuous, and
`git diff --check` plus Prettier checks pass for the issue docs.
