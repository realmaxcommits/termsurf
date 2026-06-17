# Experiment 7: Batch F Wezboard UX Regression Audit

## Description

Classify Batch F from Experiment 4: issues `0743`-`0788`. This batch covers
Wezboard-era UX and regression work: overlay positioning, split and tab
behavior, browser input, clipboard, target blank, persistent cookies, link hover
state, DevTools, native popups, split borders, PDF bootstrap, and Chromium
upgrade preparation.

This experiment should read every Batch F issue and map each durable lesson to
current Ghostboard risk using the schema defined in Experiment 4. The output is
a classification table, not fixes.

This is an audit/documentation experiment only. It must not change application
code, generated code, historical issue files, closed issue files, scripts, test
harnesses, screenshots, or website assets.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/07-batch-f-wezboard-ux-regressions.md`
  - record this experiment design, design review, Batch F classification result,
    completion review, and conclusion;
  - classify every issue in Batch F using the Experiment 4 historical audit row
    schema.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 7 to the `## Experiments` index with status `Designed`, then
    update status after the result.

No application code, generated protobuf code, historical issue files, closed
issue files, scripts, test harnesses, screenshots, or website assets should be
edited.

## Verification

Design-gate pass criteria:

- The issue README links this experiment as `Designed`.
- A fresh-context adversarial design review approves the plan.
- The plan commit exists before implementation begins.

Implementation pass criteria:

- The result audits every Batch F issue exactly once:
  - `0743-cmd-r-reload`
  - `0744-website-icon`
  - `0745-self-hosted-git`
  - `0746-overlay-positioning`
  - `0747-multiscreen-overlay`
  - `0748-clipboard`
  - `0749-initial-overlay-flash`
  - `0750-target-blank`
  - `0751-blog-top-level`
  - `0752-scroll-inactive-pane`
  - `0753-blog-failures`
  - `0754-screenshots`
  - `0755-scroll-neovim`
  - `0756-surfari`
  - `0757-overlay-fade`
  - `0758-tui-message-routing`
  - `0759-link-hover-url`
  - `0760-cli-short-flags`
  - `0761-browser-label`
  - `0762-persistent-cookies`
  - `0763-scroll-initial`
  - `0764-viewport-profile-label`
  - `0765-terminfo-crash`
  - `0766-new-logo`
  - `0767-overlay-titlebar-offset`
  - `0768-cloudflare-website`
  - `0769-tab-id-collision`
  - `0770-browser-not-loading`
  - `0771-tab-id-collision`
  - `0772-command-shortcuts`
  - `0773-loading-screen`
  - `0774-zoom-webview-overlay`
  - `0775-devtools-multi-profile`
  - `0776-pdf-not-loading`
  - `0777-split-border-overlap`
  - `0778-back-nav-title-stale`
  - `0779-date-picker-popup-position`
  - `0780-link-drag-freeze`
  - `0781-chromium-upgrade`
  - `0782-native-popup-followups`
  - `0783-native-popup-remainders`
  - `0784-datalist-popup`
  - `0785-split-border-bottom-row`
  - `0786-grid-native-split-borders`
  - `0787-split-border-outer-margin`
  - `0788-native-popup-split-pane-y`
- The result uses the Experiment 4 row schema for every classification: source
  issue, batch, subsystem, durable lesson, current Ghostboard relevance,
  evidence paths, likelihood, risk or impact, recommended follow-up, and
  historical classification note.
- The result classifies each row as `Highly likely`, `Maybe`, or `No`, and
  explains the classification from issue evidence plus current code/test/doc
  evidence.
- The result treats open Issue `0756` as open historical evidence without trying
  to close or modify it.
- The result distinguishes Wezboard-specific UI fixes from current Ghostboard
  evidence. A Wezboard bug fix is not proof that Ghostboard works, and a
  Wezboard-only problem is not automatically a Ghostboard bug.
- The result groups or summarizes related repeated findings after the table, but
  the table itself must still contain one row per Batch F issue.
- The result carries forward relevant Issue 810 findings where Batch F overlaps
  current Ghostboard risk, especially viewport geometry coverage, cursor/link
  hover behavior, DevTools, native popup behavior, browser state updates, and
  GUI-responsibility messages.
