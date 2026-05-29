# Experiment 4: Load PDF Viewer Resource Bytes

## Description

Experiment 3 proved the PDF component extension can be registered and that the
generated PDF resource map resolves paths to resource ids. It did **not** prove
that TermSurf can read the viewer resource bytes. That is the next required
layer before any viewer frame, MimeHandlerView, or PDF stream handoff can work.

This experiment loads Chromium's generated `pdf_resources.pak` into
`ui::ResourceBundle` from TermSurf-owned startup code, then proves the PDF
viewer HTML bytes are available from the resource bundle. It does not wire
`chrome-extension://` serving, `PdfNavigationThrottle`,
`PdfViewerStreamManager`, guest-view, MimeHandlerView, or PDF rendering.

This is intentionally a resource-pack experiment, not a PDF-navigation
experiment. If resource bytes cannot be loaded reliably in the debug runtime,
later navigation work would fail for the wrong reason.

This experiment must receive Claude design review before implementation. After
implementation and result recording, Claude must review the completed output
before any next experiment is designed.

## Changes

1. Create the Chromium implementation branch.

   Start from the accepted Experiment 3 branch:

   ```bash
   git -C chromium/src checkout 148.0.7778.97-issue-792-exp3
   git -C chromium/src checkout -b 148.0.7778.97-issue-792-exp4
   ```

   Add the branch to `chromium/README.md` only after the branch builds and the
   result is accepted.

2. Add a narrow PDF resource-pack loader.

   Add TermSurf-owned code under:

   ```text
   chromium/src/content/libtermsurf_chromium/extensions/
   ```

   Suggested name:

   ```text
   ts_pdf_resource_bundle.{h,cc}
   ```

   The helper should:
   - be called once per process from a new `TsMainDelegate::PreSandboxStartup()`
     override, after calling `ShellMainDelegate::PreSandboxStartup()`;
   - locate `pdf_resources.pak` for the current repo-built Chromium runtime;
   - call `ui::ResourceBundle::GetSharedInstance().AddDataPackFromPath(...)`;
   - log a clear failure if the pak cannot be found or loaded.

   `PreSandboxStartup()` is the correct hook because content_shell initializes
   its own resource bundle there, and the hook runs for all process types. That
   all-process guarantee matters for later URL-loader/viewer work: if only the
   browser process loads the pak, future renderer-side resource reads can fail
   for the wrong reason.

   For the debug repo runtime, the expected path is:

   ```text
   chromium/src/out/Default/gen/chrome/pdf_resources.pak
   ```

   Resolve this with:

   ```text
   base::PathService::Get(base::DIR_ASSETS, ...)
   ```

   then append:

   ```text
   gen/chrome/pdf_resources.pak
   ```

   For the debug content_shell build, `DIR_ASSETS` resolves to `out/Default/`.
   Do not walk `argv[0]`, parse the current working directory, or add a
   hardcoded absolute `/Users/...` path. If the debug path cannot be discovered
   cleanly, record Partial rather than adding a local-machine-only path.

   Packaging for release builds is out of scope. The result must note that
   release packaging will need to include `pdf_resources.pak` once inline PDF is
   complete. Track release packaging as a follow-up once inline PDF is
   end-to-end; both the `.app` bundle `Contents/Resources/` layout and the
   Homebrew `/opt/homebrew/opt/termsurf-roamium/` layout will need the pak.

3. Prove viewer bytes are available.

   After loading the pak, run a startup self-test:
   - include `chrome/grit/pdf_resources.h`, which defines `IDR_PDF_INDEX_HTML`;
   - read `IDR_PDF_INDEX_HTML` with
     `ui::ResourceBundle::LoadDataResourceString()`;
   - confirm the returned string is non-empty;
   - log the byte count and a stable lightweight signature: case-insensitive
     substring match within the first 256 bytes for `<!doctype html` or `<html`.

   Use `LoadDataResourceString()` for this check, not `LoadDataResourceBytes()`,
   because the generated GRIT resource may be stored compressed and the string
   API performs the needed decoding for the HTML signature check.

   Gate the self-test log to the browser process (`process_type.empty()`), or
   otherwise ensure it emits at most once per process. Avoid noisy repeated logs
   from every renderer.

   Do not add a test that depends on visual PDF rendering. Rendering remains out
   of scope.

