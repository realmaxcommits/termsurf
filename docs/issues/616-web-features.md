# Issue 616: Implement missing web features

## Goal

Systematically identify and implement browser features that are missing from the
current gui/ generation. High-impact features are implemented first. Features
that can't be implemented yet are logged for future work.

## Background

ts1 (WKWebView generation) implemented a comprehensive set of browser features
from v0.3 through v1.0. The current gui/ generation (Chromium via Content API)
has the core streaming pipeline working — live rendering, mouse input, keyboard
input, multi-pane multi-profile — but is missing many user-facing browser
features that ts1 had.

Some ts1 features translate directly to Chromium (downloads, file uploads, JS
dialogs). Others are WKWebView-specific and either aren't needed with Chromium
or require a different approach. This issue catalogs everything and prioritizes
what to build.

### Feature inventory

Features are organized by priority. Priority is based on how often a user would
encounter the missing feature during normal browsing.

#### High priority

These features affect common browsing scenarios. A user will hit these within
minutes of browsing.

| # | Feature                      | ts1 status                       | gui/ status     | Notes                                                                                                               |
| - | ---------------------------- | -------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------- |
| 1 | **target="_blank" handling** | Loads in same view               | Not implemented | Links requesting new windows (OAuth, "Open in new tab") silently fail without this. Very common on modern websites. |
| 2 | **JavaScript dialogs**       | alert/confirm/prompt via NSAlert | Not implemented | Many sites use confirm() for destructive actions, prompt() for input. Sites break without these.                    |
| 3 | **Downloads**                | WKDownloadDelegate + NSSavePanel | Not implemented | Any file download link currently does nothing.                                                                      |
| 4 | **File uploads**             | NSOpenPanel via WKUIDelegate     | Not implemented | `<input type="file">` does nothing without this. Common for profile pictures, attachments, etc.                     |
| 5 | **Page zoom**                | Cmd+=/-/0 via pageZoom           | Not implemented | Users expect standard zoom keybindings.                                                                             |
| 6 | **HTTP Basic Auth**          | NSAlert with username/password   | Not implemented | Password-protected pages show blank or error without this.                                                          |
| 7 | **URL normalization**        | Prepend https://                 | Not implemented | Users type `google.com`, not `https://google.com`. The `web` TUI or Chromium server should handle this.             |

#### Medium priority

These features matter but are encountered less frequently or have workarounds.

| #  | Feature                      | ts1 status                     | gui/ status     | Notes                                                                                                              |
| -- | ---------------------------- | ------------------------------ | --------------- | ------------------------------------------------------------------------------------------------------------------ |
| 8  | **Crash recovery**           | Reload/close dialog            | Not implemented | Chromium renderer crashes are rare but should be handled gracefully.                                               |
| 9  | **Camera/mic permissions**   | Permission prompt              | Not implemented | Only needed for video calls, media recording. Can defer.                                                           |
| 10 | **Console capture**          | JS injection → stdout/stderr   | Not implemented | Useful for developers. The `web` TUI could display console output. Requires Chromium DevTools protocol or similar. |
| 11 | **Web Inspector / DevTools** | Safari Inspector via Cmd+Alt+I | Not implemented | Chromium has DevTools built in, but we need a way to open them (remote debugging port, or in-process).             |

#### Lower priority

These are nice-to-have or may not apply to the Chromium architecture.

| #  | Feature                            | ts1 status                     | gui/ status         | Notes                                                                                      |
| -- | ---------------------------------- | ------------------------------ | ------------------- | ------------------------------------------------------------------------------------------ |
| 12 | **User-Agent spoofing**            | Custom Safari UA string        | Probably not needed | Chromium sends a real browser UA by default. Unlikely to get mobile layouts.               |
| 13 | **Header injection**               | Upgrade-Insecure-Requests      | Probably not needed | Chromium sends this header natively. Was a WKWebView-specific workaround.                  |
| 14 | **Blob download workaround**       | JS interceptor for WebKit bug  | Not needed          | This was a WebKit bug. Chromium handles blob: downloads natively.                          |
| 15 | **Session isolation (incognito)**  | Ephemeral WKWebsiteDataStore   | Not implemented     | Chromium supports incognito via BrowserContext. Low urgency — named profiles already work. |
| 16 | **Bookmarking**                    | Cmd+B, file-based JSON storage | Not implemented     | Useful but not critical for initial release.                                               |
| 17 | **JavaScript API (--js-api)**      | window.termsurf.exit(code)     | Not implemented     | Niche feature for scripting. Defer.                                                        |
| 18 | **Hide/show webviews (ctrl+z/fg)** | isHidden property              | Not implemented     | Terminal backgrounding support. Defer.                                                     |
| 19 | **Multi-webview stacking**         | Stack per pane with indicator  | Not implemented     | Multiple webviews per pane. Current architecture is one-per-pane. Defer.                   |
| 20 | **Dynamic tab titles**             | KVO on WKWebView.title         | Not implemented     | Tab shows page title. Requires Chromium to send title updates via XPC.                     |

