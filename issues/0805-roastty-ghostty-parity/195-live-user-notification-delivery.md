# Experiment 195: Live user notification delivery

## Description

Experiment 194 closed external Launch Services handler delivery. The remaining
`RUNTIME-012B2B2B2B2B3C` gap is now limited to actual OS notification
delivery/banner/sound after authorization is available, audible bell output, and
OS-visible Dock attention bounce/state beyond AppKit request dispatch.

This experiment targets only actual macOS user-notification delivery. Existing
guards prove Roastty reaches the desktop-notification request path and record
that this VM currently reports denied notification authorization
(`authorizationStatus=1`), but they do not prove that an authorized app can add
a notification that appears in `UNUserNotificationCenter`'s delivered
notification list.

The goal is to prove live delivered-notification state through
`UNUserNotificationCenter.getDeliveredNotifications`, or to document the exact
macOS authorization boundary that prevents deterministic proof in this VM.

## Changes

- Add a focused live guard, tentatively
  `issues/0805-roastty-ghostty-parity/macos_live_user_notification_delivery.py`.
  - Launch the built debug Roastty app with isolated config/defaults,
    `desktop-notifications = true`, `macos-applescript = true`, and a trace
    path.
  - Add an env-gated AppleScript/UI-test action, only if needed, that asks the
    focused `SurfaceView` to schedule a notification with a deterministic title,
    body, and `requireFocus=false` through the same `showUserNotification` path
    used by production desktop notifications.
  - Record `UNUserNotificationCenter` authorization status and notification
    settings before scheduling.
  - If authorization is `.authorized`, schedule the notification and require:
    - `userNotification request ...` trace evidence;
    - `userNotification added ... tracked=true` trace evidence;
    - `getDeliveredNotifications` evidence containing the exact notification
      identifier, title, subtitle, body, category, and `userInfo` surface ID.
  - If authorization is not `.authorized`, stop before claiming notification
    delivery and record the exact status/settings as the remaining authorization
    boundary.
  - Remove any delivered notifications created by the guard and check for new
    Roastty crash reports.
- Update `macos_user_notification_runtime_parity.py` so the existing copied
  macOS user-notification lifecycle parity guard still passes with intentional
  Roastty UI-test trace hooks, without weakening its source-parity checks for
  notification request construction, category/action registration, authorization
  gates, delivered-notification cleanup, and notification response handling.
- Update `config_runtime_inventory.py` according to the result:
  - If delivered notification proof passes, split a new Oracle-complete row from
    `RUNTIME-012B2B2B2B2B3C` for actual user-notification delivery.
  - Keep notification banner/sound presentation separate unless the guard has a
    real OS-visible banner/sound oracle.
  - If authorization remains denied, leave an exact gap row naming the denied
    authorization state and do not claim notification delivery parity.
- Update `notification_link_bell_gui_residual_parity.py` to enforce the new row
  split or exact authorization-boundary wording.
- Regenerate `config-runtime-inventory.md` and `config-matrix.md`.
- Update Issue 805 `README.md` Learnings and Experiments index after the result
  is known.

## Verification

Pass criteria:

- The guard proves exact debug-app launch, isolated config/defaults,
  AppleScript/UI-test gating, and no new Roastty crash report.
- The guard records `UNUserNotificationCenter` authorization settings before
  scheduling.
- If authorization is `.authorized`, the guard proves the production
  `showUserNotification` path added the deterministic notification and that
  `getDeliveredNotifications` returns the exact notification content.
- If authorization is not `.authorized`, the result is `Partial` or `Fail`, and
  the issue records the exact authorization/settings boundary without claiming
  notification delivery.
- The result does not claim notification banner/sound pixels/audio, audible bell
  output, or OS-visible Dock attention bounce/state.
- Inventory counts and remaining gap IDs are updated exactly and asserted by
  guards.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_user_notification_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_live_user_notification_delivery.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
