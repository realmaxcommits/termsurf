# Experiment 22: Prove WebKit pointer injection

## Description

Experiment 21 proved the real app keyboard path end to end and localized the
remaining pointer gap. Ghostboard hit testing works, IPC forwarding works, and
Surfari receives mouse and wheel messages, but synthetic AppKit mouse/wheel
events delivered by `libtermsurf_webkit` do not become DOM `click` or `wheel`
events in the fixture page.

This experiment should stay focused on that boundary: Surfari's WebKit pointer
injection. It should not expand into split panes, tab switching, window
switching, restart, profile isolation, crash handling, or the full feature
matrix. The goal is to find and verify the smallest correct way to make
forwarded TermSurf pointer events produce page-visible WebKit pointer behavior
in the single-window, single-tab, single-pane real app case.

## Changes

- Study WebKit's own macOS testing and automation event paths before changing
  code:
  - `webkit/src/Tools/WebKitTestRunner/mac/EventSenderProxy.mm`;
  - `webkit/src/Tools/WebKitTestRunner/mac/UIScriptControllerMac.mm`;
  - `webkit/src/Source/WebKit/UIProcess/API/Cocoa/WKWebViewPrivate.h`;
  - relevant `WKWebView` private methods such as `_simulateMouseMove`,
    `_doAfterProcessingAllPendingMouseEvents`, and automation-event marking.
- Update `libtermsurf_webkit` pointer delivery only where evidence shows the
  current synthetic AppKit path is incomplete.
- Keep Ghostboard, WebTUI, protocol, and WebKit source changes out of scope
  unless the investigation proves the pointer failure cannot be solved in
  `libtermsurf_webkit`.
- Extend `scripts/test-issue-756-real-app-surfari-input-routing.sh` only as
  needed to produce stronger pointer evidence or better failure diagnostics.
- Preserve Experiment 21's keyboard proof. Keyboard must remain fatal if it
  regresses.

Possible implementation paths to test, in order of least invasive to most:

- Correct event construction details: window-relative coordinates, event number,
  graphics context, click count, pressed-button state, event phases, and
  pixel-vs-point deltas.
- Use WebKit private completion hooks such as
  `_doAfterProcessingAllPendingMouseEvents` so the harness waits for WebKit's
  asynchronous mouse processing rather than racing it.
- Mark forwarded events as synthesized for WebKit automation if the evidence
  shows WebKit filters unmarked synthetic events.
- Use the private WebKit2/UIProcess event path directly if AppKit dispatch to
  `WKWebView` is the wrong seam.

## Verification

Pass criteria:

- Build or confirm the required binaries:

```bash
surfari/libtermsurf_webkit/build.sh
cargo build -p surfari
cargo build -p webtui
cd ghostboard && zig build
```

- Run the real Debug `TermSurf.app` through
  `scripts/test-issue-756-real-app-surfari-input-routing.sh`.
- Preserve Experiment 21 keyboard evidence:
  - Ghostboard stays frontmost;
  - Surfari logs `key-event`;
  - the fixture page logs `kind=input value=a`.
- Prove at least one page-visible pointer behavior in the real Surfari WebKit
  view:
  - DOM `click` on the fixture click zone; or
  - DOM `wheel`/scroll on the fixture page; or
  - an equivalent page-visible pointer signal if the harness records why it is
    equivalent.
- Keep Surfari-side pointer evidence:
  - `mouse-event` for click; and/or
  - `scroll-event` for wheel.
- The harness must fail if page-visible pointer evidence is missing. It must not
  print final `PASS` after only proving that Surfari received an IPC pointer
  message.
- Run hygiene checks:

```bash
git diff --check
bash -n scripts/test-issue-756-real-app-surfari-input-routing.sh
prettier --check --prose-wrap always --print-width 80 \
  issues/0756-surfari/README.md \
  issues/0756-surfari/22-webkit-pointer-injection.md
```

Run formatting/checks for any source files touched:

```bash
cargo fmt -- <rust-files>
zig fmt <zig-files>
```

Result classification:

- `Pass` means page-visible pointer behavior is proven in the real app without
  regressing Experiment 21's keyboard proof.
- `Partial` means the exact remaining WebKit pointer boundary is narrowed but
  page-visible pointer behavior is still not proven.
- `Fail` means the experiment cannot reach the real Surfari overlay or cannot
  produce enough evidence to improve on Experiment 21's localization.

## Design Review

Adversarial design review returned `APPROVED` with no Required findings. The
reviewer confirmed that the README links Experiment 22 as `Designed`, the file
has the required Description, Changes, and Verification sections, the scope
follows Experiment 21's `Partial` result, the plan stays focused on WebKit
pointer injection, the verification requires page-visible pointer behavior
instead of Surfari IPC receipt alone, Experiment 21's keyboard proof remains a
regression requirement, and the plan commit had not already been made.
