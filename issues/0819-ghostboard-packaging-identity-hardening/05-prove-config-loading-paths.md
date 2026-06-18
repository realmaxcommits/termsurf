# Experiment 5: Prove Config Loading Paths

## Description

Experiments 1 and 4 found conflicting config-path evidence. Source comments and
TermSurf docs point at `$XDG_CONFIG_HOME/termsurf/config`, while inherited macOS
docs and Settings UI history pointed at Ghostty paths such as
`~/.config/ghostty/config.ghostty` and
`~/Library/Application Support/com.mitchellh.ghostty/config.ghostty`.

This experiment will prove the current Ghostboard runtime config loading
behavior under controlled environment paths before changing config docs,
Settings UI specificity, generated mdgen text, or fallback behavior.

## Changes

Planned source changes:

- `scripts/ghostboard-geometry-matrix.sh`
  - Add a focused config-path scenario, only if the existing harness cannot
    already prove the needed path behavior.
  - The scenario should launch `TermSurf Ghostboard.app` with controlled `HOME`,
    `XDG_CONFIG_HOME`, and/or `GHOSTTY_CONFIG_PATH` values and verify which
    config file is read from app logs or visible `HelloReply` behavior.

Planned issue-document changes:

- Add `## Result` and `## Conclusion` after verification.
- Update the Issue 819 README experiment status after verification.

Explicitly out of scope:

- Changing actual config loading behavior.
- Changing mdgen docs, Settings UI path specificity, release packaging, or
  Homebrew packaging.
