# Experiment 1: Build, Publish, and Verify 1.0.0

## Description

Publish TermSurf `1.0.0` through the existing release workflow and verify that
the resulting Homebrew cask installs a usable production build.

The experiment has a package-only preflight before the irreversible publish
step. If the preflight finds stale paths, missing assets, a dirty tap, or a
broken tarball layout, stop and record the result instead of publishing.

## Changes

Planned release actions:

1. Confirm the repository and Homebrew tap are clean.

   ```bash
   git status --short
   git -C homebrew status --short --branch
   ```

2. Confirm GitHub release and tap preconditions.

   ```bash
   gh auth status
   gh release view v1.0.0 --repo termsurf/termsurf --json tagName,name,url,assets
   git -C homebrew fetch origin main
   git -C homebrew status --short --branch
   ```

   Expected preflight state:

   - `gh` is authenticated with permission to create releases.
   - `v1.0.0` does not already exist.
   - `homebrew/` is on `main` and not behind `origin/main`.

   If `v1.0.0` already exists, stop. Do not run `scripts/release.sh 1.0.0`,
   because the release script deletes any existing release before creating the
   replacement. Replacing an existing `1.0.0` release requires explicit user
   approval, a recorded rationale, and a recovery plan.

3. Record the design review and commit this experiment plan.

   Before running any build, package, publish, or install command:

   - record the design review findings and fixes in this file;
   - obtain reviewer approval after any fixes;
   - commit the approved experiment plan separately from the later result
     commit.

4. Build all release artifacts.

   ```bash
   scripts/build.sh all --release
   ```

5. Run a package-only release preflight.

   ```bash
   TERMSURF_RELEASE_PACKAGE_ONLY=1 scripts/release.sh 1.0.0
   ```

6. Inspect the generated tarball before publishing.

   ```bash
   tar tzf dist/termsurf-1.0.0-aarch64-apple-darwin.tar.gz | sort | sed -n '1,200p'
   shasum -a 256 dist/termsurf-1.0.0-aarch64-apple-darwin.tar.gz
   ```

   The tarball must contain:

   - `./TermSurf.app/`
   - `./web`
   - `./roamium/roamium`
   - Chromium runtime resources under `./roamium/`

   The tarball must not contain stale Wezboard app or path names.

7. Publish the release.

   ```bash
   scripts/release.sh 1.0.0
   ```

   This should create GitHub release `v1.0.0`, upload
   `termsurf-1.0.0-aarch64-apple-darwin.tar.gz`, update
   `homebrew/Casks/termsurf.rb` to `version "1.0.0"` and the matching SHA, and
   push the tap commit to `termsurf/homebrew-termsurf`.

8. Verify the published release and tap.

   ```bash
   gh release view v1.0.0 --repo termsurf/termsurf --json tagName,name,url,assets
   git -C homebrew status --short --branch
   git -C homebrew log --oneline -5
   sed -n '1,220p' homebrew/Casks/termsurf.rb
   ```

9. Verify Homebrew install behavior.

   Use the local machine unless a permission or cache issue requires a clean
   second machine.

   ```bash
   brew update
   brew uninstall --cask termsurf || true
   brew install --cask termsurf
   command -v web
   test "$(command -v web)" = "/opt/homebrew/bin/web"
   ls -ld /Applications/TermSurf.app
   ls -l /opt/homebrew/opt/termsurf-roamium/roamium
   test -f /opt/homebrew/opt/termsurf-roamium/roamium
   test -f /opt/homebrew/opt/termsurf-roamium/icudtl.dat
   test -f /opt/homebrew/opt/termsurf-roamium/gen/chrome/pdf_resources.pak
   test -f /opt/homebrew/opt/termsurf-roamium/gen/chrome/generated_resources_en-US.pak
   test -f /opt/homebrew/opt/termsurf-roamium/gen/chrome/common_resources.pak
   test -f /opt/homebrew/opt/termsurf-roamium/gen/components/components_resources.pak
   test -f /opt/homebrew/opt/termsurf-roamium/gen/components/strings/components_strings_en-US.pak
   test -f /opt/homebrew/opt/termsurf-roamium/gen/extensions/extensions_renderer_resources.pak
   find /opt/homebrew/opt/termsurf-roamium -maxdepth 1 -name '*.dylib' -type f | grep .
   find /opt/homebrew/opt/termsurf-roamium -maxdepth 1 -name '*.pak' -type f | grep .
   find /opt/homebrew/opt/termsurf-roamium -maxdepth 1 -name 'v8_context_snapshot*.bin' -type f | grep .
   find /opt/homebrew/opt/termsurf-roamium -type f | sed -n '1,160p'
   brew uninstall --cask termsurf
   brew install --cask termsurf
   ```