#### Already implemented (in gui/ or differently)

| Feature               | Notes                                                                |
| --------------------- | -------------------------------------------------------------------- |
| **Profile isolation** | Multi-profile via separate Chromium Profile Servers (Issues 604–605) |
| **Three-mode focus**  | Browse/Control modes with Esc/Enter switching (Issue 607)            |
| **Focus management**  | Chromium focus/blur via XPC (Issue 606)                              |
| **Control bar**       | `web` TUI draws URL bar, status bar (Issue 504)                      |

### Implementation approach

Each feature is a self-contained experiment. Features that require Chromium-side
changes (new XPC messages, new Content API calls) are harder than features that
can be handled entirely in gui/ Zig code or the `web` TUI.

**Chromium-side changes** are needed for: downloads, file uploads, JS dialogs,
HTTP auth, crash recovery, camera/mic permissions, console capture, DevTools,
dynamic tab titles.

**GUI-side only** changes: target="_blank" (if we load in same tab), page zoom,
URL normalization.

## Experiments

### Experiment 1: Unify test pages and audit existing demos

#### Goal

Replace `html/` and `box-demo/` with a single `test-html/` directory containing
a Bun server that serves all test pages. A main index page links to every demo.
Each existing demo is tested in the current gui/ + Chromium pipeline to identify
which features work and which are broken.

#### Background

The repo currently has test HTML scattered across two top-level directories:

- `html/` — 4 standalone HTML files (dialogs, downloads, mouse, uploads)
- `box-demo/` — Bun server + spinning square demo (FPS, localStorage)

These were created ad-hoc during different experiments. They need a single home
with a proper server so we can systematically test browser features.

`ts4/box-demo/` and `ts5/box-demo/` are identical historical copies and are left
as-is.

#### Steps

##### Step 1: Create `test-html/` with Bun server

Create `test-html/server.ts` — a Bun HTTP server that:

- Serves static files from `test-html/public/`
- Runs on port 9616 (Issue 616)
- Has a root route (`/`) that serves an index page with links to all demos

##### Step 2: Create the index page

Create `test-html/public/index.html` — a main page listing all test demos with
links. Organized by feature category matching the inventory in this issue.

##### Step 3: Move existing test pages

Move the existing test pages into `test-html/public/`:

- `html/test-dialogs.html` → `test-html/public/test-dialogs.html`
- `html/test-download.html` → `test-html/public/test-download.html`
- `html/test-mouse.html` → `test-html/public/test-mouse.html`
- `html/test-upload.html` → `test-html/public/test-upload.html`
- `box-demo/public/index.html` → `test-html/public/test-box-demo.html`

##### Step 4: Add new test pages for untested features

Create minimal test pages for features that don't have test pages yet:

- `test-html/public/test-target-blank.html` — Links with `target="_blank"` and
  `window.open()`
- `test-html/public/test-zoom.html` — Text at various sizes to verify zoom
  behavior
- `test-html/public/test-auth.html` — Link to an HTTP Basic Auth endpoint (can
  use httpbin.org or similar)

##### Step 5: Delete old directories

```bash
git rm -r html/
git rm -r box-demo/
```

##### Step 6: Test each demo

Launch TermSurf, run `web http://localhost:9616`, and systematically test each
demo page. Record pass/fail for each:

| Demo              | Feature tested                      | Expected behavior                           | Result |
| ----------------- | ----------------------------------- | ------------------------------------------- | ------ |
| test-box-demo     | Canvas rendering, FPS, localStorage | Spinning square at 60fps, identity persists |        |
| test-mouse        | Mouse events                        | Click counter increments, events logged     |        |
| test-dialogs      | alert/confirm/prompt                | Native dialogs appear                       |        |
| test-download     | File downloads                      | Save dialog appears                         |        |
| test-upload       | File uploads                        | File picker opens                           |        |
| test-target-blank | target="_blank" links               | Link loads (in same or new view)            |        |
| test-zoom         | Page zoom                           | Cmd+=/-/0 changes text size                 |        |
| test-auth         | HTTP Basic Auth                     | Login dialog appears                        |        |

#### Verification

1. `bun run test-html/server.ts` starts and serves the index page at
   `http://localhost:9616`
2. All demo pages are accessible from the index
3. `html/` and `box-demo/` are deleted from the repo
4. `ts4/box-demo/` and `ts5/box-demo/` are unchanged
5. Each demo has a pass/fail result recorded in the table above