- Renaming internal `GHOSTTY_CONFIG_PATH` environment variables or broad config
  implementation symbols.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0819-ghostboard-packaging-identity-hardening/README.md issues/0819-ghostboard-packaging-identity-hardening/05-prove-config-loading-paths.md`.

Static checks:

1. `git diff --check`.

Runtime proof:

1. Run a controlled config-path smoke that proves explicit config-file loading:

   ```bash
   scripts/ghostboard-geometry-matrix.sh ghostboard-config-paths
   ```

2. The scenario should create distinguishable config files under temporary
   locations, for example:

   - `$RUN_DIR/xdg/termsurf/config`
   - `$RUN_DIR/xdg/ghostty/config.ghostty`
   - `$RUN_DIR/home/Library/Application Support/com.mitchellh.ghostty/config.ghostty`
   - `$RUN_DIR/home/Library/Application Support/com.termsurf/config.ghostty`
   - `$RUN_DIR/home/Library/Application Support/com.termsurf/config`
   - `$RUN_DIR/home/Library/Application Support/com.termsurf.ghostboard/config.ghostty`
   - `$RUN_DIR/home/Library/Application Support/com.termsurf.ghostboard/config`
   - an explicit config file passed through `GHOSTTY_CONFIG_PATH`

3. Use config values already observable through existing plumbing, such as
   `homepage = ...` or `browser = ...`, so the proof can verify the loaded file
   through `HelloReply`/webtui state trace or app log lines rather than relying
   only on file existence.

Pass criteria:

- The experiment proves the current highest-priority explicit config path, if
  any.
- The experiment proves whether `$XDG_CONFIG_HOME/termsurf/config` is loaded.
- The experiment proves whether inherited Ghostty XDG/macOS fallback paths are
  loaded.
- The experiment proves whether current bundle-id-derived macOS Application
  Support paths such as `com.termsurf` or `com.termsurf.ghostboard` are loaded,
  including both `config.ghostty` and `config` filename candidates where
  relevant.
- The experiment records the observed precedence order when multiple candidate
  config files exist.
- The experiment records the exact log/state evidence used to identify the
  loaded file.
- No config loading behavior is changed.

Partial criteria:

- The experiment proves explicit `GHOSTTY_CONFIG_PATH` behavior but cannot prove
  default fallback behavior because the app or harness always injects an
  explicit config path.
- The experiment proves behavior through app logs but cannot prove it through
  `HelloReply`/webtui trace because the chosen config value is not observable.

Fail criteria:

- The experiment changes config loading behavior before recording the current
  contract.
- The runtime proof cannot distinguish which config file was loaded.
- The result updates docs or Settings UI path specificity without proving the
  runtime path.

## Design Review

This experiment is plan-only until a fresh-context adversarial design review
approves it. Record the reviewer verdict here, fix all real findings, and commit
the approved plan before implementation begins.

Fresh-context adversarial design review by Codex subagent `Arendt the 2nd`:

- **Initial verdict:** Changes required.
- **Required finding:** The plan omitted current bundle-id-derived macOS
  Application Support candidates such as
  `$HOME/Library/Application Support/com.termsurf/config.ghostty`. Fixed by
  adding `com.termsurf` and `com.termsurf.ghostboard` Application Support
  sentinels with both `config.ghostty` and `config` filename candidates, and by
  adding a pass criterion for proving those paths.
- **Re-review verdict:** Approved. The reviewer confirmed the sentinel list and
  pass criteria now cover inherited Ghostty, `com.termsurf`, and
  `com.termsurf.ghostboard` Application Support candidates and introduce no new
  Required finding.

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

## Result

**Result:** Pass

Implemented a focused `ghostboard-config-paths` scenario in
`scripts/ghostboard-geometry-matrix.sh`. The scenario launches
`TermSurf Ghostboard.app` in isolated subcases with temporary `HOME` and
`XDG_CONFIG_HOME` values. Each subcase seeds every relevant candidate path with
a different observable `homepage` value and a complete `initial-command`, then
verifies the loaded file through both app logs and `HelloReply`/webtui behavior.

The runtime proof passed:

```bash
scripts/ghostboard-geometry-matrix.sh ghostboard-config-paths
```

Observed subcases:

- `explicit-env`: with `GHOSTTY_CONFIG_PATH` set, Ghostboard read the explicit
  file: `$RUN_DIR/config-paths-explicit-env/explicit-config`.
- `xdg-default`: without `GHOSTTY_CONFIG_PATH`, Ghostboard read
  `$XDG_CONFIG_HOME/termsurf/config`.
- `no-current-xdg`: without `GHOSTTY_CONFIG_PATH` and without a current
  `$XDG_CONFIG_HOME/termsurf/config`, Ghostboard did not load any seeded
  inherited config candidate. It logged creation of the default
  `$XDG_CONFIG_HOME/termsurf/config` template and used default Hello config
  values.

Across those subcases, the seeded inherited candidates were not selected:

- `$XDG_CONFIG_HOME/ghostty/config.ghostty`
- `$HOME/Library/Application Support/com.mitchellh.ghostty/config.ghostty`
- `$HOME/Library/Application Support/com.termsurf/config.ghostty`
- `$HOME/Library/Application Support/com.termsurf/config`
- `$HOME/Library/Application Support/com.termsurf.ghostboard/config.ghostty`
- `$HOME/Library/Application Support/com.termsurf.ghostboard/config`

Evidence came from app log lines such as:

- `reading configuration file path=...`
- `creating template config file: path=...`
- `TermSurf Hello config homepage=... browsers=roamium`
- `TermSurf HelloReply sent homepage=... browsers=roamium`
- `SetOverlay: pane_id=... profile=default browser=roamium url=...`

The explicit and XDG-loaded cases also verified that the no-`--browser` webtui
launch used the debug Roamium resolver through `TERMSURF_ROAMIUM_PATH`, so the
scenario did not depend on an installed Roamium.

Formatting and static checks:

```bash
bash -n scripts/ghostboard-geometry-matrix.sh
prettier --write --prose-wrap always --print-width 80 \
  issues/0819-ghostboard-packaging-identity-hardening/README.md \
  issues/0819-ghostboard-packaging-identity-hardening/05-prove-config-loading-paths.md
git diff --check
```

## Conclusion

The current Ghostboard config-loading contract is now proven:

1. `GHOSTTY_CONFIG_PATH` is the highest-priority explicit config path.
2. Without `GHOSTTY_CONFIG_PATH`, Ghostboard loads
   `$XDG_CONFIG_HOME/termsurf/config`.
3. If `$XDG_CONFIG_HOME/termsurf/config` is absent, Ghostboard does not fall
   back to the tested inherited Ghostty XDG or macOS Application Support paths;
   it creates the default TermSurf XDG template and uses default runtime config
   values.

Future config docs and Settings UI work should describe
`$XDG_CONFIG_HOME/termsurf/config` as the normal Ghostboard config location
unless a later experiment intentionally changes the runtime behavior.

## Completion Review

Fresh-context adversarial completion review by Codex subagent `Faraday the 2nd`:

- **Initial verdict:** Changes required.
- **Required finding:** The first implementation proved explicit
  `GHOSTTY_CONFIG_PATH` and current TermSurf XDG precedence, but did not prove
  whether inherited Ghostty or bundle-id-derived Application Support paths were
  loaded when `$XDG_CONFIG_HOME/termsurf/config` was absent. Fixed by adding the
  `no-current-xdg` subcase, which seeds the inherited/App Support candidates
  while omitting the current TermSurf XDG config, then verifies that no config
  file is loaded, no fallback sentinel homepage is consumed, and the default
  TermSurf XDG template path is used.
- **Re-review verdict:** Approved.