python3 -m py_compile issues/0805-roastty-ghostty-parity/*.py
rm -rf issues/0805-roastty-ghostty-parity/__pycache__
prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/195-live-user-notification-delivery.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

The result must state the exact runtime row count, Oracle-complete count, closed
count, incomplete count, gap count, CFG-223 status, and remaining gap IDs.

## Design Review

Fresh-context Codex adversarial reviewer `Pasteur the 3rd` reviewed the initial
design and returned **Changes Required**.

Required finding accepted: the design omitted the existing focused
`macos_user_notification_runtime_parity.py` guard for copied macOS
user-notification lifecycle parity, and that guard is currently broken because
it does not account for newer intentional Roastty UI-test trace hooks. The
design now includes updating and running that guard before the new live
notification-delivery guard.

Re-review verdict after the fix: **Approved**.

## Result

**Result:** Partial

The experiment fixed the existing static macOS user-notification lifecycle guard
and added a live same-app delivered-notification guard, but the current VM still
blocks actual notification delivery before scheduling.

Implementation changes:

- `macos_user_notification_runtime_parity.py` now normalizes the intentional
  Roastty UI-test trace hooks and current CFG-223 counts/residual wording while
  preserving the source-parity checks for notification request construction,
  category/action registration, authorization gates, delivered-notification
  cleanup, and notification response handling.
- `ScriptTerminal.perform(action:)` gained an env-gated
  `ui_test_user_notification` action.
- `SurfaceView.showUserNotification` gained an optional identifier parameter so
  the live guard can use a deterministic notification ID while production
  callers still get a UUID.
- `SurfaceView.showUITestUserNotification` records notification settings,
  schedules the deterministic notification only when authorization is
  `.authorized`, and then queries
  `UNUserNotificationCenter.getDeliveredNotifications` from the Roastty app
  process.
- `macos_live_user_notification_delivery.py` launches the built debug app with
  isolated config/defaults, invokes the env-gated action through AppleScript,
  and records either exact delivered-notification content or the exact
  authorization boundary.

The passing live guard recorded the authorization boundary:

- `authorization_status = 1`;
- `alert_setting = 2`;
- `sound_setting = 2`;
- trace `userNotification settings status=1 alert=2 sound=2`;
- trace `userNotification uiTestAction=blocked status=1`;
- no new Roastty crash report.

Because authorization was not `.authorized`, the guard did not schedule the
notification and did not claim delivered-notification parity. If a future VM run
reports `authorization_status = 2`, the same guard will require the
deterministic notification ID, title, subtitle, body, category, surface ID, and
`requireFocus=false` userInfo to appear in `getDeliveredNotifications`.

The regenerated CFG-223 counts are unchanged:

- runtime rows: 98
- Oracle complete: 94
- closed: 97
- audit covered: 0
- incomplete: 1
- runtime gaps: 1
- CFG-223 status: `Gap`

Verification logs:

- `logs/issue805-exp195-build-2.log`
- `logs/issue805-exp195-user-notification-2.log`
- `logs/issue805-exp195-config-runtime-inventory-1.log`
- `logs/issue805-exp195-residual-guard-2.log`

Additional verification:

- `PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_user_notification_runtime_parity.py`
- `python3 -m py_compile issues/0805-roastty-ghostty-parity/macos_live_user_notification_delivery.py issues/0805-roastty-ghostty-parity/macos_user_notification_runtime_parity.py`
- `prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/195-live-user-notification-delivery.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md`
- `git diff --check`

## Completion Review

Fresh-context Codex adversarial reviewer `Linnaeus the 3rd` reviewed the
completed experiment, implementation diff, issue/inventory updates, and
verification logs.

Verdict: **Approved**.

Findings: no required findings.

Optional finding accepted and fixed: the first implementation used callback
forms that introduced new Swift concurrency warnings in
`SurfaceView_AppKit.swift`. The code now uses async UserNotifications APIs for
notification add/settings/delivered queries, and the rebuilt
`logs/issue805-exp195-build-2.log` no longer reports the experiment-specific
UserNotifications concurrency warnings. Remaining build warnings are preexisting
unrelated warnings.

## Conclusion

Experiment 195 narrowed the notification slice to an OS authorization boundary:
the current VM reports denied notification authorization before scheduling, so
actual delivered notification/banner/sound proof remains in
`RUNTIME-012B2B2B2B2B3C`. The guard is now ready to prove delivered notification
content on an authorized macOS state.