4. Keep Experiment 3 behavior intact.

   The PDF component extension must still register with:

   ```text
   id=mhjfbmdgcfjbbpaeojofohoefgiehjai
   enabled=1
   ```

   The resource-manager path-to-id self-test must still pass.

5. Update GN narrowly.

   Expected deps:

   ```text
   //chrome/browser/resources/pdf:resources
   //ui/base
   ```

   Forbidden deps in this experiment:

   ```text
   //chrome/browser:resources
   //chrome/browser/pdf
   //chrome/browser/pdf:pdf
   //chrome/browser/plugins:impl
   //chrome/browser/extensions:extensions
   //chrome/browser/ui
   ```

   If loading the pak requires a forbidden dep, stop and record Partial instead
   of importing it.

6. Add structured diagnostics.

   Use Chromium `LOG(INFO)` lines with this exact prefix:

   ```text
   [issue-792-exp4]
   ```

   Required lines:

   ```text
   [issue-792-exp4] pdf-resource-pak path=<path> found=<0|1> loaded=<0|1>
   [issue-792-exp4] pdf-resource-bytes id=<id> bytes=<n> html_signature=<0|1>
   ```

   Low-volume diagnostics may remain.

7. Build and archive only after verification.

   Build:

   ```bash
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the branch builds and verification passes or produces a useful Partial, do
   the full bookkeeping after Claude after-review accepts the result:
   - commit the Chromium branch;
   - regenerate `chromium/patches/issue-792/`;
   - add the new branch row to `chromium/README.md`;
   - update Experiment 4's line in `issues/0792-pdf-support/README.md` from
     `Designed` to the final status.

## Verification

1. Confirm starting state.

   ```bash
   git status --short
   git -C chromium/src status --short
   git -C chromium/src branch --show-current
   ```

   Chromium should start clean on `148.0.7778.97-issue-792-exp3`.

2. Build the branch.

   ```bash
   autoninja -C chromium/src/out/Default libtermsurf_chromium
   ```

3. Run the automated debug-path HTML smoke.

   Reuse the screenshot harness against:

   ```text
   http://localhost:9616/index.html
   ```

   Pass requires:
   - the page visibly renders or, if screenshot capture flakes, lifecycle logs
     reach `TitleChanged` and `LoadingState`;
   - no extension IPC crash;
   - Experiment 3 registration logs still appear;
   - Experiment 4 resource-pak logs show `found=1`, `loaded=1`,
     `bytes=<nonzero>`, and `html_signature=1`, where `html_signature=1` means
     the decoded string's first 256 bytes contain `<!doctype html` or `<html`
     case-insensitively.

4. Run the PDF unchanged smoke.

   Load:

   ```text
   http://localhost:9616/bitcoin.pdf
   ```

   The PDF is still expected to be blank/download/default-path because this
   experiment does not install PDF navigation or stream handling. A browser
   crash, renderer IPC crash, or hang is a failure.

5. Run Claude review after recording the result.

   Provide Claude with the experiment file, Chromium diff, build output summary,
   runtime logs, screenshot artifact paths, and the recorded result. Fix all
   real findings before proceeding.

## Pass Criteria

- Chromium branch `148.0.7778.97-issue-792-exp4` builds `libtermsurf_chromium`.
- The PDF resource pak is found and loaded without forbidden Chrome deps.
- `IDR_PDF_INDEX_HTML` returns non-empty bytes from `ui::ResourceBundle`.
- The PDF component extension still registers as in Experiment 3.
- Normal HTML browsing still works through the full debug TermSurf path.
- Loading `bitcoin.pdf` does not crash; rendering is not required.
- Claude reviews the completed result and agrees it is good enough to proceed.

## Partial Criteria

Partial if:

- the pak can be found but `ui::ResourceBundle` cannot load it in the
  content_shell runtime;
- the bytes can be loaded only through a path that is too debug-specific for
  future packaging;
- the branch builds and HTML browsing works, but resource bytes remain empty;
- a forbidden dep is proven necessary and the result records the exact blocker.

## Failure Criteria

- The experiment imports Chrome's broad browser resource target or browser UI
  stack to make the resource bytes appear.
- The experiment changes PDF navigation, stream handling, guest-view,
  MimeHandlerView, or renderer process routing.
- The experiment tries to render PDFs visually.
- The experiment regresses normal HTML browsing or reintroduces the extension
  renderer IPC crash.
- The experiment proceeds without Claude design review or ignores real Claude
  findings.

## Result

**Result:** Pass

Experiment 4 built and verified the PDF viewer resource-pack layer.

Implementation branch:

```text
148.0.7778.97-issue-792-exp4
```

Build command:

```bash
export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
git -C chromium/src cl format --upstream=148.0.7778.97-issue-792-exp3 --full
autoninja -C chromium/src/out/Default libtermsurf_chromium
```

Build result:

```text
Build Succeeded: 0 steps
```

HTML smoke artifact:

```text
logs/issue-792-exp4-html-20260529-092117/
```

Required diagnostics appeared:

```text
[issue-792-exp4] pdf-resource-pak path=/Users/ryan/dev/termsurf/chromium/src/out/Default/gen/chrome/pdf_resources.pak found=1 loaded=1
[issue-792-exp4] pdf-resource-bytes id=21596 bytes=529 html_signature=1
```

Experiment 3 behavior remained intact in the same run:

```text
[issue-792-exp3] pdf-component-extension-registered context=<ptr> enabled=1 inserted=1
```

The normal HTML page reached TermSurf lifecycle messages:

```text
LoadingState
TitleChanged
LoadingState
```

PDF unchanged-smoke artifact:

```text
logs/issue-792-exp4-pdf-20260529-092316/
```

The PDF run produced the same required resource logs:

```text
[issue-792-exp4] pdf-resource-pak path=/Users/ryan/dev/termsurf/chromium/src/out/Default/gen/chrome/pdf_resources.pak found=1 loaded=1
[issue-792-exp4] pdf-resource-bytes id=21596 bytes=529 html_signature=1
```

As expected, `bitcoin.pdf` still takes the pre-existing content_shell download
path:

```text
ShellDownloadManagerDelegate::ChooseDownloadPath(...)
```

That is not a failure for this experiment because Experiment 4 deliberately did
not install PDF navigation, stream handling, guest-view, MimeHandlerView, or PDF
renderer routing.

Both smoke runs still show the known teardown `SEGV_ACCERR` after artifacts were
captured. This is the same cleanup crash recorded in earlier PDF experiments and
is not caused by resource-pack loading.

Release-packaging follow-up: this experiment loads `pdf_resources.pak` from the
debug repo layout at `out/Default/gen/chrome/`. Once inline PDF is end-to-end,
the release packaging path must copy `pdf_resources.pak` into both the `.app`
bundle `Contents/Resources/` layout and the Homebrew
`/opt/homebrew/opt/termsurf-roamium/` layout, and `DIR_ASSETS` must resolve to
the matching Resources path in release builds.

Bookkeeping status: Chromium branch commit, patch archive refresh,
`chromium/README.md` branch row, and the issue README status flip were deferred
until Claude after-review accepted this result.

## Conclusion

TermSurf can load Chromium's generated `pdf_resources.pak` through
`base::DIR_ASSETS`, add it to `ui::ResourceBundle`, and decode
`IDR_PDF_INDEX_HTML` without importing Chrome's broad browser resource stack.
The PDF component extension registration from Experiment 3 still works, and
normal HTML browsing still reaches the expected lifecycle messages.

The next experiment can move from "resource bytes exist" to "extension resource
URLs can serve those bytes." That should stay narrow: implement the
`chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/index.html` resource
loading path for the registered PDF component extension, then prove the viewer
HTML is returned through Chromium's URL-loader path before touching PDF
navigation or stream handoff.
