# Experiment 1: Inventory and Classify Missing Browser APIs

## Description

Build the authoritative inventory of TermSurf's missing browser APIs and
web-feature surfaces, then classify each item by whether it can be implemented
and verified automatically.

This is a research and triage experiment only. It must not change runtime code,
Chromium branches, protocol messages, or browser behavior. The output is a
decision table that defines the implementation scope for the rest of Issue 799.

The key question is not "does TermSurf eventually need this?" The key question
is "can this issue solve and verify this item without relying on ongoing manual
testing?" Items that fail that automation bar must be deferred with a clear
reason, even if they are important product gaps.

## Changes

1. Audit source documents.

   Read and extract candidate APIs/features from:
   - `TODO.md`, especially the Web features and Future issues sections;
   - `issues/0616-web-features/README.md`;
   - `issues/0655-substack-blank/README.md`;
   - `issues/0750-target-blank/README.md`;
   - `issues/0780-link-drag-freeze/README.md`;
   - `issues/0792-pdf-support/README.md`;
   - `issues/0794-pdf-viewer-interactions/README.md`;
   - `issues/0795-pdf-native-print/README.md`;
   - `issues/0796-pdf-implementation-audit/README.md`;
   - `issues/0797-pdf-core-workflow-coverage/README.md`;
   - `issues/0798-pdf-advanced-features/README.md`;
   - any other issue found by grep for missing browser APIs, Mojo binders,
     browser delegates, permissions, downloads, dialogs, file pickers, auth,
     crash recovery, console capture, or drag/drop.

2. Audit current implementation evidence.

   Use source searches to determine whether each candidate is already solved,
   partially solved, missing, or unknown. At minimum, inspect:
   - `termsurf.proto` and generated protocol usage for relevant message types;
   - `roamium/src/dispatch.rs`;
   - `roamium/src/ffi.rs`;
   - `webtui/src/`;
   - `wezboard/wezboard-gui/src/termsurf/`;
   - `wezboard/wezboard-gui/src/termwindow/`;
   - `chromium/src/content/libtermsurf_chromium/`;
   - `chromium/README.md` and the latest relevant Chromium patch archives.

   The experiment should use concrete greps, not memory. Suggested searches:

   ```bash
   rg -n "alert|confirm|prompt|download|upload|file picker|FileChooser|zoom|Basic Auth|Auth|permission|camera|microphone|console|crash|BadgeService|mojo|binder|drag|drop|clipboard|DevTools|target|blank" \
     TODO.md issues docs roamium webtui wezboard/wezboard-gui chromium/README.md

   rg -n "RegisterBrowserInterfaceBinders|RegisterAssociatedInterfaceBinders|AddInterface|mojom|BadgeService|Permission|Download|FileChooser|JavaScriptDialog|AuthRequired|RenderProcessGone|RenderFrameDeleted" \
     chromium/src/content/libtermsurf_chromium
   ```

   If Chromium source searches are too broad or slow, narrow them to
   `content/libtermsurf_chromium`, `content/shell`, `chrome/browser`,
   `components/`, and the branch patch archives relevant to TermSurf.

3. Compare TermSurf against reference embedders.

   The inventory must not only ask "what did TermSurf already mention?" It must
   also compare TermSurf's embedder surface against Chromium embedders that wire
   browser APIs more completely.

   Inspect reference code where practical:
   - `chromium/src/content/public/browser/content_browser_client.h`;
   - `chromium/src/content/public/renderer/content_renderer_client.h`;
   - `chromium/src/content/shell/`;
   - `chromium/src/headless/`;
   - relevant `chromium/src/chrome/browser/` binder, delegate, permission,
     download, file chooser, dialog, auth, and crash-handling code;
   - local Electron source, if present, especially `shell/browser/` and
     `shell/renderer/`.

   For each candidate where this comparison is meaningful, record whether
   Chrome, content shell, headless, or Electron provides a binder/delegate/API
   implementation that TermSurf lacks. The result can be brief, but it must be
   specific enough to justify the classification. For example:

   | Candidate     | Reference evidence              | TermSurf evidence         |
   | ------------- | ------------------------------- | ------------------------- |
   | BadgeService  | headless has `StubBadgeService` | fixed in Issue 655        |
   | File uploads  | Chrome wires file chooser       | no TermSurf path observed |
   | Notifications | Chrome has permission stack     | no TermSurf binder audit  |

