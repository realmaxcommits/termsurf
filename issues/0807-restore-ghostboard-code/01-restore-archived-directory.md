# Experiment 1: Restore the Archived Directory

## Description

Restore the archived `ghostboard/` directory to the working tree from the
documented recovery point. This experiment deliberately does not try to build,
run, modernize, rename, or integrate Ghostboard. The goal is only to make the
historical code available again.

## Changes

- Restore `ghostboard/` with:

  ```bash
  git checkout 90b966458bd17 -- ghostboard/
  ```

- Leave restored files mechanically as they were at that historical point.
- Do not edit restored Ghostboard source, scripts, project files, assets, or
  dependency metadata.
- Do not update active build/install/release scripts to include Ghostboard.
- Update this experiment file with the result after verification.

## Verification

Pass criteria:

- `ghostboard/` exists in the working tree.
- Representative expected files exist:
  - `ghostboard/build.zig`;
  - `ghostboard/src/Surface.zig`;
  - `ghostboard/src/apprt/embedded.zig`;
  - `ghostboard/macos/`;
  - `ghostboard/include/termsurf.h`.
- `git diff --stat -- ghostboard/` shows a restore of the archived directory and
  no unrelated product-code changes.
- Provenance spot checks compare representative restored files against
  `90b966458bd17`.
- `git diff --check` passes.
- The issue README lists this experiment as `Pass` when complete.

Fail criteria:

- Any file outside `ghostboard/` and issue documentation is changed for product
  behavior.
- The restore source is not the documented recovery commit.
- The experiment attempts to build or fix Ghostboard.
- Representative restored files differ from `90b966458bd17` without an explicit
  reason.

## Design Review

Fresh-context adversarial review returned `CHANGES REQUIRED`.

- Required: the original plan used
  `git checkout 90b966458bd17~1 -- ghostboard/`, which would restore a tree one
  commit older than the documented Ghostboard state. Fixed by using
  `git checkout 90b966458bd17 -- ghostboard/`.
- Required: the issue README repeated the incorrect parent-commit explanation.
  Fixed by distinguishing the documented Ghostboard tree commit `90b966458bd17`
  from the later deletion/archive commit `2874f578f`.
- Required on re-review: two stale `90b966458bd17~1` provenance references
  remained. Fixed by changing both to `90b966458bd17`.

Fresh-context adversarial re-review returned `APPROVED`.

- The reviewer confirmed the restore command now uses the documented tree state.
- The reviewer confirmed `90b966458bd17:ghostboard` and `2874f578f~1:ghostboard`
  resolve to the same tree.
- No new required findings were reported.
