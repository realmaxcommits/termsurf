# Experiment 7: Add Ghostboard Script Component

## Description

Issue 819's remaining packaging work depends on repo-level scripts recognizing
Ghostboard as a first-class component. The current scripts support `wezboard`,
`roamium`, `webtui`, `chromium`, and `all`, but not Ghostboard. Release and
Homebrew packaging should build on a stable local build/install/uninstall
contract rather than inventing separate paths.

This experiment will add `ghostboard` to the repo-level build, install, and
uninstall scripts. It will not change release tarball packaging or Homebrew cask
contents yet.

## Changes

Planned script changes:

- `scripts/build.sh`
  - Add `ghostboard` to usage/component lists.
  - Add a `build_ghostboard` function that runs
    `ghostboard/macos/build.nu --configuration Debug --action build` by default
    and `--configuration Release --action build` for `--release`.
  - Include Ghostboard in `all`.
  - Keep `--clean` conservative. If a clean Ghostboard build is requested, use
    an established project command only if one exists; otherwise remove the
    local `ghostboard/macos/build/{Debug,Release}/TermSurf Ghostboard.app`
    product before building.
- `scripts/install.sh`
  - Add `ghostboard` to usage/component lists.
  - Install `ghostboard/macos/build/Release/TermSurf Ghostboard.app` to
    `/Applications/TermSurf Ghostboard.app`.
  - Fail with a clear message if the release app bundle is missing, telling the
    user to run `scripts/build.sh ghostboard --release`.
  - Codesign the installed app ad hoc, following the existing Wezboard install
    style.
  - Include Ghostboard in `all`.
- `scripts/uninstall.sh`
  - Add `ghostboard` to usage/component lists.
  - Remove `/Applications/TermSurf Ghostboard.app`.
  - Include Ghostboard in `all`.

Planned issue-document changes:

- Add `## Result` and `## Conclusion` after verification.
- Update the Issue 819 README experiment status after verification.

Explicitly out of scope:

- `scripts/release.sh` tarball contents.
- Homebrew cask changes.
- Installed Roamium discovery behavior.
- Runtime config or app source changes.
- Installing a CLI symlink for `ghostboard`; the macOS product is an app bundle
  with executable `Contents/MacOS/ghostboard`.

## Verification

Formatting actions:

1. Format edited issue Markdown:

   ```bash
   prettier --write --prose-wrap always --print-width 80 \
     issues/0819-ghostboard-packaging-identity-hardening/README.md \
     issues/0819-ghostboard-packaging-identity-hardening/07-add-ghostboard-script-component.md
   ```

Static checks:

1. `bash -n scripts/build.sh scripts/install.sh scripts/uninstall.sh`.
2. `git diff --check`.
3. Confirm usage strings include `ghostboard` and `all` includes the new
   component:

   ```bash
   rg -n 'ghostboard|Components:' scripts/build.sh scripts/install.sh scripts/uninstall.sh
   ```

Runtime/build checks:

1. Build Ghostboard through the repo-level script:

   ```bash
   scripts/build.sh ghostboard
   ```

2. Verify the debug app exists and has the expected executable:

   ```bash
   test -x 'ghostboard/macos/build/Debug/TermSurf Ghostboard.app/Contents/MacOS/ghostboard'
   ```

3. Verify the existing config-path smoke still works against the script-built
   app:

   ```bash
   scripts/ghostboard-geometry-matrix.sh ghostboard-config-paths
   ```

4. Build Ghostboard release through the repo-level script:

   ```bash
   scripts/build.sh ghostboard --release
   ```

5. Verify the release app exists and has the expected executable:

   ```bash
   test -x 'ghostboard/macos/build/Release/TermSurf Ghostboard.app/Contents/MacOS/ghostboard'
   ```

6. Run an actual install/uninstall smoke after the release build exists:

   ```bash
   sudo scripts/install.sh ghostboard
   test -x '/Applications/TermSurf Ghostboard.app/Contents/MacOS/ghostboard'
   sudo scripts/uninstall.sh ghostboard
   test ! -e '/Applications/TermSurf Ghostboard.app'
   ```

   If `/Applications` cannot be safely touched in this VM, record the result as
   `Partial` instead of `Pass`.

Pass criteria:

- `scripts/build.sh ghostboard` builds
  `ghostboard/macos/build/Debug/TermSurf Ghostboard.app`.
- `scripts/build.sh ghostboard --release` builds
  `ghostboard/macos/build/Release/TermSurf Ghostboard.app`.
- `scripts/build.sh` usage and `all` include Ghostboard.
- `scripts/install.sh` can install from the release Ghostboard app bundle path
  to `/Applications/TermSurf Ghostboard.app` and has a clear missing-build
  error.
- `scripts/uninstall.sh` removes `/Applications/TermSurf Ghostboard.app`.
- `scripts/install.sh` and `scripts/uninstall.sh` usage and `all` include
  Ghostboard.
- The config-path smoke still passes against the script-built debug app.
- No release tarball, Homebrew cask, installed Roamium discovery, or runtime app
  behavior changes are made.

Partial criteria:

- Build script support is complete, but install/uninstall cannot be safely
  runtime-tested against `/Applications` in the VM.
- Install/uninstall support is added but `all` intentionally remains unchanged
  because release/install behavior should not include Ghostboard yet.

Fail criteria:

- Ghostboard support is added to release/Homebrew packaging before the local
  script contract is verified.
- The build script invokes stale `TermSurf.app` or `termsurf` paths.
- The config-path smoke regresses.

## Design Review

This experiment is plan-only until a fresh-context adversarial design review
approves it. Record the reviewer verdict here, fix all real findings, and commit
the approved plan before implementation begins.

Fresh-context adversarial design review by Codex subagent `Ptolemy the 2nd`:

- **Initial verdict:** Changes required.
- **Required finding:** Runtime verification only built the debug app even
  though install support depends on the release app bundle. Fixed by adding a
  `scripts/build.sh ghostboard --release` check and verifying the release app
  executable exists.
- **Required finding:** Install/uninstall verification was too vague while pass
  criteria still claimed install/uninstall support. Fixed by requiring an actual
  `sudo scripts/install.sh ghostboard` and
  `sudo scripts/uninstall.sh ghostboard` smoke after the release build exists,
  with `Partial` required if `/Applications` cannot be safely touched.
- **Re-review verdict:** Approved.

## Completion Gate

After implementation and verification:

- add `## Result` and `## Conclusion` to this experiment file;
- update the Issue 819 README experiment status from `Designed` to `Pass`,
  `Partial`, or `Fail`;
- request a fresh-context completion review;
- fix all real completion-review findings and record the final verdict in this
  file; and
- commit the reviewed result separately before designing or implementing the
  next experiment.
