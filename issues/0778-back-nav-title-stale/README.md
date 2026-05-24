+++
status = "open"
opened = "2026-04-12"
+++

# Issue 778: Back navigation leaves stale title in webtui

## Goal

When the user navigates back in the webtui, the displayed page title must update
to match the page being restored, not remain stuck on the title of the page they
navigated away from.

## Background

In the `web` TUI, navigating from page A to page B via a link works correctly —
both the URL and the title update to reflect page B. However, when the user then
presses "back" to return to page A:

- The URL correctly updates back to page A.
- The title does **not** update — it remains showing page B's title.

This means the title shown in the TUI is out of sync with the actual page
displayed in the browser pane after a back navigation.

## Analysis

The webtui listens for protocol messages from the browser engine to keep its
chrome (URL bar, title) in sync with the tab state. The URL update path works
for back navigation, but the title update path does not.

Possible causes:

1. **Missing title update on history navigation** — The browser engine may emit
   a `TitleChanged` message on initial page load but not when a history entry is
   restored from the back/forward cache.
2. **Cached title not re-sent** — When restoring from bfcache, Chromium may not
   fire the title-changed notification because the title hasn't technically
   "changed" from the engine's perspective, leaving the TUI with the previously
   cached value.
3. **webtui ignores title updates tied to history events** — The TUI may only
   update its title on explicit navigation-complete events, missing the separate
   title notification.

The fix likely involves ensuring the browser engine emits a title update
whenever a history navigation commits, or having the webtui proactively request
the current title after a back/forward navigation completes.

## Experiments

### Experiment 1: Re-Emit Title On Navigation Commit

#### Description

The most likely cause is that Chromium emits `UrlChanged` from
`TsTabObserver::DidFinishNavigation()` whenever a primary main-frame navigation
commits, but only emits `TitleChanged` from `TsTabObserver::TitleWasSet()`.
During back/forward navigation, Chromium may restore an already-known history
entry title without firing `TitleWasSet()` again. webtui then receives the
restored URL but keeps displaying the previous page's title.

Fix this at the Chromium observer layer. Whenever a primary main-frame
navigation commits and TermSurf sends `UrlChanged`, also read the current
committed navigation entry title and send `TitleChanged`. Keep the existing
`TitleWasSet()` path for JavaScript-driven title changes after navigation.

This should be a Chromium-only experiment. Do not add new protocol messages and
do not paper over the stale state in webtui by merely clearing the title on
`UrlChanged`.

#### Changes

1. Create a new Chromium branch for this issue from the most relevant recent
   TermSurf Chromium branch: `148.0.7778.97-issue-778`.
2. Add the branch to the Branches table in `chromium/README.md`.
3. In `chromium/src/content/libtermsurf_chromium/ts_tab_observer.cc`, update
   `TsTabObserver::DidFinishNavigation()`:
   - keep the existing committed primary-main-frame guard;
   - keep the existing `TsNotifyUrlChanged(...)` call;
   - after notifying the URL, get the current committed navigation entry from
     `web_contents()->GetController()`;
   - if an entry exists, convert `entry->GetTitleForDisplay()` to UTF-8 and call
     `TsNotifyTitleChanged(...)`.
4. Keep `TsTabObserver::TitleWasSet()` intact. It remains responsible for title
   changes that happen after navigation commit.
5. Add any include needed by Chromium 148 for `NavigationController` access, but
   avoid unrelated include churn.
6. Regenerate the issue 778 Chromium patch archive after the branch commit.

#### Non-Negotiable Invariants

- URL updates must continue to work on normal navigation and back/forward
  navigation.
- JavaScript title changes after page load must still update webtui through the
  existing `TitleWasSet()` path.
- webtui should not synthesize or guess page titles.
- No new TermSurf protocol messages are introduced.
- No popup, DevTools, split-border, or drag-suppression behavior should change.

#### Verification

1. Build Chromium/Roamium and the debug TermSurf components using the normal
   debug testing flow.
2. Run debug Wezboard and debug webtui with the newly built Roamium binary, not
   the installed release Roamium.
3. Open a page A with a distinctive title.
4. Navigate by clicking a link to page B with a different distinctive title.
   Verify webtui shows page B's URL and title.
5. Press back. Verify webtui shows page A's URL and page A's title on the first
   restored view. The title must not remain page B's title.
6. Press forward. Verify webtui shows page B's URL and page B's title again.
7. Test a page that changes `document.title` after load. Verify the later title
   update still reaches webtui.
8. Smoke-test ordinary link clicks, reload, and direct URL entry.

#### Pass Criteria

Back and forward navigation always leave webtui's displayed title matching the
currently displayed page, including when Chromium restores a cached history
entry and does not fire a separate `TitleWasSet()` notification.

#### Partial Criteria

The stale-title case is fixed for ordinary back/forward navigation, but one
secondary title path remains wrong, such as JavaScript title updates after load
or an edge case involving pages with no title. Record the failing path and
design a follow-up experiment.

#### Failure Criteria

- The title remains stale after back navigation.
- The fix only hides the stale title by clearing it in webtui instead of sending
  the correct title from Chromium.
- JavaScript-driven title changes stop working.
- The patch introduces protocol changes or touches unrelated subsystems.
