# Experiment 33: Acceptance Audit and Walkthrough

## Description

Experiment 32 made the visible normal Roamium overlay interactive. Before adding
more implementation, this experiment will audit Issue 808's acceptance criteria
against the current tree and run a focused ordinary-browsing walkthrough.

The goal is to determine whether Ghostboard is already close enough to close the
issue or, if not, to identify the next concrete blocker with evidence. This is a
verification and audit experiment first. It should not make product code changes
unless the audit finds a tiny documentation-only correction needed to record the
result.

This experiment will specifically check:

- history-preserving Ghostty `v1.3.1` import evidence;
- build and app launch evidence;
- `TermSurf.app` identity, bundle executable, app icon, menu/about naming, and
  config path;
- whether the "CLI command is `termsurf`" requirement is currently satisfied by
  the app bundle executable only, or whether a standalone CLI artifact remains a
  gap;
- ordinary browsing with the real `webtui` and Chromium-output Roamium;
- the current protocol surface that ordinary browsing depends on: socket/env
  propagation, `Hello`, `SetOverlay`, `ServerRegister`, `CreateTab`, `TabReady`,
  `BrowserReady`, direct browser socket, `CaContext`, overlay presentation,
  resize/focus/mode/input, `web last`, DevTools split flow, cleanup;
- browser state behavior for URL, title, loading state, and target URL, noting
  whether it is delivered over the direct browser socket or still needs GUI
  routing;
- known cleanup debt, including leftover Roamium listen sockets.

The expected result is an acceptance matrix with each criterion marked **Pass**,
**Partial**, **Fail**, or **Not tested**, plus the recommended next experiment.

## Changes

Expected files:

- `issues/0808-recreate-ghostboard-from-ghostty-1-3-1/33-acceptance-audit-and-walkthrough.md`
  - record the audit procedure, evidence, matrix, result, and conclusion.
- `issues/0808-recreate-ghostboard-from-ghostty-1-3-1/README.md`
  - add Experiment 33 to the experiment index.

No product code changes are planned. In particular, this experiment will not
modify:

- `ghostboard/` source code;
- `webtui/`;
- `roamium/`;
- `chromium/`;
- `proto/termsurf.proto`;
- build scripts;
- app assets.

If the audit discovers a product bug, record it and design a follow-up
experiment instead of fixing it in this experiment.

## Verification

Pass criteria:

- The audit reads Issue 808's acceptance criteria and produces a matrix that
  covers every bullet in `README.md`.
- The audit records concrete evidence for each **Pass** or **Partial** item:
  issue experiment references, current file paths, command output, logs, or
  screenshots.
- Current source checks include:
  - `git status --short`;
  - `git log --oneline --max-count=20 -- ghostboard`;
  - `git log --grep='Import Ghostty v1.3.1' --oneline`;
  - `git merge-base --is-ancestor 22efb0be2bbea73e5339f5426fa3b20edabcaa11 HEAD`
    or equivalent tag/reachability evidence proving the exact Ghostty `v1.3.1`
    commit is reachable from TermSurf history;
  - `git remote -v | rg ghostty`;
  - bundle metadata from
    `ghostboard/macos/build/Debug/TermSurf.app/Contents/Info.plist`;
  - current config path references from `ghostboard/src/config/file_load.zig`
    and related user-facing config help;
  - menu/about branding source checks from
    `ghostboard/macos/Sources/App/macOS/MainMenu.xib` and
    `ghostboard/macos/Sources/Features/About/AboutView.swift`, or screenshot
    evidence if source checks are inconclusive;
  - app icon resource checks for `TermSurf.icns` and the Wezboard-derived source
    icon evidence from Experiment 6.
- Build checks either reuse the latest valid logs from Experiments 31 and 32 or
  rerun:
  - `cargo build -p webtui`;
  - `./scripts/build.sh roamium`;
  - `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`
    inside `ghostboard/`;
  - `macos/build.nu --scheme Ghostty --configuration Debug --action build`
    inside `ghostboard/`.
- Runtime walkthrough launches `ghostboard/macos/build/Debug/TermSurf.app` as a
  bundle with a temporary config that runs the real debug `web` binary with
  `/Users/astrohacker/dev/termsurf/chromium/src/out/Default/roamium`. If a
  bundle launch cannot be automated in this VM, mark the `TermSurf.app` launch
  acceptance item **Partial** or **Not tested** instead of using direct
  executable launch as full proof.
- Runtime walkthrough proves ordinary browsing still works after Experiment 32:
  - visible `Example Domain` overlay screenshot;
  - `ModeChanged ... browsing=true`;
  - keyboard and pointer input reach Roamium;
  - `web last` returns the normal tab.
- Runtime walkthrough checks browser state behavior:
  - the TUI title or visible UI updates to `Example Domain`;
  - the URL field is `https://example.com/`;
  - loading state reaches done or an equivalent loaded proof is recorded;
  - if target-url hover is not practically automatable in this run, record it as
    **Not tested** instead of inferring success.
- Runtime cleanup check records no stale matching
  `TermSurf.app/Contents/MacOS/termsurf`, `target/debug/web`, or
  `chromium/src/out/Default/roamium` processes and records whether GUI and
  Roamium listen sockets remain.
- The audit records whether ignored GUI-side messages such as `UrlChanged`,
  `TargetUrlChanged`, `LoadingState`, and `TitleChanged` are harmless because
  `webtui` receives them over the direct browser socket, or whether they are a
  real parity gap.
- `git diff --check` is clean.
- `git status --short` is checked against an explicit allowlist and shows only
  the issue README and Experiment 33 document, including untracked paths.
- `git diff --name-only` also shows only the issue README and Experiment 33
  document.

Fail criteria:

- The experiment changes product code.
- The acceptance matrix omits any Issue 808 acceptance criterion.
- A **Pass** item relies only on assertion without concrete evidence.
- The audit hides a known failure by calling it pass.
- Runtime proof uses fake `webtui`, fake Roamium, direct protocol injection, or
  a product path different from the current app unless that limitation is
  explicitly recorded as **Not tested** or **Partial**.
- The audit claims preserved upstream Ghostty history without proving the exact
  `v1.3.1` commit `22efb0be2bbea73e5339f5426fa3b20edabcaa11` is reachable.
- The audit claims app bundle launch, menu branding, or about-page branding
  without bundle launch evidence, source evidence, or screenshot evidence.
- The audit uses `git diff --name-only` alone to prove no product code changed
  while untracked paths could exist.

## Design Review

A fresh-context adversarial Codex subagent reviewed the Experiment 33 design and
returned **CHANGES REQUIRED** with three required findings:

- import-history verification did not prove the exact upstream Ghostty `v1.3.1`
  commit was reachable from TermSurf history;
- app-bundle launch and menu/about branding were not concretely verified;
- `git diff --name-only` could miss untracked product files.

All three findings were accepted. The design now requires exact
`22efb0be2bbea73e5339f5426fa3b20edabcaa11` reachability evidence, source or
screenshot proof for menu/about branding, bundle-launch proof or an explicit
Partial/Not-tested classification, and status-based path allowlisting that
accounts for untracked files.

The same reviewer re-reviewed the updated design and returned **APPROVED**. The
reviewer confirmed that all three prior required findings were resolved and
found no remaining required findings.
