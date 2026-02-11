# Issue 413: One Profile to Two Profiles

## Goal

Convert the One Profile app (a Content Shell clone running at 60fps) into a Two
Profiles app that renders two isolated browser profiles side by side in a single
window at 60fps. Each change is a separate experiment. When something breaks, we
fix it before moving on.

## Background

### How we got here

Issues 407–412 explored multiple approaches to rendering two Chromium
`BrowserContext` instances in one window:

- **Issue 407** proved in-process Chromium works: multiple profiles coexist with
  full isolation, and a single WebContents renders at 60fps via Content Shell.
  But placing two WebContents in one window dropped both to 2fps.
- **Issue 408** studied Electron's patches. Three throttling bypass patches
  target `Hide()`, `WasOccluded()`, and `WasHidden()`.
- **Issue 409** tried applying all 147 Electron patches. They can't build
  without Electron's full build infrastructure (Node.js, custom DEPS, etc.).
- **Issue 410** applied the three throttling patches in isolation. The bypass
  calls had no effect — both panes still rendered at 2fps.
- **Issue 410.2** added logging and discovered that `Hide()`, `WasOccluded()`,
  and `WasHidden()` are **never called** on either view. The entire throttling
  hypothesis was targeting the wrong code path.
- **Issue 411** attempted a deferred view attachment fix (wait for
  `RenderFrameCreated` before adding the view to the window). WebContents B
  never appeared, and Shell A was still 2fps — even though Shell A uses the
  exact same `Shell::CreateNewWindow` code path that Content Shell uses for
  60fps.
- **Issue 412** took a step back and cloned Content Shell as "One Profile."
  Confirmed it runs at 60fps. Established a known-good baseline.

### What we know

1. Content Shell runs at 60fps with a single profile.
2. One Profile (a Content Shell clone) runs at 60fps.
3. Two Profiles runs at 2fps — both panes, including Shell A which goes through
   the standard Content Shell lifecycle.
4. The throttling patches (Issues 408–410) are irrelevant — `Hide()`,
   `WasOccluded()`, and `WasHidden()` are never called.
5. The deferred view attachment (Issue 411) is irrelevant — Shell A is 2fps
   despite going through the standard lifecycle.
6. Something about the Two Profiles app's setup degrades Shell A's rendering.
   The candidates are: `TwoProfilesMainParts` class, `SHELL_DIR_USER_DATA`
   override, second `ShellBrowserContext`, second `WebContents`, or view
   hierarchy manipulation.

### The key architectural problem

In Content Shell and One Profile, **Chromium owns the window**. The `Shell`
class creates the NSWindow, manages the toolbar, and places the WebContents view
as the sole occupant of the content area. This works perfectly for one profile.

To render two profiles side by side, we need to **own the window ourselves** so
we can place two WebContents views into it. This is the single biggest
architectural change. Chromium's Shell class assumes one WebContents per window,
and its visibility tracking, compositor lifecycle, and platform delegate all
reflect this assumption.

## Branch

Create a new branch `146.0.7650.0-issue-413` in the `termsurf-chromium`
submodule, starting from the `146.0.7650.0-issue-412` branch (which has the One
Profile app at 60fps). Each experiment is a commit on this branch.

## Approach

Start from One Profile (60fps) and make one change at a time toward Two
Profiles. After each change, build and test. If fps drops, stop and fix before
proceeding. The changes, in order:

### Step 1: Override SHELL_DIR_USER_DATA

Add the `SHELL_DIR_USER_DATA` path override to point the profile at
`~/.config/termsurf/poc/profile-a`. This changes where Chromium stores profile
data but should not affect rendering.

**Expected: 60fps.**

### Step 2: Add second BrowserContext

Create a second `ShellBrowserContext` with a `SHELL_DIR_USER_DATA` override
pointing to `~/.config/termsurf/poc/profile-b`. Hold it but don't use it.

**Expected: 60fps.** If this drops to 2fps, creating a second BrowserContext
alone (possibly through the storage service crash that Issue 411 observed)
degrades Shell A.

### Step 3: Own the window

This is the critical step. Stop letting Chromium's `Shell` class own the window.
Instead, create the NSWindow ourselves in `InitializeMessageLoopContext` and
place Shell A's WebContents view into it. The `Shell` still creates its own
window (we can't easily prevent that), but we reparent the WebContents view into
our window.

This tests whether reparenting a single WebContents view out of its Shell-owned
window and into our own window breaks the compositor lifecycle. If it does, we
need to fix it before we can add a second profile.

**Expected: 60fps.** If this drops to 2fps, the view reparenting itself is the
problem and we need to fix the compositor lifecycle for reparented views.

### Step 4: Add second WebContents (no view attachment)

Create a second `WebContents` with `browser_context_b_` and navigate it to the
test page. Don't add its view to any window.

**Expected: 60fps.** If this drops to 2fps, the mere existence of a navigating
second WebContents degrades Shell A's rendering.

### Step 5: Attach second view side by side

Add WebContents B's view to our window, side by side with WebContents A.

**Expected: Both at 60fps.** If Shell A drops, the view hierarchy manipulation
is the cause. If Shell A stays at 60fps but Shell B is at 2fps, the visibility
race condition from Issue 411 applies to Shell B specifically and we need to fix
it (e.g., by deferring attachment until `RenderFrameCreated`).

## Process

For each step:

1. Modify `content/one_profile/` to match the step's description.
2. Build with `autoninja -C out/Default one_profile`.
3. Run the app and observe fps.
4. Record the result.
5. If fps dropped, investigate and fix before proceeding.
6. Commit each step (and each fix) separately.

## Experiments

(Experiments will be recorded here as they are conducted.)
