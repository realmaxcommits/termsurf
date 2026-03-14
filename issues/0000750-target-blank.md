# Issue 750: target="\_blank" links don't open

## Goal

Links with `target="_blank"` (and `window.open()` calls) navigate the current
tab instead of silently failing. Multi-tab support is deferred — for now, all
new-window requests open in the same tab.

## Background

### The problem

Clicking a `target="_blank"` link or triggering `window.open()` in a browser
overlay does nothing. The link silently fails because Content Shell's default
behavior creates a new `Shell` window, which has no connection to TermSurf's
overlay system — the new window is invisible and orphaned.

### Prior art

Issue 639 solved this exact problem in an earlier Chromium fork branch
(`146.0.7650.0-issue-639`). The solution used Electron's pattern:

1. Override `IsWebContentsCreationOverridden()` to return `true`, suppressing
   new `WebContents` creation. Post a deferred `PostTask` to navigate the source
   tab to the target URL after the `CreateNewWindow` call chain unwinds.
2. Override `CreateCustomWebContents()` to return `nullptr`.
3. Modify `OpenURLFromTab()` to route `NEW_POPUP`, `NEW_WINDOW`,
   `NEW_BACKGROUND_TAB`, and `NEW_FOREGROUND_TAB` dispositions to the source
   tab.

The patch is preserved at
`chromium/patches/issue-639/0042-Suppress-new-window-navigate-same-tab.patch`.

### Why the fix was lost

Issue 708 refactored the Chromium fork from `content/chromium_profile_server/`
to `content/libtermsurf_chromium/` and created a new branch
(`146.0.7650.0-issue-708`). The Issue 639 commits were not carried forward to
the new branch. The current branch (`146.0.7650.0-issue-708`) has the vanilla
Content Shell `OpenURLFromTab()` behavior — it creates new `Shell` windows for
new-tab/popup requests.

### What needs to happen

Re-apply the Issue 639 fix to the current `shell.h`/`shell.cc` on a new Chromium
branch. The code is nearly identical — the only difference is the file paths
changed from `content/chromium_profile_server/browser/shell.*` to
`content/shell/browser/shell.*`.