4. Include broad Web Platform browser-service candidates.

   Issue 616 lists the known product gaps, but Issue 655 showed that renderer
   crashes can come from any missing browser-process API surface exposed through
   Mojo or delegates. The inventory must therefore include at least these broad
   families, even if many are deferred:
   - Notifications and Push API browser services;
   - Geolocation;
   - WebAuthn and Credential Management;
   - Payment Request;
   - Web Share;
   - File System Access and related storage/quota prompts;
   - Web Bluetooth, USB, HID, Serial, and MIDI;
   - screen capture and media-capture permissions;
   - Permissions API plumbing;
   - service-worker-adjacent browser services that require browser-process
     delegates or binders.

   If a family is not relevant to the current Chromium build, disabled by
   feature flags, unsupported by content shell, or intentionally outside
   TermSurf's product scope, record that instead of omitting it.

5. Create the inventory table inside this experiment file.

   Append a `## Result` section containing a table with one row per candidate.
   The table must include these columns:

   | Candidate | Source | Current status | Likely layer | Automation class | Proposed verification | In Issue 799 scope? | Deferred/follow-up |
   | --------- | ------ | -------------- | ------------ | ---------------- | --------------------- | ------------------- | ------------------ |

   Classification values:
   - `Automatable now`;
   - `Automatable after setup`;
   - `Deferred: not automatable enough`;
   - `Already solved`;
   - `Out of scope product decision`.

6. Define concrete verification plans for in-scope items.

   For every `Automatable now` or `Automatable after setup` item, record a
   proposed automated verification method. Acceptable methods include:
   - local deterministic HTML fixtures;
   - fake-GUI protocol harnesses;
   - DevTools Protocol probes;
   - screenshot classification;
   - log assertions for Chromium/Roamium/Wezboard;
   - contained download directories;
   - synthetic protocol messages;
   - controlled local servers;
   - test-only env vars or command-line switches, as long as production behavior
     remains unchanged.

   Do not mark an item in-scope if the proposed verification only says "manual
   test" or "visually inspect."

7. Treat native UI and platform permissions carefully.

   Items involving native dialogs, file pickers, permission prompts, print
   dialogs, camera/mic permissions, system Accessibility permissions, Screen
   Recording permissions, or OS-level drag/drop must be classified honestly.
   They can remain in scope only if the experiment identifies a contained or
   deterministic automation strategy. Otherwise they must be deferred.

8. Decide the next experiment.

   Append a `## Conclusion` section that states:
   - the in-scope implementation queue for Issue 799;
   - the deferred list and why each item is deferred;
   - any "automatable after setup" harness work that should happen before the
     first feature implementation;
   - the recommended Experiment 2.

9. Review the result with Codex before treating the triage as complete.

   After filling in `## Result` and `## Conclusion`, run `codex-review` against
   the completed experiment. Ask Codex to check for:
   - missing candidate APIs from the cited source documents;
   - incorrect "already solved" claims;
   - automation classifications that are too optimistic;
   - deferred items that actually have a practical automation path;
   - in-scope items whose proposed verification would not prove the feature.

   Fix all real findings before marking the experiment `Pass`, `Partial`, or
   `Fail`.

10. Update the issue README. When the result is recorded, update this
    experiment's line in `issues/0799-browser-api-automation-triage/README.md`
    from `Designed` to the final status.

## Verification

This experiment passes if:

- the candidate list is derived from issue history, `TODO.md`, and current code,
  not from memory alone;
- every candidate has exactly one automation classification;
- every in-scope candidate has a concrete automated verification proposal;
- every deferred candidate has a concrete deferral reason;
- already solved items cite the issue or source evidence that solved them;
- Codex reviews the completed triage and no blocking findings remain;
- no runtime code, protocol files, Chromium branches, patch archives, or test
  harness behavior changes are made.

This experiment is partial if:

- the inventory is useful but one or more major source areas could not be
  audited;
- Chromium source access or branch state prevents confident classification of
  Mojo binder coverage;
- Codex identifies unresolved gaps that require a second triage experiment.

This experiment fails if:

- it implements behavior instead of triaging;
- it marks manual-only work as automatable without a credible containment plan;
- it omits Issue 616, Issue 655, or `TODO.md` from the source inventory;
- it produces an implementation order without enough evidence to justify the
  automation boundary.