10. Smoke-test the installed production app.

    ```bash
    open -a /Applications/TermSurf.app
    ```

    In the launched TermSurf window, run:

    ```bash
    /opt/homebrew/bin/web --browser /opt/homebrew/opt/termsurf-roamium/roamium https://example.com/
    ```

    Verify that the page loads in Roamium and record concrete log evidence that
    the installed app used the Homebrew paths:

    - `/opt/homebrew/bin/web`
    - `/opt/homebrew/opt/termsurf-roamium/roamium`

    If available, also run the existing installed-release harness:

    ```bash
    TERMSURF_GHOSTBOARD_APP=/Applications/TermSurf.app \
      TERMSURF_INSTALLED_ROAMIUM_PATH=/opt/homebrew/opt/termsurf-roamium/roamium \
      scripts/ghostboard-geometry-matrix.sh installed-roamium-release-launch
    ```

11. Record the result, run completion review, and commit the result.

    After verification:

    - append `## Result` and `## Conclusion` to this file;
    - update the README experiment status to `Pass`, `Partial`, or `Fail`;
    - run the required fresh-context completion review;
    - fix any real completion-review findings;
    - commit the experiment result separately from the plan commit.

## Verification

Pass criteria:

- Release build succeeds.
- Package-only preflight creates a tarball with the expected production layout.
- The tarball and cask contain no stale Wezboard names.
- `scripts/release.sh 1.0.0` succeeds.
- GitHub release `v1.0.0` exists with the expected tarball asset.
- The Homebrew tap cask is pushed at `version "1.0.0"` with the tarball SHA.
- `brew install --cask termsurf` installs `/Applications/TermSurf.app`, `web`,
  and Roamium under `/opt/homebrew/opt/termsurf-roamium`.
- `command -v web` is exactly `/opt/homebrew/bin/web`.
- Explicit checks prove required Chromium resources exist under
  `/opt/homebrew/opt/termsurf-roamium`, including the generated resources copied
  by `scripts/roamium-resources.sh`.
- The installed app can run `web https://example.com/` and load the page.
- The runtime smoke test records evidence that the installed app used
  `/opt/homebrew/bin/web` and `/opt/homebrew/opt/termsurf-roamium/roamium`, not
  a stale debug or legacy path.

Fail criteria:

- Any release artifact is missing.
- The package layout would prevent Roamium from finding Chromium resources.
- The cask references stale Wezboard app names or removed install paths.
- `v1.0.0` already exists and explicit replacement approval has not been given.
- The GitHub release or Homebrew tap publish fails.
- The installed production app cannot run `web` and load a page.

If the publish succeeds but local Homebrew verification is blocked by local
machine state, record the publish result as **Partial** and continue with a
second verification experiment.

## Design Review

Fresh-context adversarial design review returned **CHANGES REQUIRED**.

Required findings:

- Existing-release handling was too permissive because `scripts/release.sh`
  deletes an existing `v1.0.0` before recreating it.
- Homebrew install verification did not explicitly prove `web` resolved from
  `/opt/homebrew/bin/web`.
- Roamium asset-root verification did not check deep generated Chromium resource
  paths.
- Runtime smoke testing did not record concrete evidence that the installed app
  used Homebrew `web` and Homebrew Roamium paths.
- The design did not explicitly list the required plan and result workflow
  gates.

Fixes applied:

- The publish precondition now requires `v1.0.0` to be absent, otherwise the
  experiment stops unless the user explicitly approves replacement with a
  recorded rationale and recovery plan.
- Homebrew install verification now asserts
  `test "$(command -v web)" = "/opt/homebrew/bin/web"`.
- Roamium verification now includes explicit `test -f` checks for the generated
  resources copied by `scripts/roamium-resources.sh`, plus checks for dylibs,
  top-level pak files, `icudtl.dat`, and V8 snapshots.
- Runtime smoke testing now runs
  `/opt/homebrew/bin/web --browser /opt/homebrew/opt/termsurf-roamium/roamium`
  and requires path evidence in logs.
- The plan now includes explicit design-review, plan-commit, completion-review,
  and result-commit gates.

Re-review result:

- Fresh-context adversarial re-review returned **APPROVED** with no remaining
  Required findings.
