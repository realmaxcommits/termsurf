# Experiment 4: Enable Contained Generic Downloads

## Description

Experiment 1 classified generic downloads as `Automatable now`. Experiment 2 did
not probe them yet because it focused on browser-service no-crash APIs. After
Experiment 3 fixed the concrete `payments.mojom.PaymentRequest` empty binder,
the next highest-value automatable browser feature is ordinary non-PDF
downloads.

The current Chromium evidence points to a small, contained fix:

- Roamium uses TermSurf's `TsBrowserClient`, which derives from Content Shell.
- Content Shell's `ShellBrowserContext::GetDownloadManagerDelegate()` creates a
  `ShellDownloadManagerDelegate`.
- `ShellDownloadManagerDelegate::DetermineDownloadTarget()` can save directly to
  a directory when `suppress_prompting_` is enabled via
  `SetDownloadBehaviorForTesting(...)`.
- Without that no-prompt mode, the macOS `ChooseDownloadPath()` path reaches
  `NOTIMPLEMENTED()` and returns an empty target path, so generic downloads
  cannot complete deterministically in TermSurf.

TermSurf should not open a native save dialog for this experiment. It should
enable a contained default download directory for Roamium, then prove with a
local fixture that an attachment download and a blob download produce the
expected files and hashes without manual interaction.

This experiment intentionally does not add a download shelf, TUI notification,
download manager UI, history database, or user-facing preference system. Those
can be separate product work if needed. The target here is the browser API
plumbing: when a page triggers a normal download, the Chromium download stack
must have a valid target path and complete the file write.

## Changes

1. Create a new Chromium branch for Issue 799 follow-up work.

   In `chromium/src`, fork from the current Issue 799 branch:

   ```text
   148.0.7778.97-issue-799-exp4
   ```

   Add the branch to `chromium/README.md` with a description such as
   `Enable contained generic downloads`.

2. Add TermSurf download setup in Chromium.

   Add a small helper under `content/libtermsurf_chromium/`, for example:

   ```text
   ts_download_support.h
   ts_download_support.cc
   ```

   The helper should configure each TermSurf `ShellBrowserContext` with a
   contained no-prompt download directory by using the existing
   `ShellDownloadManagerDelegate::SetDownloadBehaviorForTesting(...)` hook.

   Preferred directory source:
   - if a command-line switch such as `--termsurf-download-dir=/path` is
     present, use that exact path;
   - otherwise default to `{browser_context->GetPath()}/Downloads`.

   The helper should create the directory if needed and log the chosen path with
   a stable grep prefix such as:

   ```text
   [termsurf-download] configured path=...
   ```

   Scope:
   - configure the regular browser context;
   - configure the off-the-record browser context only if it has a stable
     context path or can safely use a temp/profile-scoped subdirectory;
   - do not use Chrome's download UI, download bubble, Safe Browsing UI,
     download history, or `chrome/browser/download` stack;
   - do not add `termsurf.proto`, Roamium FFI, Wezboard, or webtui messages in
     this experiment.

3. Wire the helper from `TsBrowserMainParts`.

   In `TsBrowserMainParts::InitializeBrowserContexts()`, after the
   `ShellBrowserContext` instances exist and before ordinary page loads can
   trigger downloads, call the TermSurf download helper for the context(s).

   Keep the change local to `content/libtermsurf_chromium` plus the narrow
   Content Shell delegate API usage. Do not patch the generic Content Shell
   download behavior globally unless there is no clean TermSurf-only call path.

4. Extend the Issue 799 browser API harness with download probes.

   Add two deterministic local fixture endpoints:
   - `/download/attachment.txt` returns a small fixed byte string with
     `Content-Disposition: attachment; filename="termsurf-download.txt"`;
   - `/download/blob.html` creates a Blob URL with a distinct fixed byte string
     and clicks an `<a download="termsurf-blob-download.txt">`.

   Add harness support for a per-run download directory:

   ```text
   {run_dir}/downloads
   ```

   Pass that directory to Roamium with:

   ```text
   --termsurf-download-dir={run_dir}/downloads
   ```

   The download probes should wait for the expected file(s) to appear, wait
   until any `.crdownload` intermediate file is gone, then verify file contents
   by hash or exact bytes.

   The file verification must drive probe classification. A page-side "download
   triggered" report is not sufficient. The attachment and blob probes should
   classify as something explicit like `download_completed` only after the
   harness has verified the final file path, size, and exact bytes. If the page
   reports success but the file is missing, still has a lingering `.crdownload`,
   or has the wrong bytes, classify the probe as a download failure rather than
   `exercised`.

   Required new artifacts:
   - include `download_dir` in `run.json`;
   - include per-download file path, size, and hash in `probe-results.json`;
   - include the `[termsurf-download] configured path=...` log evidence in the
     result when available.

