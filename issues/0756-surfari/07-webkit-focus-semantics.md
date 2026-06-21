# Experiment 7: Resolve WebKit focus semantics

## Description

Experiment 6 implemented real Cocoa/WebKit event forwarding for mouse click,
mouse movement, wheel scroll, keyboard input, inactive/blur behavior, and dark
color scheme. It remained **Partial** because `ts_set_focus(true)` made the
`WKWebView` the AppKit first responder and keyboard delivery worked, but the
page did not observe DOM-visible focus state. The smoke-test state stayed
`"focus":false` while `"blur":true`, `"key":"a"`, and other input evidence
passed.

This experiment should resolve that focus gap before Surfari Rust integration.
The goal is either to make `ts_set_focus(true)` produce reliable page-visible
focus state, or to prove that the current public `WKWebView`/Cocoa boundary
cannot do that and identify the exact WebKit/private API or source patch needed.

This experiment should not create the Surfari Rust binary, modify Ghostboard,
modify `termsurf.proto`, implement unrelated browser features, or change WebKit
source. If the focus investigation proves a WebKit source patch is necessary,
this experiment should record **Partial** and design that patch as the next
experiment.

## Changes

- Study WebKit's macOS focus handling in the local source tree, especially:
  - `Source/WebKit/UIProcess/API/mac/WKWebViewMac.mm`;
  - `Source/WebKit/UIProcess/mac/WebViewImpl.mm`;
  - `Tools/TestWebKitAPI/Helpers/cocoa/TestWKWebView.mm`;
  - WebKitTestRunner and DumpRenderTree focus/event-sending code.
- Identify the exact API path WebKit's tests or MiniBrowser use to make a
  `WKWebView` page focused enough that `document.hasFocus()` and DOM focus
  events reflect activation.
- Try focused implementation changes in `libtermsurf_webkit` only:
  - AppKit first responder/window activation ordering;
  - `acceptsFirstResponder` / focus ring / view-window ordering if relevant;
  - private `WKWebView` methods only if they are present in the local source and
    the experiment documents their risk;
  - deterministic page-side focus target behavior that still comes from real
    WebKit focus, not JavaScript faking.
- Keep the public `libtermsurf_webkit` header Roamium-compatible. If private
  smoke-test helpers are needed, keep them in the private smoke-test header.
- Extend the smoke test so it explicitly verifies:
  - `ts_set_focus(true)` causes `document.hasFocus()` or a DOM focus/focusin
    observation to become true;
  - keyboard input still reaches the page after focus;
  - `ts_set_focus(false)` or inactive GUI state causes DOM blur/inactive state;
  - all Experiment 6 passing input evidence remains intact.
- If WebKit source changes are required, do not edit `webkit/src` in this
  experiment unless the source patch is the explicitly chosen next step after
  design review. Instead, record **Partial** with:
  - exact public/private APIs tried;
  - exact WebKit source path that appears to own the missing behavior;
  - proposed WebKit branch/patch experiment.

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
  > logs/issue756-exp7-focus.log 2>&1
rc=$?
echo "SMOKE_EXIT_STATUS=$rc" >> logs/issue756-exp7-focus.log
```

The smoke log must prove:

- Experiment 6 lifecycle and input behavior still passes.
- `ts_set_focus(true)` produces page-visible focus state through real
  WebKit/Cocoa focus, not JavaScript faking.
- Keyboard input still reaches the page after focus.
- `ts_set_focus(false)` or inactive GUI state produces page-visible blur or
  inactive state.
- Unsupported APIs remain documented honestly.

Verify symbols/linkage and WebKit checkout state:

```bash
nm -gU surfari/libtermsurf_webkit/build/libtermsurf_webkit.dylib | rg ' _ts_|_ts_webkit_test' | sort
otool -L surfari/libtermsurf_webkit/build/libtermsurf_webkit.dylib | rg 'WebKit|JavaScriptCore|libtermsurf'
otool -L surfari/libtermsurf_webkit/build/smoke-test | rg 'WebKit|JavaScriptCore|libtermsurf'
git diff --check
git -C webkit/src status --short
git -C webkit/src rev-parse HEAD
git -C webkit/src rev-parse --abbrev-ref HEAD
```

**Pass** = focus is page-visible after `ts_set_focus(true)`, keyboard and all
Experiment 6 passing input behavior still work, the smoke test exits 0,
unsupported APIs remain explicit, and `webkit/src` remains unchanged.

**Partial** = the investigation identifies why focus is not page-visible and
proves that a private WebKit API or WebKit source patch is required. The result
must record the exact APIs/source paths tried and the next patch experiment.

**Fail** = the focus investigation regresses the Experiment 6 lifecycle/input
smoke test or cannot identify a concrete next step.

Before recording the result, capture:

```bash
git status --short
git -C webkit/src status --short
git -C webkit/src rev-parse HEAD
git -C webkit/src rev-parse --abbrev-ref HEAD
git -C webkit/src rev-parse --is-shallow-repository
```

The TermSurf worktree must contain only the intended library, smoke-test, docs,
and issue changes plus ignored `logs/` and build output.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

**Verdict:** Approved.

Required findings: none.

Optional/nit findings accepted:

- Tightened the source-edit wording so this experiment records **Partial** and
  designs a later patch experiment if WebKit source changes are required.
- The implementation result should record exact WebKit files/functions
  inspected, not only broad areas.