- The result identifies the next audit slice after Batch F.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/07-batch-f-wezboard-ux-regressions.md
  ```

- Whitespace check passes:

  ```bash
  git diff --check
  ```

- A fresh-context completion review approves the completed result before the
  result commit.
- All real completion-review findings are fixed and recorded in this experiment
  file.
- The result commit is made after completion-review approval and before any next
  experiment is designed.

Fail criteria:

- Any Batch F issue is omitted or classified more than once.
- The experiment edits historical issue files, application code, scripts, tests,
  screenshots, or website assets.
- The result treats Wezboard historical fixes as current Ghostboard proof
  without current Ghostboard evidence.
- The result labels website/blog/Git infrastructure issues as Ghostboard bugs
  without a direct current product path.
- The result expands into other historical batches before Batch F is concluded.

## Design Review

Fresh-context adversarial design review returned **APPROVED**.

Reviewer checks confirmed:

- The README links Experiment 7 as `Designed`.
- Required sections are present.
- Scope is audit-only.
- Batch F matches `0743`-`0788` exactly once.
- Verification carries the Experiment 4 schema and Issue 810 findings forward.
- Issue `0756` is treated as open evidence.
- `git diff --check` passed.
- The plan commit had not yet been made before review.

Findings: none.

## Result

**Result:** Pass

Batch F was audited as the Wezboard-era UX/regression slice. The classification
unit is each historical issue folder, so the table below has exactly forty-six
rows: one for every issue from `0743` through `0788`.

### Classification Table

| Source issue                      | Batch | Subsystem                        | Durable lesson                                                                                                                   | Current Ghostboard relevance                                                                                                              | Evidence paths                                                                                                                                       | Likelihood      | Risk or impact                                                                              | Recommended follow-up                                                                                  | Historical classification note                                                                                 |
| --------------------------------- | ----- | -------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `0743-cmd-r-reload`               | F     | Browser command shortcuts        | Terminal config shortcuts must not steal browser-reserved commands in browse mode.                                               | Current Ghostboard key handling was proven for browser text input, but Cmd+R browser reload was not specifically replayed.                | Issue 743 README; Issue 809 keyboard matrix; `ghostboard/src/apprt/termsurf.zig` key forwarding.                                                     | `Maybe`         | Browser reload may be intercepted by native app/keybindings rather than Roamium.            | Add Cmd+R to a focused Ghostboard browser-command smoke.                                               | Wezboard fix is not proof that Ghostboard's Ghostty keybindings behave the same.                               |
| `0744-website-icon`               | F     | Website branding                 | Website icon pipelines should be reproducible.                                                                                   | Website asset work does not affect current Ghostboard GUI parity.                                                                         | `issues/0744-website-icon/README.md`.                                                                                                                | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Website-only historical issue.                                                                                 |
| `0745-self-hosted-git`            | F     | Infrastructure                   | Git hosting should use filesystem repos plus database metadata, not database-backed Git storage.                                 | Infrastructure decision does not affect Ghostboard runtime behavior.                                                                      | `issues/0745-self-hosted-git/README.md`.                                                                                                             | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Infrastructure-only historical issue.                                                                          |
| `0746-overlay-positioning`        | F     | Overlay geometry                 | Overlay position should be derived from the terminal render/layout pass, not a duplicate formula.                                | Current Ghostboard has its own geometry bridge and Issue 809 proves broad pane/window/tab geometry.                                       | Issue 746 README; Issue 809 conclusion; `ghostboard/macos/Sources/App/macOS/AppDelegate+TermSurf.swift`.                                             | `No`            | Covered by current Ghostboard geometry matrix except the separate multi-display caveat.     | Keep Issue 809 matrix as regression guard.                                                             | Wezboard mechanism differs, but invariant is covered.                                                          |
| `0747-multiscreen-overlay`        | F     | Overlay geometry / displays      | Initial layer placement and steady-state frame updates must not fight, especially on secondary displays.                         | Current Ghostboard verified only a single-display backing-scale path because the VM has one display.                                      | Issue 747 README; Issue 809 single-display caveat.                                                                                                   | `Maybe`         | Multi-display overlay placement could still fail on real hardware.                          | Add multi-display Ghostboard verification when hardware/VM support exists.                             | Same caveat already surfaced in Batch H.                                                                       |
| `0748-clipboard`                  | F     | Browser clipboard / command keys | Browser edit commands need explicit forwarding before terminal/menu keybindings consume them.                                    | Current Ghostboard lacks specific proof for browser Cmd+C/X/V/A/Z against editable web content.                                           | Issue 748 README; Issue 809 general keyboard proof; `ghostboard/src/apprt/termsurf.zig` key forwarding.                                              | `Maybe`         | Clipboard/edit commands in browser overlays may fail even if normal typing works.           | Add browser clipboard/edit-command cases to Ghostboard browser-command smoke.                          | Wezboard key pipeline differs from Ghostty, so direct carryover is uncertain.                                  |
| `0749-initial-overlay-flash`      | F     | Overlay creation timing          | Native layers should be created only when split-aware geometry is available.                                                     | Issue 809 initial open and split geometry passed in current Ghostboard.                                                                   | Issue 749 README; Issue 809 initial/split rows.                                                                                                      | `No`            | No current evidence of initial flash or wrong-side creation.                                | None beyond Issue 809 matrix.                                                                          | Invariant is covered by current runtime evidence.                                                              |
| `0750-target-blank`               | F     | Chromium navigation              | `target="_blank"` and `window.open()` require embedder delegate policy.                                                          | Engine-owned behavior; no Ghostboard-specific evidence suggests the GUI is involved.                                                      | Issue 750 README; Issue 799 browser API coverage lineage.                                                                                            | `No`            | If broken, owner is Roamium/Chromium delegate policy.                                       | Include target-blank in future browser API smoke only if Issue 799 coverage is judged stale.           | Chromium-side fix, not a Ghostboard GUI gap.                                                                   |
| `0751-blog-top-level`             | F     | Website content                  | Website build context must include top-level blog content.                                                                       | Website-only work does not affect Ghostboard runtime behavior.                                                                            | `issues/0751-blog-top-level/README.md`.                                                                                                              | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Website-only historical issue.                                                                                 |
| `0752-scroll-inactive-pane`       | F     | Mouse/scroll routing             | Scroll should route to the overlay under the cursor, not only the active pane.                                                   | Issue 809 proved same-tab focus switching and mouse input after geometry in current Ghostboard.                                           | Issue 752 README; Issue 809 focus/mouse matrix rows.                                                                                                 | `No`            | Current matrix covers the relevant Ghostboard invariant.                                    | Keep Issue 809 matrix.                                                                                 | Wezboard fix differs, but behavior is covered.                                                                 |
| `0753-blog-failures`              | F     | Website/blog                     | Document failed approaches honestly.                                                                                             | Blog content does not affect Ghostboard runtime behavior.                                                                                 | `issues/0753-blog-failures/README.md`.                                                                                                               | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Website/content-only historical issue.                                                                         |
| `0754-screenshots`                | F     | Website screenshots              | Screenshot processing should be reproducible and asset sizes controlled.                                                         | Website/readme screenshot pipeline does not affect current Ghostboard behavior.                                                           | `issues/0754-screenshots/README.md`.                                                                                                                 | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Website/docs asset issue.                                                                                      |
| `0755-scroll-neovim`              | F     | Hidden-tab overlay hit testing   | Hidden overlays must not intercept scroll events from visible terminal panes.                                                    | Issue 809 tab visibility, focus, scrollback, and mouse-input rows passed in current Ghostboard.                                           | Issue 755 README; Issue 809 tab/scroll/mouse matrix rows.                                                                                            | `No`            | Current evidence covers hidden-tab overlay isolation for Ghostboard.                        | Keep Issue 809 matrix.                                                                                 | Behavior covered, though implementation differs.                                                               |
| `0756-surfari`                    | F     | Future WebKit engine             | A second engine requires a WebKit profile process and CAContext-style compositing.                                               | Surfari remains open future work; current Ghostboard parity is with Roamium/Chromium.                                                     | `issues/0756-surfari/README.md`; current engine table in `AGENTS.md`.                                                                                | `No`            | No current Ghostboard bug; future multi-engine scope remains open.                          | Leave Issue 756 open. Revisit when implementing WebKit engine support.                                 | Open evidence, but intentionally outside restored Ghostboard/Roamium parity.                                   |
| `0757-overlay-fade`               | F     | Overlay visibility animation     | Native layer visibility changes should disable implicit CoreAnimation actions.                                                   | Issue 809 tab switching, minimize/hide/restore, and visibility transitions passed; no fade issue recorded.                                | Issue 757 README; Issue 809 visibility rows.                                                                                                         | `No`            | Low; current visual matrix would likely catch visible fade/hide regressions.                | None beyond Issue 809 matrix.                                                                          | Current evidence covers the relevant behavior.                                                                 |
| `0758-tui-message-routing`        | F     | TUI browser-state filtering      | TUI state updates must filter messages by tab id.                                                                                | Current `webtui` has tab filtering, and Issue 809 multi-tab/window rows passed.                                                           | Issue 758 README; `webtui/src/ipc.rs` tab filtering; Issue 809 tab/window rows.                                                                      | `No`            | Low current Ghostboard risk.                                                                | None.                                                                                                  | TUI-owned fix with current source evidence.                                                                    |
| `0759-link-hover-url`             | F     | Link hover / target URL          | Link hover URL flows directly from Roamium to webtui as `TargetUrlChanged`.                                                      | Experiment 3 keeps hover target/browser-state evidence as `Maybe`; current Ghostboard did not prove hover URL or cursor shape.            | Issue 759 README; Experiment 3 browser-state row; Batch H `CursorChanged` finding.                                                                   | `Maybe`         | Link destination text or cursor feedback may be absent under Ghostboard.                    | Add link-hover URL plus cursor-shape checks to focused Ghostboard browser-state test.                  | Direct URL path likely works, but current Ghostboard runtime evidence is missing.                              |
| `0760-cli-short-flags`            | F     | webtui CLI                       | CLI ergonomics can be solved in webtui without GUI changes.                                                                      | Current Ghostboard is not involved in `web` CLI argument parsing.                                                                         | Issue 760 README; `webtui/src/main.rs`.                                                                                                              | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | TUI-only issue.                                                                                                |
| `0761-browser-label`              | F     | BrowserReady / viewport label    | GUI should populate `BrowserReady.browser` so the TUI can display engine identity.                                               | Current Ghostboard sends `BrowserReady` and ordinary browsing works; specific label rendering was not singled out but source paths exist. | Issue 761 README; `ghostboard/src/apprt/termsurf.zig` BrowserReady path; `webtui/src/main.rs` label rendering.                                       | `No`            | Low. Missing label would be cosmetic and likely covered by visible webtui state.            | No separate follow-up unless manual use shows the label missing.                                       | Current source evidence is enough for low-risk label path.                                                     |
| `0762-persistent-cookies`         | F     | Roamium profile persistence      | Content-shell embedders must explicitly configure network persistence and cookie encryption flags.                               | Roamium/profile-owned behavior; Ghostboard launches browser profile paths but does not own Chromium network persistence.                  | Issue 762 README; Issue 808 ordinary Roamium launch proof.                                                                                           | `No`            | If cookies fail, likely Roamium/Chromium profile plumbing, not Ghostboard.                  | Include cookie persistence in later browser/product regression, not as a Ghostboard GUI bug.           | Engine-owned behavior.                                                                                         |
| `0763-scroll-initial`             | F     | Initial overlay visibility       | New overlays should be visible from birth when created for the active pane.                                                      | Issue 809 initial open and focus rows passed in current Ghostboard.                                                                       | Issue 763 README; Issue 809 initial/focus rows.                                                                                                      | `No`            | Current evidence covers the user-visible invariant.                                         | Keep Issue 809 matrix.                                                                                 | Behavior covered by restored-Ghostboard matrix.                                                                |
| `0764-viewport-profile-label`     | F     | webtui label UI                  | Profile and engine identity should be visible together in the viewport label.                                                    | TUI-owned label polish; current Ghostboard only needs to provide BrowserReady data.                                                       | Issue 764 README; `webtui/src/main.rs`; `ghostboard/src/apprt/termsurf.zig` BrowserReady path.                                                       | `No`            | Low cosmetic risk.                                                                          | No follow-up unless label is observed missing.                                                         | TUI-owned display behavior.                                                                                    |
| `0765-terminfo-crash`             | F     | Build/install assets             | Never binary-replace inside compiled terminfo; rebuild from source.                                                              | Wezboard rename/build issue, not current Ghostboard browser-overlay behavior.                                                             | Issue 765 README.                                                                                                                                    | `No`            | Low for Ghostboard; terminfo still matters generally but not from this historical path.     | None from this audit.                                                                                  | Build hygiene issue, not current GUI parity.                                                                   |
| `0766-new-logo`                   | F     | Branding assets                  | Logo/icon changes need a repeatable app and website asset process.                                                               | Asset issue; current Ghostboard branding was audited in Issue 808.                                                                        | Issue 766 README; Issue 808 branding acceptance.                                                                                                     | `No`            | None beyond existing branding checks.                                                       | None.                                                                                                  | Asset pipeline issue.                                                                                          |
| `0767-overlay-titlebar-offset`    | F     | Overlay coordinate space         | Native overlays must live in the same coordinate space as terminal content.                                                      | Issue 809 tested window/titlebar/fullscreen/minimize geometry in current Ghostboard.                                                      | Issue 767 README; Issue 809 geometry rows.                                                                                                           | `No`            | Current geometry evidence covers the invariant.                                             | Keep Issue 809 matrix.                                                                                 | Behavior covered by restored-Ghostboard matrix.                                                                |
| `0768-cloudflare-website`         | F     | Website deployment               | Static site deployment can replace server-side hosting for termsurf.com.                                                         | Website hosting does not affect Ghostboard runtime behavior.                                                                              | `issues/0768-cloudflare-website/README.md`.                                                                                                          | `No`            | None for Ghostboard behavior.                                                               | None.                                                                                                  | Website-only historical issue.                                                                                 |
| `0769-tab-id-collision`           | F     | Multi-profile tab routing        | Browser tab ids are per-process; routing keys need profile/browser identity.                                                     | Current Ghostboard stores pane profile/browser and tab lookup state, reducing risk, but multi-profile runtime proof is not explicit.      | Issue 769 README; `ghostboard/src/apprt/termsurf.zig` `TabLookupState`, `findServer`, and pane profile/browser state.                                | `Maybe`         | Multi-profile visual routing could regress if tab ids collide across browser processes.     | Add multi-profile Ghostboard smoke covering two profiles with overlapping tab ids and DevTools lookup. | Static evidence is promising; runtime proof is still missing.                                                  |
| `0770-browser-not-loading`        | F     | Build/signing/SDK compatibility  | macOS updates can invalidate Chromium sandbox/signing assumptions; rebuild against current SDK.                                  | Current build environment has recently been validated during Ghostboard work; this is operational risk, not a Ghostboard code gap.        | Issue 770 README; Issue 808/809 successful builds and runtime runs.                                                                                  | `No`            | Build failures after OS updates remain possible but are not current Ghostboard parity bugs. | Keep build/toolchain checks before runtime issues.                                                     | Environment lesson, not application behavior.                                                                  |
| `0771-tab-id-collision`           | F     | Multi-profile tab routing        | Composite `(server_key, tab_id)` routing fixed the collision without breaking browser connections.                               | Current Ghostboard appears profile/browser-aware but lacks an explicit multi-profile collision regression.                                | Issue 771 README; `ghostboard/src/apprt/termsurf.zig` profile/browser server and tab lookup state.                                                   | `Maybe`         | Same risk as Issue 769: wrong pane could receive browser layer/state in multi-profile runs. | Combine with Issue 769 follow-up: multi-profile Ghostboard routing smoke.                              | Static evidence reduces likelihood but does not prove runtime parity.                                          |
| `0772-command-shortcuts`          | F     | TUI commands / install signing   | Explicit command aliases are better than ambiguous subsequence matching; installed binaries should be signed.                    | TUI command aliases are webtui-owned; signing is build/install hygiene.                                                                   | Issue 772 README; webtui command handling; current Ghostboard build docs.                                                                            | `No`            | Low current Ghostboard risk.                                                                | None from this audit.                                                                                  | TUI/build issue, not GUI parity.                                                                               |
| `0773-loading-screen`             | F     | webtui loading UX                | Browser startup should have visible staged loading, warning, and timeout states.                                                 | webtui-owned UI; Ghostboard ordinary launch works, but slow-launch UX was not specifically rechecked.                                     | Issue 773 README; Issue 808 ordinary launch proof.                                                                                                   | `No`            | Low Ghostboard-specific risk; missing loading UX would be TUI-owned.                        | Reuse in future UX walkthrough only if startup diagnostics are weak.                                   | TUI UX issue.                                                                                                  |
| `0774-zoom-webview-overlay`       | F     | Overlay visibility on pane zoom  | Browser overlays must hide when their owning pane is hidden by zoom and restore on unzoom.                                       | Issue 809 split zoom/restore row passed in current Ghostboard.                                                                            | Issue 774 README; Issue 809 zoom row.                                                                                                                | `No`            | Current matrix covers the invariant.                                                        | Keep Issue 809 matrix.                                                                                 | Behavior covered by restored-Ghostboard matrix.                                                                |
| `0775-devtools-multi-profile`     | F     | DevTools / multi-profile routing | DevTools needs unambiguous target identity across profiles and browser processes.                                                | Issue 809 proves DevTools flow generally, but not multi-profile DevTools routing.                                                         | Issue 775 README; Issue 809 DevTools row; current `QueryDevtools` profile/browser paths in `ghostboard/src/apprt/termsurf.zig`.                      | `Maybe`         | DevTools could target the wrong tab/profile in multi-profile sessions.                      | Include DevTools in the multi-profile Ghostboard smoke from Issues 769/771.                            | General DevTools proof exists; multi-profile proof is missing.                                                 |
| `0776-pdf-not-loading`            | F     | PDF architecture discovery       | PDF viewing is a browser feature stack requiring Electron-style embedder infrastructure.                                         | Batch G already classifies current PDF rendering as needing a restored-Ghostboard smoke, not a proven Ghostboard bug.                     | Issue 776 README; Experiment 6 Batch G result.                                                                                                       | `Maybe`         | Restored Ghostboard still lacks a focused PDF smoke.                                        | Use the Batch G recommended Ghostboard PDF smoke.                                                      | Kept aligned with Experiment 6 rather than duplicating findings.                                               |
| `0777-split-border-overlap`       | F     | Wezboard split borders           | Border layout must preserve terminal content, overlay placement, mouse mapping, and resize regions.                              | Wezboard-specific rendering model; current Ghostboard uses Ghostty split UI, and Issue 809 covers overlay geometry in splits.             | Issue 777 README; Issue 809 split rows.                                                                                                              | `No`            | Low current Ghostboard risk.                                                                | None from this audit.                                                                                  | Wezboard grid/border implementation does not transfer directly.                                                |
| `0778-back-nav-title-stale`       | F     | Browser title/state propagation  | Back/forward navigation should update title along with URL.                                                                      | Experiment 3 marks broader loading/title/target/console state as `Maybe` under Ghostboard.                                                | Issue 778 README; Experiment 3 browser-state row.                                                                                                    | `Maybe`         | Back/forward title could be stale under Ghostboard even if navigation works.                | Add title/back-forward checks to focused Ghostboard browser-state test.                                | Direct path likely works, but current Ghostboard runtime proof is missing.                                     |
| `0779-date-picker-popup-position` | F     | Native browser popups            | PagePopup and select controls require correct screen geometry and separate lifecycle handling.                                   | Current Ghostboard has no native popup runtime proof; popup geometry depends on native screen rects and GUI activation.                   | Issue 779 README; Batch H `SetGuiActive` finding; Issue 809 geometry does not include native browser popups.                                         | `Maybe`         | Date/select/color popups may appear in the wrong place or persist incorrectly.              | Add native popup smoke for date/time/color/select/datalist under restored Ghostboard.                  | Strong related risk, but not proven enough for `Highly likely` except the SetGuiActive subcase in Issue 783.   |
| `0780-link-drag-freeze`           | F     | Chromium native drag suppression | Native macOS drag start should be suppressed in overlay mode to prevent freezes.                                                 | Roamium/Chromium-owned behavior; Ghostboard forwards mouse input but does not own Chromium's native drag pipeline.                        | Issue 780 README; Issue 809 mouse input proof.                                                                                                       | `No`            | If it regresses, likely Roamium/Chromium overlay-mode behavior.                             | Include link-drag in future browser API/native interaction sweep if not already covered.               | Engine-owned freeze fix.                                                                                       |
| `0781-chromium-upgrade`           | F     | Chromium upgrade workflow        | Track Electron-stable Chromium and carry patch archives forward through upgrades.                                                | Chromium workflow, not current Ghostboard runtime behavior.                                                                               | Issue 781 README; `chromium/README.md`.                                                                                                              | `No`            | Upgrade drift can affect all components, but no current Ghostboard bug.                     | Keep existing Chromium workflow.                                                                       | Build/release process issue.                                                                                   |
| `0782-native-popup-followups`     | F     | Native popup lifecycle/input     | Chromium Shell windows must be mouse-transparent so hidden engine windows do not steal AppKit events.                            | Current Ghostboard has not replayed native popup/select interactions; hidden Roamium window behavior may still matter.                    | Issue 782 README; current Ghostboard popup behavior untested.                                                                                        | `Maybe`         | Native widgets could stop opening or hidden windows could steal input.                      | Include post-select date/color/select reopening in native popup smoke.                                 | Engine/native integration issue crossing GUI boundary.                                                         |
| `0783-native-popup-remainders`    | F     | Native popups / GUI active state | PagePopup dismissal on app deactivation requires GUI-to-Roamium `SetGuiActive`; select x-position needs direct NSMenu placement. | Batch H already found current Ghostboard likely lacks `SetGuiActive`, so popup dismissal on app deactivation is a high-confidence risk.   | Issue 783 README; Experiment 3 and Batch H `SetGuiActive` findings; `ghostboard/src/apprt/termsurf.zig` lacks active `SetGuiActive` sender evidence. | `Highly likely` | PagePopup controls may remain visible after app deactivation under Ghostboard.              | Prioritize `SetGuiActive` follow-up; include PagePopup Cmd-Tab dismissal in its regression test.       | Classified `Highly likely` because this historical fix directly depends on a message currently likely missing. |
| `0784-datalist-popup`             | F     | Native datalist popup            | Datalist suggestions need a narrow TermSurf-specific bridge rather than full Chromium Autofill UI.                               | Engine-owned bridge, but restored Ghostboard has no datalist native popup proof.                                                          | Issue 784 README; current Ghostboard popup behavior untested.                                                                                        | `Maybe`         | Datalist suggestions may fail or appear incorrectly in Ghostboard-hosted overlays.          | Include datalist in native popup smoke after basic PagePopup/select checks.                            | Engine-owned but GUI-visible.                                                                                  |
| `0785-split-border-bottom-row`    | F     | Wezboard split borders           | Pixel border hacks can hide terminal rows; grid-native layout needs care.                                                        | Wezboard-specific split border rendering; Ghostboard uses Ghostty's layout, and Issue 809 covers browser overlay split behavior.          | Issue 785 README; Issue 809 split rows.                                                                                                              | `No`            | Low current Ghostboard risk.                                                                | None.                                                                                                  | Wezboard rendering implementation does not transfer directly.                                                  |
| `0786-grid-native-split-borders`  | F     | Wezboard split borders           | Split borders should reserve terminal cells and keep PTY dimensions truthful.                                                    | Wezboard-specific terminal layout change, not current Ghostboard overlay parity.                                                          | Issue 786 README; Ghostboard imports Ghostty split model.                                                                                            | `No`            | Low current Ghostboard risk.                                                                | None.                                                                                                  | Wezboard-specific rendering architecture.                                                                      |
| `0787-split-border-outer-margin`  | F     | Wezboard split borders           | Whole-cell border reservation has architectural tradeoffs; some visual goals require a new layout model.                         | Wezboard-specific architecture issue; no direct current Ghostboard bug.                                                                   | Issue 787 README.                                                                                                                                    | `No`            | None for current Ghostboard.                                                                | None.                                                                                                  | Historical negative result, not current GUI risk.                                                              |
| `0788-native-popup-split-pane-y`  | F     | Native popup geometry            | Native popup screen rects must convert flipped CALayer coordinates into AppKit view coordinates.                                 | Current Ghostboard native popup behavior is untested; split-pane geometry is proven, but popup-specific screen rect reporting is not.     | Issue 788 README; Issue 809 split geometry; current native popup behavior untested.                                                                  | `Maybe`         | Native popups could open in the wrong split pane or y-position.                             | Include native popup-in-split cases in the Ghostboard native popup smoke.                              | Related split geometry is proven; popup coordinate conversion is not.                                          |

### Findings Summary

`Highly likely` findings:

- Native PagePopup dismissal on app deactivation is likely missing because the
  historical fix depends on `SetGuiActive`, and Batch H found no current
  Ghostboard sender/handler evidence for that GUI-responsibility message.

`Maybe` findings:

- Browser command/edit keys need Ghostboard-specific proof: Cmd+R reload and
  Cmd+C/X/V/A/Z clipboard/edit commands.
- Link hover/browser-state behavior needs proof: target URL display,
  back/forward title updates, broader title/loading/console state, and browser
  cursor shape.
- Multi-profile routing needs proof for normal overlays and DevTools, despite
  promising profile/browser state in current Ghostboard.
- Native popup behavior needs a restored-Ghostboard smoke covering PagePopup,
  select, datalist, app deactivation, and split-pane coordinate cases.
- PDF smoke remains the Batch G follow-up; Issue 776 reinforces that PDF is an
  engine feature stack, not a simple GUI rendering bug.
- Multi-display overlay geometry remains unproven for the same environmental
  reason recorded in Issue 809 and Batch H.

`No` findings:

- Website/blog/Git/logo/screenshot/cloudflare issues do not map to Ghostboard
  GUI behavior.
- Most Wezboard split-border and overlay-positioning issues are covered by the
  current Ghostboard geometry matrix or are Wezboard-specific rendering
  architecture.
- Roamium/Chromium-owned fixes such as target-blank, persistent cookies, native
  drag suppression, and Chromium upgrade workflow are not current
  Ghostboard-owned bugs without additional evidence.

### Verification

Commands run:

```bash
for d in issues/07{43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88}-*; do
  sed -n '/^## Conclusion/,$p' "$d/README.md" | sed -n '1,90p'
