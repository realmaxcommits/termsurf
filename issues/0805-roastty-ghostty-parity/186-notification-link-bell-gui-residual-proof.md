# Experiment 186: Notification/link/bell GUI residual proof

## Description

`RUNTIME-012B2B2B2B2B3` is the only remaining CFG-223 runtime/UI gap. Earlier
experiments split out deterministic URL opening, BEL dispatch, OSC desktop
notification dispatch, copied macOS bell presentation plumbing, copied macOS
user-notification lifecycle plumbing, desktop notification rate limiting,
command-finished dispatch, copied link-hover banner plumbing, deterministic link
preview predicates, context-menu selection semantics, and surface-side
`mouse_over_link` dispatch.

What remains is not generic runtime plumbing. It is the live GUI/OS-facing
surface:

- OS notification banner/sound delivery;
- actual bell audio, dock attention, border, and title effects;
- real app link hover/cursor UI;
- native link preview display;
- native context/menu display;
- OS URL-opening flows.

This experiment will audit that remaining bucket and then either close it with
focused live proof or split it into exact smaller rows if any OS-controlled
effect cannot be proven honestly in this VM. The output must eliminate the vague
`RUNTIME-012B2B2B2B2B3` bucket as a catch-all.

## Changes

- Add a focused guard:
  - `issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py`
  - It must bind the final `RUNTIME-012B2B2B2B2B3` decision to concrete evidence
    from existing deterministic guards, copied macOS source parity guards, and
    any new live GUI/OS guards created by this experiment.
  - It must fail if `RUNTIME-012B2B2B2B2B3` still contains broad unowned wording
    such as “actual OS banner/sound delivery”, “actual bell side-effect”, “real
    app hover/cursor”, “native preview display”, “native context/menu display”,
    or “OS URL-opening” without an associated row, guard command, and evidence.
- Audit the live-testability of each remaining slice:
  - desktop notifications and command-finished notifications through the built
    debug app. Closure requires all of: authorization state captured from
    `UNUserNotificationCenter`, a scheduled request identifier tied to the
    launched surface, pending/delivered notification evidence from the
    notification center, and cleanup evidence. If the VM cannot expose a pending
    or delivered request after authorization is granted, this slice must remain
    a split gap.
  - bell side effects through `bell-features` settings and live BEL or command-
    finished triggers. Closure requires deterministic proof for each enabled
    feature: `.system`/`.audio` must use an injected or traceable sound path
    rather than just source inspection; `.attention` must expose a measurable
    dock/user-attention state or remain a split gap; `.border` and `.title` must
    have screenshot/pixel or accessibility/title evidence before and after the
    bell.
  - link hover/cursor behavior by rendering a controlled link, moving the mouse
    to the exact link cell, and proving both hover-banner pixels/text and the
    active `SurfaceView` pointer style transition to `.link`. If pointer style
    cannot be read deterministically, split pointer pixels/state from hover-
    banner proof instead of closing both.
  - native link preview and context/menu behavior with right-click/control-click
    automation. Closure requires an accessibility or screenshot artifact naming
    the native menu items and a Quick Look/definition preview artifact tied to
    the controlled link/word; if the VM cannot observe the native preview, keep
    that exact preview slice open.
  - OS URL-opening flows with a controlled URL handler or another deterministic
    side-effect harness. Closure requires proof that Roastty invoked
    `NSWorkspace.open` for the expected URL and that the controlled handler
    recorded the URL. Opening an uncontrolled external browser is not enough.
- Add focused live guards for every slice that can be proven in the current VM.
  Guard names should make the proven slice explicit, for example:
  - `macos_notification_gui_runtime.py`;
  - `macos_bell_gui_runtime.py`;
  - `macos_link_hover_gui_runtime.py`;
  - `macos_link_menu_open_url_runtime.py`.
- Update `config_runtime_inventory.py`:
  - successful full closure path: mark `RUNTIME-012B2B2B2B2B3` as
    `Oracle complete`, with guard commands for every live GUI/OS proof;
  - split path: replace the broad bucket with exact adjacent rows for completed
    slices and any still-open slice, preserving honest status and missing-
    evidence text for each row.
- Regenerate `config-runtime-inventory.md` and `config-matrix.md`.
- Update all stale CFG-223 count assertions in Issue 805 guard scripts.
- Update Issue 805 `README.md` Learnings and Experiments index.

## Verification

Pass criteria for the full-closure path:

- `RUNTIME-012B2B2B2B2B3` is `Oracle complete`.
- CFG-223 reports 87 runtime rows, 84 Oracle-complete rows, 87 closed rows, 0
  incomplete rows, 0 runtime gaps, and `cfg223=Pass`.
- No current Issue 805 guard still asserts that CFG-223 has one remaining gap.
- The final residual guard fails if any live GUI/OS proof guard fails.
- Each closed slice has the deterministic artifact named above. A slice that
  lacks its artifact must be a split gap, not part of a full closure claim.

Pass criteria for a split path:

- The broad `RUNTIME-012B2B2B2B2B3` bucket no longer exists as a vague
  catch-all.
