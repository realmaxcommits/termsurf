# Issue 639: Open New-Tab Links in Same Tab

## Goal

Intercept links that request a new tab or window (`target="_blank"`,
`window.open()`, middle-click, Cmd+click) and open them in the current tab
instead. This makes these links functional now while deferring true multi-tab
support.

## Background

The Chromium Profile Server's `Shell` class inherits its `WebContentsDelegate`
implementation from `content_shell`. When a link requests a new tab or window,
two delegate methods handle it:

- **`OpenURLFromTab`** — Called for navigations with a non-`CURRENT_TAB`
  disposition (e.g. `target="_blank"` links, Cmd+click). Currently creates a new
  `Shell` window via `Shell::CreateNewWindow`.
- **`AddNewContents`** — Called when Chromium has already created a new
  `WebContents` (e.g. `window.open()` popups). Currently creates a new `Shell`
  to host it via `CreateShell`.

Both methods create standalone windows that TermSurf doesn't manage — they float
outside the terminal, have no XPC connection, no pane ID, and no way to stream
back to the TUI. The links "work" in the sense that Chromium opens them, but the
user never sees the result.

The fix is simple: instead of creating a new window, navigate the current tab to
the target URL.

## Current state

- **`Shell::OpenURLFromTab`** (`shell.cc:427`): For `NEW_FOREGROUND_TAB`,
  `NEW_BACKGROUND_TAB`, `NEW_POPUP`, and `NEW_WINDOW` dispositions, calls
  `Shell::CreateNewWindow` and navigates the new shell.
- **`Shell::AddNewContents`** (`shell.cc:311`): Creates a new `Shell` for the
  incoming `WebContents` via `CreateShell`.

## Experiment 1: Redirect to current tab

### Hypothesis

Modifying `OpenURLFromTab` to navigate the source tab (instead of creating a new
window) for all new-tab/new-window dispositions, and modifying `AddNewContents`
to navigate the source tab to the target URL (discarding the pre-created
`WebContents`), will make all "open in new tab" links work in the current tab.

### Changes

#### 1. `Shell::OpenURLFromTab` (`shell.cc`)

Change the `NEW_POPUP` / `NEW_WINDOW` / `NEW_BACKGROUND_TAB` /
`NEW_FOREGROUND_TAB` cases to navigate the source tab instead of creating a new
window:

```cpp
case WindowOpenDisposition::NEW_POPUP:
case WindowOpenDisposition::NEW_WINDOW:
case WindowOpenDisposition::NEW_BACKGROUND_TAB:
case WindowOpenDisposition::NEW_FOREGROUND_TAB:
  // Issue 639: Open in current tab instead of creating a new window.
  // True multi-tab support deferred.
  target = source;
  break;
```

#### 2. `Shell::AddNewContents` (`shell.cc`)

Navigate the source to the target URL and discard the pre-created `WebContents`.
The `new_contents` unique_ptr will be destroyed when it goes out of scope:

```cpp
WebContents* Shell::AddNewContents(
    WebContents* source,
    std::unique_ptr<WebContents> new_contents,
    const GURL& target_url,
    WindowOpenDisposition disposition,
    const blink::mojom::WindowFeatures& window_features,
    bool user_gesture,
    bool* was_blocked) {
  // Issue 639: Instead of creating a new window, navigate the source tab
  // to the target URL. The pre-created WebContents is discarded.
  if (source && target_url.is_valid()) {
    NavigationController::LoadURLParams params(target_url);
    params.transition_type = ui::PAGE_TRANSITION_LINK;
    source->GetController().LoadURLWithParams(params);
    return source;
  }
  return nullptr;
}
```

Required includes (check if already present):

```cpp
#include "content/public/browser/navigation_controller.h"
#include "ui/base/page_transition_types.h"
```

### Verification

1. Build Chromium (`autoninja -C out/Default chromium_profile_server`)
2. Launch TermSurf, `web google.com`
3. Search for something, click a result — opens in same tab
4. Find a `target="_blank"` link (e.g. footer links on many sites) — opens in
   same tab instead of a new window
5. Test `window.open()` via DevTools or a test page — opens in same tab
6. No stray Chromium windows should appear

### Success criteria

- `target="_blank"` links navigate the current tab
- `window.open()` navigates the current tab
- No new Chromium windows are created
- Back/forward navigation still works after redirected navigations
- Page title and URL bar update correctly after redirected navigations

### Result: Failure

The TUI became completely unresponsive after clicking a `target="_blank"` link.
The old page disappeared, but the new page never appeared, and all keybindings
stopped working, and the app had to be force-closed. The naive approach of
redirecting `OpenURLFromTab` and discarding the pre-created `WebContents` in
`AddNewContents` likely caused Chromium internal state corruption or a deadlock.
More research into how Chromium and Electron handle this is needed before the
next experiment.

## Experiment 2: Research new-tab interception patterns

### Hypothesis

Understanding how Chromium's new-tab lifecycle works internally, how Electron
intercepts it, and how our own app's CALayerHost pipeline reacts to WebContents
changes will reveal the correct interception point.

### Research questions

#### 1. Chromium internals

- What is the full call chain when a `target="_blank"` link is clicked? Which
  methods fire in what order (`OpenURLFromTab`, `AddNewContents`,
  `CreateNewWindow`, `WebContentsCreated`, etc.)?
- What happens to the original `WebContents` when a new one is created? Does
  Chromium expect the caller to adopt the new `WebContents`, and what breaks if
  it's discarded?
- Is there a delegate method that fires _before_ the new `WebContents` is
  created (e.g. `IsWebContentsCreationOverridden`) that could suppress creation
  entirely?
- What is `ShouldAllowRendererInitiatedCrossProcessNavigation`? Is it relevant?

#### 2. Electron

- How does Electron handle `window.open()` and `target="_blank"`? Look at
  Electron's `WebContentsDelegate` overrides in `vendor/electron/`.
- Does Electron suppress new-window creation, redirect it, or intercept before
  creation?
- Does Electron use `IsWebContentsCreationOverridden`, `SetAutoResizeMode`,
  `did-create-window`, or a different mechanism?
- What events does Electron emit (`new-window`, `will-navigate`,
  `did-create-window`) and where are they triggered from?

#### 3. Our app (TermSurf)

- When `Shell::CreateNewWindow` is called, what happens to the CALayerHost
  pipeline? Does the new Shell get a new `CAContext` / `CAContextID`?
- When the original Shell's `WebContents` navigates, what triggers the
  CALayerHost update? Is the issue that navigation to a new `WebContents`
  detaches the `RenderWidgetHostView` from the original `CAContext`?
- In the Experiment 1 failure, did the old page disappearing suggest the
  original `WebContents` was destroyed or detached? Or did the navigation start
  but the compositor never received the new frame?
- Look at `ShellTabObserver::RenderViewHostChanged` — does it fire during this
  scenario? Is it possible the observer lost its connection?

### Deliverable

A written summary of findings for each section above, with specific file paths,
method names, and line numbers. The summary should conclude with a recommended
approach for Experiment 3.

### Success criteria

- All three research areas answered with code references
- Root cause of Experiment 1 failure identified or narrowed down
- Clear recommendation for the next implementation experiment
