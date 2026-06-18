# Experiment 7: Prove Copy-Current-URL

## Description

Issue 816 still needs Ghostboard-specific proof for copy-current-URL behavior.
The historical Issue 334 requirement was: Cmd+C in browser control mode copies
the current webview URL to the system clipboard and briefly shows user feedback,
while Browse mode still lets browser content handle Cmd+C.

Current webtui has URL editor clipboard support through `UrlClipboard`, but the
audit did not prove a dedicated copy-current-URL workflow under Ghostboard. This
experiment will first prove the current behavior, then implement the smallest
owner-specific fix if the behavior is missing.

## Changes

Planned investigation:

- Inspect the current copy/clipboard paths in:
  - `webtui/src/main.rs`;
  - `docs/keybindings.md`;
  - Ghostboard AppKit/menu key handling for Cmd+C;
  - Issue 334 historical copy-URL behavior;
  - Issue 810 clipboard/copy-current-URL findings.
- Determine whether Cmd+C in Control mode reaches webtui as a terminal key
  event, is handled by Ghostboard/Ghostty menu copy behavior, or is intercepted
  before either path can copy the URL.
- Identify whether the right owner is webtui command/key handling, Ghostboard
  macOS menu/key assignment handling, or both.

Planned harness changes:

- Add a `copy-current-url-smoke` scenario to
  `scripts/ghostboard-geometry-matrix.sh`.
- Serve or navigate to a local fixture URL with a unique query marker.
- Clear the macOS clipboard to a sentinel value before each copy attempt.
- Drive the app into webtui Control mode and trigger the copy-current-URL
  workflow.
- Read the macOS clipboard with `pbpaste` and assert it equals the current URL.
- Capture webtui state trace, app log, Roamium trace, and clipboard before/after
  evidence.

Planned behavior targets:

- Primary target: Cmd+C in webtui Control mode copies the current URL to the
  system clipboard.
- Secondary target: the TUI records a test-only copy-current-URL trace event and
  exposes brief feedback evidence so the harness can distinguish a deliberate
  URL copy from an accidental terminal selection copy.
- Browse mode must not be hijacked by the copy-current-URL behavior. If tested
  in this experiment, Cmd+C in Browse mode should remain browser-owned.

Planned fix policy:

- If Cmd+C reaches webtui in Control mode, implement the copy-current-URL
  behavior in webtui and update keybinding documentation.
- If Cmd+C is intercepted by Ghostboard/Ghostty menu handling before webtui,
  implement the narrow Ghostboard-owned menu/key-assignment path that copies the
  active webtui URL only when the browser pane is in Control mode. Do not change
  ordinary terminal selection copy.
- If a reliable Cmd+C path cannot be implemented without broader app keybinding
  work, add a webtui command such as `:copy-url` only as a diagnostic fallback
  and record Cmd+C as unresolved. Do not count the fallback command as full pass
  for the historical requirement.

Planned issue-doc changes:

- Record the current behavior, owner, implemented path, clipboard evidence, and
  whether Browse mode remains browser-owned.
- If a keybinding is added or changed, update `docs/keybindings.md`.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/07-prove-copy-current-url.md`.
2. If Rust files change, `cargo fmt -- <changed-rust-files>`.
3. If Zig files change, `zig fmt <changed-zig-files>`.
4. If keybindings change, update `docs/keybindings.md` and run prettier on it.

Static/build checks:

1. `prettier --check --prose-wrap always --print-width 80 issues/0816-ghostboard-browser-state-interruptions/README.md issues/0816-ghostboard-browser-state-interruptions/07-prove-copy-current-url.md`.
2. `bash -n scripts/ghostboard-geometry-matrix.sh`.
3. `cargo check -p webtui` if webtui changes.
4. `cargo build -p webtui` if webtui changes.
5. If Ghostboard Zig or non-`macos/` Ghostboard files change, run
   `cd ghostboard && zig build -Demit-macos-app=false`.
6. If Ghostboard app files change or a Ghostboard rebuild is needed, run
   `cd ghostboard && macos/build.nu --configuration Debug --action build`.
7. `shellcheck scripts/ghostboard-geometry-matrix.sh` if available.
8. `git diff --check`.

Design gate:

- This experiment file is plan-only until a fresh-context design review approves
  it.
- Record design review findings and fixes in this file.
- Commit the approved experiment plan before implementation begins.

Completion gate:

- After implementation and verification, record `## Result` and `## Conclusion`
  in this file.
- Update the Issue 816 README experiment status from `Designed` to the final
  result.
- Request a fresh-context completion review, fix all real findings, and record
  the final completion-review verdict in this file.
- Commit the reviewed experiment result separately before designing or
  implementing the next experiment.

Runtime checks:

1. `scripts/ghostboard-geometry-matrix.sh copy-current-url-smoke`.
2. Confirm the local fixture URL with unique query marker is loaded.
3. Confirm the clipboard is set to a sentinel value before copying.
4. Confirm Control-mode Cmd+C triggers copy-current-URL behavior.
5. Confirm the clipboard equals the current URL after copying, including the
   query marker.
6. Confirm trace/feedback evidence identifies the URL-copy action.
7. Confirm ordinary Browse-mode browser copy behavior is not broken, or record
   Browse-mode coverage as a required follow-up if it is too broad for this
   experiment.

Pass criteria:

- Cmd+C in Control mode copies the current URL under debug Ghostboard.
- Clipboard evidence exactly matches the current URL and cannot pass from a
  stale sentinel or terminal selection.
- The harness includes durable assertions for key/menu path, URL-copy trace or
  feedback, and clipboard contents.
- Any keybinding or owner change is documented.

Partial criteria:

- A fallback command can copy the URL, but Cmd+C remains intercepted or
  unresolved.
- The owner is proven, but the correct fix requires broader Ghostboard menu or
  keybinding work.
- Control-mode copy works, but Browse-mode non-interference remains unproven.

Fail criteria:

- The harness cannot distinguish current URL copy from terminal selection copy.
- The clipboard cannot be read or written reliably under the VM.
- The implementation counts a diagnostic fallback command as full historical
  Cmd+C parity.

## Design Review

Fresh-context adversarial design review by Codex subagent `Plato`:

- **Verdict:** Approved.
- **Findings:** None.
- **Reviewer notes:** The reviewer confirmed the README links Experiment 7 as
  `Designed`, required sections are present, scope matches the Issue 816
  copy-current-URL gap, Cmd+C menu interception is explicitly handled,
  verification includes clipboard sentinel, exact URL, trace/feedback, and
  fallback-not-full-pass criteria, and hygiene plus design/completion gates and
  keybinding documentation requirements are present.
