# Experiment 2: Decide Public macOS App Identity

## Description

Experiment 1 found that the debug Ghostboard app currently identifies as
`TermSurf`, while repo-level distribution still packages `TermSurf Wezboard.app`
and the project may eventually ship multiple TermSurf GUI apps. Before changing
bundle ids, app names, Homebrew artifacts, install paths, or installed browser
discovery, Issue 819 needs a deliberate public macOS identity contract.

This experiment will make the identity decision explicit and document it as the
baseline for later implementation experiments. It is decision/documentation
only; no app source, Xcode, packaging, or Homebrew behavior changes are planned.

## Changes

Planned issue-document changes:

- Add a result section to this experiment that records:
  - the chosen public app name;
  - the chosen installed app bundle path;
  - the chosen bundle identifier family for debug, local release, and
    distributable release builds;
  - the chosen executable and CLI names;
  - whether the app must coexist with Wezboard and future GUI apps;
  - which inherited Ghostty names remain implementation-only;
  - which user-visible Ghostty names must be fixed in later experiments.
- Update the Issue 819 README experiment status after verification.

Planned source changes:

- None.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0819-ghostboard-packaging-identity-hardening/README.md issues/0819-ghostboard-packaging-identity-hardening/02-decide-public-macos-app-identity.md`.

Static checks:

1. `git diff --check`.

Decision inputs:

1. Re-read the Issue 819 goal and Experiment 1 result.
2. Re-read the repo vision in `AGENTS.md`, especially the multiple-GUI product
   model.
3. Inspect current distribution naming in:
   - `homebrew/Casks/termsurf.rb`
   - `scripts/release.sh`
   - `scripts/install.sh`
   - `scripts/uninstall.sh`
4. Inspect current Ghostboard app identity in:
   - `ghostboard/macos/build/Debug/TermSurf.app/Contents/Info.plist`
   - `ghostboard/macos/Ghostty.xcodeproj/project.pbxproj`

Pass criteria:

- The experiment chooses a concrete public macOS app identity for Ghostboard.
- The decision accounts for coexistence with `TermSurf Wezboard.app` and future
  TermSurf GUI apps.
- The decision specifies app name, installed bundle path, bundle id family,
  executable name, CLI name if any, and Homebrew/release artifact naming.
- The decision explicitly states which Ghostty names may remain
  implementation-only and which user-visible names must be fixed.
- The decision produces a direct implementation sequence for later experiments.
- No source or packaging behavior is changed.

Partial criteria:

- The experiment narrows the app identity options but still requires a user
  product decision before implementation.

Fail criteria:

- The experiment changes app/source/release behavior before the identity
  contract is recorded.
- The decision ignores Wezboard/future GUI coexistence.
- The result leaves bundle ids or install paths ambiguous.

## Design Review

This experiment is plan-only until a fresh-context adversarial design review
approves it. Record the reviewer verdict here, fix all real findings, and commit
the approved plan before implementation begins.

Fresh-context adversarial design review by Codex subagent `Ohm the 2nd`:

- **Verdict:** Approved.
- **Findings:** None.
- **Verification:** The reviewer confirmed the README links Experiment 2 as
  `Designed`, the experiment includes the required design sections, no result is
  present yet, and the plan commit had not been made before review.

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

Experiment 2 chose a concrete public macOS identity contract for Ghostboard
without changing app source, Xcode settings, packaging scripts, or Homebrew
behavior.

### Decision Inputs

The repo vision says TermSurf is a protocol and product family, not one app.
`AGENTS.md` describes multiple GUI implementations: Wezboard, Ghostboard, and
future terminal forks. The current distribution already uses a GUI-qualified app
name, `TermSurf Wezboard.app`, in both `scripts/release.sh` and
`homebrew/Casks/termsurf.rb`.

Experiment 1 proved the current Ghostboard debug bundle is internally
TermSurf-branded:

```text
CFBundleName: TermSurf
CFBundleDisplayName: TermSurf
CFBundleIdentifier: com.termsurf.debug
CFBundleExecutable: termsurf
```

The Xcode project also currently sets:

- `PRODUCT_NAME = TermSurf`;
- `INFOPLIST_KEY_CFBundleDisplayName = TermSurf`;
- release bundle id `com.termsurf`;
- debug bundle id `com.termsurf.debug`;
- executable name `termsurf`.

Those current values work for a single app, but they are too generic for a
multi-GUI distribution that already has `TermSurf Wezboard.app` and is now
hardening Ghostboard.

### Public Identity Contract

Ghostboard should ship as a GUI-qualified TermSurf app:

| Surface                                         | Decision                                                                                                          |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Public app display name                         | `TermSurf Ghostboard`                                                                                             |
| Installed app bundle path                       | `/Applications/TermSurf Ghostboard.app`                                                                           |
| Release bundle identifier                       | `com.termsurf.ghostboard`                                                                                         |
| Debug bundle identifier                         | `com.termsurf.ghostboard.debug`                                                                                   |
| Local unsigned release bundle identifier        | `com.termsurf.ghostboard.local` if a separate local release id is needed; otherwise use `com.termsurf.ghostboard` |
| macOS executable name inside the app bundle     | `ghostboard`                                                                                                      |
| Optional CLI command name                       | `ghostboard`                                                                                                      |
| Release artifact path inside TermSurf tarball   | `TermSurf Ghostboard.app`                                                                                         |
| Homebrew cask app stanza                        | `app "TermSurf Ghostboard.app"` when Ghostboard is added to distribution                                          |
| Dock Tile plugin display name                   | `TermSurf Ghostboard Dock Tile Plugin`                                                                            |
| Dock Tile plugin bundle id                      | `com.termsurf.ghostboard.dock-tile`                                                                               |
| User-facing config docs and UI                  | TermSurf/Ghostboard names only; no Ghostty instructions                                                           |
| AppleScript dictionary title/suite/descriptions | TermSurf Ghostboard names                                                                                         |
| AppleScript Cocoa class names                   | May remain `GhosttyScript...` unless a focused automation compatibility experiment proves they should be migrated |
| Xcode project/target/source directory names     | May remain Ghostty-named as implementation inheritance unless they leak to users or block packaging               |
| iOS target                                      | Out of scope for this macOS packaging issue                                                                       |

This contract intentionally differs from the current debug bundle. The current
`TermSurf.app`/`com.termsurf.debug`/`termsurf` identity should be treated as a
temporary debug baseline, not the final distributable Ghostboard identity.

### Coexistence Decision

Ghostboard must coexist with Wezboard and future TermSurf GUI apps. Therefore:

- `TermSurf` remains the product/protocol/tap/release family name.
- GUI app bundles use `TermSurf <GuiName>.app`.
- GUI bundle identifiers use `com.termsurf.<gui-name>`.
- GUI app executables and optional CLI names use the lowercase GUI name.
- shared components keep shared names: `web`, `roamium`, and the TermSurf
  protocol/config/data roots where applicable.

This keeps Finder, LaunchServices, logs, Activity Monitor, Homebrew cask
artifacts, and future support instructions distinguishable.

### Implementation Sequence

The next implementation experiments should use this order:

1. Rename Ghostboard macOS app bundle identity in Xcode/build outputs:
   `PRODUCT_NAME`, display name, bundle id family, executable name, app icon
   reference if needed, Dock Tile plugin identity, and built app path.
2. Align user-visible Ghostty leakage: Settings UI strings, AppleScript
   dictionary title/suite/descriptions, service/menu names where they do not
   already inherit the display name, and docs generated from Ghostty templates.
3. Prove and align the config path with the chosen identity and existing
   TermSurf XDG contract.
4. Add repo-level build/install/uninstall/release support for
   `TermSurf Ghostboard.app`.
5. Add Homebrew packaging for Ghostboard using the chosen app stanza and shared
   Roamium artifact policy.
6. Define installed Roamium discovery for Ghostboard and add a regression test
   that distinguishes debug `TERMSURF_ROAMIUM_PATH` from installed app behavior.

Verification completed:

1. Re-read the Issue 819 goal and Experiment 1 result.
2. Re-read the multi-GUI product model in `AGENTS.md`.
3. Re-inspected current install/uninstall/release/Homebrew naming in
   `scripts/install.sh`, `scripts/uninstall.sh`, `scripts/release.sh`, and
   `homebrew/Casks/termsurf.rb`.
4. Re-inspected the current Ghostboard debug bundle and Xcode project identity.
5. Ran `prettier` on the Issue 819 README and this experiment file.
6. Ran `git diff --check`.

## Conclusion

Issue 819 now has an app identity contract: Ghostboard should become
`TermSurf Ghostboard.app` with bundle identifiers under
`com.termsurf.ghostboard` and executable/CLI name `ghostboard`.

The next experiment should implement and verify the macOS app bundle identity
change in Ghostboard's Xcode/build outputs, while preserving inherited
implementation-only Ghostty names that do not leak into user-visible packaging.

## Completion Review

Fresh-context adversarial completion review by Codex subagent `Maxwell the 2nd`:

- **Verdict:** Approved.
- **Required findings:** None.
- **Optional finding:** The completed verification list mentioned
  `scripts/release.sh` and `homebrew/Casks/termsurf.rb` but omitted the planned
  `scripts/install.sh` and `scripts/uninstall.sh` inputs. Fixed by adding both
  files to the verification list.