5. Keep PDF save behavior out of scope.

   PDF save already has its own PDF-specific path and issue history. This
   experiment must test ordinary generic downloads, not PDF toolbar save.

6. Build and verify.

   Build Chromium with:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the harness changes are Rust-free, no `cargo fmt` is needed. If any Rust
   files are touched, run `cargo fmt` and accept its output.

7. Run the automated harness.

   First run only the new download probes:

   ```bash
   scripts/test-issue-799-browser-api-audit.py --probe download-attachment --seconds 8
   scripts/test-issue-799-browser-api-audit.py --probe download-blob --seconds 8
   ```

   Then run the full suite:

   ```bash
   scripts/test-issue-799-browser-api-audit.py --seconds 8
   ```

   Record the run directories and the relevant `probe-results.json`,
   `coverage-map.md`, `reference-coverage-map.md`, `binder-errors.tsv`, and
   download file evidence in this experiment.

8. Archive Chromium patches only after a successful implementation.

   If the experiment passes, commit the Chromium branch, regenerate the Issue
   799 Chromium patch archive, update `chromium/README.md`, and record the main
   repo patch/docs changes.

   If the experiment is partial or fails, record the exact blocker and do not
   silently expand into Chrome's full download product stack.

9. Run Codex review before completion is accepted.

   After implementation and verification, run `codex-review` against:
   - this experiment file;
   - the Chromium diff;
   - the main repo diff;
   - the narrow download harness runs;
   - the full browser API harness run.

   Ask Codex to verify that the implementation is TermSurf-scoped, no-prompt,
   deterministic, does not import Chrome download UI/product code, and that the
   file/hash evidence proves generic downloads complete. Fix real findings
   before marking the experiment `Pass`, `Partial`, or `Fail`.

## Verification

This experiment passes if:

- Chromium builds successfully with
  `autoninja -C out/Default libtermsurf_chromium`;
- Roamium accepts `--termsurf-download-dir={run_dir}/downloads`;
- the Chromium log contains `[termsurf-download] configured path=...` for the
  run directory;
- the attachment probe writes `termsurf-download.txt` into the run download
  directory with the exact expected bytes;
- the blob probe writes `termsurf-blob-download.txt` into the run download
  directory with the exact expected bytes;
- no native save dialog appears;
- no manual interaction, real Wezboard window, screenshot, Accessibility
  permission, or Screen Recording permission is required;
- the full Issue 799 harness still runs without regressing Experiment 3's
  Payment Request behavior;
- no Chrome download UI, download bubble, Safe Browsing UI, or download history
  product stack is imported;
- Codex reviews the completed experiment and has no blocking findings.

This experiment is partial if:

- the attachment download works but blob downloads need a follow-up trigger or
  user-activation strategy;
- downloads complete but the harness cannot yet produce reliable hash/file
  evidence;
- the default profile `Downloads` path works but the explicit
  `--termsurf-download-dir` switch does not;
- a second lifecycle issue appears after file write, such as shutdown cleanup or
  lingering `.crdownload` files.

This experiment fails if:

- it opens native save UI;
- it depends on manual clicking;
- it changes `termsurf.proto`, Roamium FFI, Wezboard, or webtui for generic
  downloads;
- it imports Chrome's full download product UI stack;
- it breaks PDF loading/saving, Payment Request default-deny behavior, or the
  existing browser API harness.

## Expected Outcome

The expected outcome is not a complete download manager UI. The expected outcome
is that ordinary page-triggered downloads have a deterministic target path and
complete their file writes in Roamium.

If this passes, the next Issue 799 experiment should move to the next
automatable queue item from Experiment 1: likely page zoom or console capture if
we want a narrow Chromium-only feature, or JavaScript dialogs / HTTP Basic Auth
if we are ready to add a protocol-mediated prompt/reply design.