done

sed -n '1,180p' issues/0754-screenshots/README.md
sed -n '1,180p' issues/0756-surfari/README.md
sed -n '1,180p' issues/0768-cloudflare-website/README.md
sed -n '1,180p' issues/0769-tab-id-collision/README.md
sed -n '1,180p' issues/0770-browser-not-loading/README.md

rg -n \
  "SetGuiActive|TargetUrlChanged|BrowserReady|QueryDevtools|DevTools|popup|datalist|select|clipboard|cookies|profile|tab_id|overlay|visibility|split|focus|scroll|title|loading|CursorChanged|target" \
  ghostboard/src/apprt/termsurf.zig ghostboard/macos/Sources webtui/src roamium/src \
  issues/0810-ghostboard-preventive-parity-audit/0*.md

prettier --write --prose-wrap always --print-width 80 \
  issues/0810-ghostboard-preventive-parity-audit/README.md \
  issues/0810-ghostboard-preventive-parity-audit/07-batch-f-wezboard-ux-regressions.md

git diff --check
```

Verification results:

- All forty-six Batch F issues are represented exactly once in the
  classification table.
- Every row uses the Experiment 4 schema.
- Open issue `0756` is treated as open historical evidence and was not modified.
- No historical issue files, application code, generated code, scripts, test
  harnesses, screenshots, or website assets were edited.
- Markdown formatting passed.
- Whitespace check passed.

## Conclusion

Batch F adds one `Highly likely` finding that strengthens the existing Batch H
`SetGuiActive` follow-up: native PagePopup controls likely will not dismiss on
app deactivation until Ghostboard sends GUI active/inactive state to Roamium.

The other Batch F Ghostboard risks are focused `Maybe` coverage gaps:
browser-command keys, browser clipboard/edit commands, link-hover/state updates,
multi-profile routing, native popups, PDF smoke coverage, and real multi-display
geometry.

The next audit slice should move backward to Batch E (`0715`-`0742`), because it
covers the Wezboard implementation and the Ghostboard archive transition.

## Completion Review

Fresh-context adversarial completion review returned **APPROVED**.

Reviewer checks confirmed:

- Batch F `0743`-`0788` appears exactly once: forty-six rows, with no missing or
  duplicate classifications.
- Every row follows the Experiment 4 schema.
- Issue `0756` remains open evidence only.
- Issue `0783` is defensibly classified as `Highly likely` because current
  Ghostboard lacks active `SetGuiActive` sender/handler evidence while Roamium
  handles that message and Wezboard historically sent it.
- Geometry rows are not overclaimed: ordinary Issue 809-covered geometry is
  `No`, while multi-display remains `Maybe`.
- Website and infrastructure rows are defensibly `No`.
- The issue README marks Experiment 7 as `Pass`.
- Only Issue 810 docs are changed.
- `git diff --check` passes.
- The result commit had not yet been made before review.

Findings: none.