- Every completed slice has its own Oracle-complete row with a concrete guard
  command.
- Every still-open slice has its own exact row with a narrow behavior statement,
  explicit missing evidence, and a planned guard strategy.
- CFG-223 counts in `config-matrix.md` match the split exactly. The experiment
  result must state the exact runtime row count, Oracle-complete count, closed
  count, incomplete count, gap count, and remaining gap IDs.
- All Issue 805 guard scripts agree with the new counts.
- The residual guard either runs every live proof guard named by completed
  slices or statically proves the exact split rows and rejects stale
  broad-bucket text.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py
for guard in issues/0805-roastty-ghostty-parity/*_parity.py issues/0805-roastty-ghostty-parity/*_residual_audit.py issues/0805-roastty-ghostty-parity/macos_*_runtime.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$guard" || exit 1
done
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
python3 -m py_compile issues/0805-roastty-ghostty-parity/*.py
prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/186-notification-link-bell-gui-residual-proof.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
git diff --check
```

## Design Review

Fresh-context Codex adversarial reviewer `James the 3rd` reviewed the initial
design and returned `VERDICT: CHANGES REQUIRED` with two required findings:

- the verification claimed all Issue 805 guards would agree with the new CFG-223
  counts, but the command list did not run the existing guard scripts that carry
  hard-coded count assertions;
- the live GUI/OS proof strategy was too vague, because it did not name
  deterministic proof artifacts or split rules for OS notification, dock
  attention, cursor, native preview, context menu, and URL-opening behavior.

The design was updated to require deterministic closure artifacts for each
remaining slice and to force split rows when an artifact cannot be observed in
the VM. The verification command now runs all `*_parity.py`,
`*_residual_audit.py`, and `macos_*_runtime.py` guards so stale CFG-223 count
assertions cannot survive silently.

James re-reviewed the fixes and returned `VERDICT: APPROVED` with no remaining
required findings.

## Result

**Result:** Partial

The experiment eliminated the vague `RUNTIME-012B2B2B2B2B3` bucket and split it
into exact rows:

- `RUNTIME-012B2B2B2B2B3A` — **Oracle complete** for live macOS OSC notification
  request dispatch and `UNUserNotificationCenter` authorization state capture.
- `RUNTIME-012B2B2B2B2B3B` — **Oracle complete** for the live macOS
  `bell-features` bridge, app-level bell branch dispatch, surface bell state
  dispatch, and configured audio-path request trace.
- `RUNTIME-012B2B2B2B2B3C` — **Gap** for remaining OS-controlled/native GUI
  effects: actual OS notification delivery/banner/sound, audible bell output,
  measurable dock-attention state, bell border/title visible effects, real link
  hover/cursor pixels, native link preview display, native context-menu display,
  and OS URL-opening with a controlled handler.

The implementation also fixed a real app bridge bug: Swift was reading
`bell-features` through `roastty_config_get`, but `libroastty` did not implement
that key. `roastty_config_get` now returns the packed bitset for system, audio,
attention, title, and border features, with a focused Rust regression test.

The new residual guard is
`issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py`.
It statically enforces the split rows and exact CFG-223 counts, rejects the old
broad row, and runs the live macOS trace guard
`macos_notification_link_bell_trace_runtime.py`.

The live guard found that this VM reports notifications denied for the debug app
(`desktopNotification authorizationStatus=1`). Therefore this experiment cannot
honestly claim OS banner/sound delivery. Attempts to prove native context-menu
display with CGEvent right-click and Accessibility `AXShowMenu` did not produce
a deterministic `SurfaceView.menu(for:)` trace in this VM, so native menu
display remains in the exact residual gap.

Final CFG-223 split counts:

- Runtime rows: 89
- Oracle-complete rows: 85
- Closed rows: 88
- Incomplete rows: 1
- Runtime gaps: 1
- Remaining gap ID: `RUNTIME-012B2B2B2B2B3C`
- CFG-223 status: `Gap`

Verification performed:

```bash
cargo test --manifest-path roastty/Cargo.toml config_get_bell_features_runtime
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_walkthrough_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/renderer_visual_residual_audit.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py
```

`macos_walkthrough_residual_parity.py` initially exposed a repeated
Accessibility timing flake inside `macos_window_padding_pixel_runtime.py` when
run after several GUI guards. The window-padding guard passed standalone, and
the composite guard passed after wiring its existing `focus_evidence` timeout
parameter through to the internal AppleScript call.

## Conclusion

Experiment 186 made concrete progress but did not close CFG-223. The remaining
Issue 805 work is now sharply bounded to `RUNTIME-012B2B2B2B2B3C`, not a broad
notification/link/bell bucket. The next experiment should focus on obtaining
deterministic OS/native GUI proof for that row or fixing the app/VM automation
path that prevents that proof.

## Completion Review

Fresh-context Codex adversarial reviewer `Dalton the 3rd` reviewed the completed
Experiment 186 result, implementation diff, generated matrices, residual guard,
and live guard evidence. The reviewer returned:

```text
VERDICT: APPROVED

No findings.
```
