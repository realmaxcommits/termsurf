# Experiment 4: Historical Issue Inventory

## Description

Start the historical issue epic with a complete inventory of the issue archive.
Issue 810 must audit all historical issues, including issues from older
prototypes and subprojects that may not directly target Ghostboard. Before
classifying individual historical lessons, this experiment builds the coverage
map that later audit slices will use.

The result should enumerate every prior issue folder, define audit batches, and
record the schema later experiments must use when mapping historical lessons to
current Ghostboard risk.

This is an audit/documentation experiment only. It must not change application
code, generated code, historical issue files, closed issue files, scripts, or
test harnesses.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/04-historical-issue-inventory.md`
  - record this experiment design, design review, result, completion review, and
    conclusion;
  - record the full prior-issue inventory and batch plan after implementation.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 4 to the `## Experiments` index with status `Designed`, then
    update status after the result.

No application code, generated protobuf code, historical issue files, closed
issue files, scripts, or test harnesses should be edited.

## Verification

Design-gate pass criteria:

- The issue README links this experiment as `Designed`.
- A fresh-context adversarial design review approves the plan.
- The plan commit exists before implementation begins.

Implementation pass criteria:

- The result computes the authoritative prior-issue set from the filesystem:
  every `issues/[0-9][0-9][0-9][0-9]-*/` folder with number lower than `0810`.
- The result records:
  - total prior issue count;
  - first and last issue numbers;
  - any numbering gaps or duplicate issue numbers;
  - which prior issues are currently open versus closed, using
    `issues/README.md` as the index evidence;
  - any mismatch between filesystem issue folders, `issues/README.md` index
    entries, and README frontmatter status;
  - audit batches suitable for later experiments, with every prior issue
    assigned to exactly one batch.
- The result defines the historical audit row schema for later experiments:
  source issue, subsystem, durable lesson, current Ghostboard relevance,
  evidence paths, likelihood (`Highly likely`, `Maybe`, `No`), risk or impact,
  and recommended follow-up.
- The result must not classify every issue yet. It may include sample
  classifications only to validate the schema, but the full classification work
  belongs in later audit-slice experiments.
- The result identifies the next audit slice. Expected next slice: classify the
  newest/highest-signal Ghostboard/Roastty/Wezboard era batch first, unless the
  inventory reveals a better batch boundary.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/04-historical-issue-inventory.md
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

- Any prior issue folder is omitted from the inventory.
- Any prior issue is assigned to zero batches or more than one batch.
- The experiment edits historical issue files or application code.
- The result presents a partial inventory as complete.
- The result performs the full historical classification instead of creating the
  coverage map and batch plan.

## Design Review

Fresh-context adversarial design review returned **APPROVED**.

Optional finding:

- The status source could be more robust. The design used filesystem folders as
  the authoritative prior-issue set but used `issues/README.md` as status
  evidence; if the index is stale or omits a folder, the inventory should report
  that mismatch.

Fix:

- Added a requirement to report any mismatch between filesystem issue folders,
  `issues/README.md` index entries, and README frontmatter status.

## Result

**Result:** Pass

The prior-issue inventory was computed from filesystem issue folders matching
`issues/[0-9][0-9][0-9][0-9]-*/` with numeric prefix lower than `0810`.

Summary:

- Prior issue folders: `320`.
- Unique numeric issue ids: `318`.
- First prior issue number: `0001`.
- Last prior issue number: `0809`.
- Frontmatter status count: `315` closed, `5` open.
- Prior open issues: `0756`, `0795`, `0797`, `0798`, `0803`.
- Filesystem/index/frontmatter mismatches: none found.
- Batch assignment check: all `320` prior issue folders assigned to exactly one
  batch.

### Numbering Gaps and Duplicates

Numbering gaps in the historical archive:

- `0004`-`0099` before `0100`
- `0109`-`0199` before `0200`
- `0211`-`0300` before `0301`
- `0351`-`0399` before `0400`
- `0419`-`0499` before `0500`
- `0516`-`0599` before `0600`

Duplicate numeric prefixes:

- `0401`: `0401-chromium-feasibility`, `0401-programming-language`
- `0410`: `0410-partial-electron`, `0410-two-profiles-2`

These duplicates are historical records and should not be renamed in this issue.
Later historical audit batches must treat folders, not only numeric ids, as the
coverage unit.

### Status Cross-Check

The inventory cross-checked:

- filesystem issue folders below `0810`;
- index entries in `issues/README.md`;
- `status = "..."` frontmatter in each issue README.

Every prior issue folder appeared in `issues/README.md`, every prior index entry
had a matching folder, and frontmatter status matched index status for all prior
issues.

### Audit Row Schema

Later historical audit experiments should use this row schema:

| Field                          | Meaning                                                                 |
| ------------------------------ | ----------------------------------------------------------------------- |
| Source issue                   | Issue folder and title.                                                 |
| Batch                          | Historical inventory batch from this experiment.                        |
| Subsystem                      | Protocol, GUI, TUI, Roamium, Chromium, PDF, website, build, etc.        |
| Durable lesson                 | The lasting product or engineering lesson from the historical issue.    |
| Current Ghostboard relevance   | Why the lesson could or could not affect restored Ghostboard.           |
| Evidence paths                 | Current code, tests, issue docs, logs, or lack of evidence.             |
| Likelihood                     | `Highly likely`, `Maybe`, or `No`.                                      |
| Risk or impact                 | User-visible or engineering impact if the finding is real.              |
| Recommended follow-up          | Later experiment, issue, regression guard, or no action.                |
| Historical classification note | Brief reason the row was classified without rewriting historical facts. |

### Audit Batches

The historical audit should proceed in batches. Each prior issue folder below is
assigned exactly once.

### Batch A: Early prototypes and architecture

Range: `0001`-`0350`. Count: 73.

```text
0001-competitors
0002-merge-upstream
0003-website
0100-bookmarks
0101-build
0102-console
0103-ctrl-z
0104-keybindings
0105-libghostty
0106-release
0107-target-blank
0108-webview
0200-architecture
0201-cef
0202-cef-mvp
0203-cef-mvp2
0204-cef-mvp3
0205-cef-mvp4
0206-cef-mvp5
0207-cef-wezterm
0208-profile
0209-web
0210-wezterm-analysis
0301-architecture
0302-webview
0303-xpc
0304-webpage
0305-profile
0306-resize
0307-profile
0308-resize
0309-resize
0310-tabs
0311-resize
0312-stealing-focus
0313-helper
0314-control
0315-mode
0316-dim
0317-input
0318-cmd
0319-mouse
0320-double-click
0321-scroll
0322-drag-selection
0323-shift-click
0324-cursor-feedback
0325-webview-framerate
0326-process-lifecycle
0327-scroll-speed
0328-caret
0329-focus-lifecycle
0330-multi-pane
0331-termsurf
0332-profile-reconnect
0333-control-panel-profile
0334-copy-url
0335-navigation
0336-background
0337-refresh
0338-lag
0339-electron
0340-architecture
0341-performance
0342-perf-no-win
0343-optimal-performance
0344-cef-test
0345-benchmark
0346-mouse-performance
0347-lingering-lag
0348-cef-test-ceiling
0349-bimodal
0350-callback
```

### Batch B: Chromium/Electron feasibility and Ghostboard iterations

Range: `0400`-`0515`. Count: 37.

```text
0400-a-new-hope
0401-chromium-feasibility
0401-programming-language
0402-wezterm-vs-alacritty
0403-swift-rust-cpp
0404-terminal-emulator
0405-architecture-comparison
0406-chromium
0407-chromium-poc
0408-two-profiles
0409-electron-patch
0410-partial-electron
0410-two-profiles-2
0411-two-profiles-3
0412-one-profile
0413-one-profile-2
0414-two-profiles-xpc
0415-swift-receiver
0416-rust-receiver
0417-ghostty-vs-wezterm
0418-repo-restructure
0500-rename
0501-two-profiles
0502-attach-delay
0503-one-two-three
0504-web-tui
0505-pink-texture
0506-xpc-gateway
0507-chromium
0508-checkerboard
0509-chromium
0510-two-profiles
0511-three-profiles
0512-vsync
0513-ctrl-esc
0514-mouse
0515-drag
```

### Batch C: TermSurf/Ghostboard product hardening

Range: `0600`-`0679`. Count: 80.

```text
0600-termsurf-ghost
0601-zig-xpc
0602-pink-texture
0603-box-demo
0604-two-panes
0605-two-profiles
0606-mouse-input
0607-keyboard-input
0608-search-input
0609-keyboard-input-2
0610-app-icon
0611-rename
0612-icon
0613-rename-directories
0614-docs-review
0615-xdg
0616-web-features
0617-alpha
0618-url-sync
0619-input-latency
0620-zig-content-shell
0621-single-process
0622-javascript-is-slow
0623-viz-display-serialization
0624-chromium-ipc
0625-calayerhost
0626-x-y-calayerhost
0627-resize-calayerhost
0628-navigation-calayerhost
0629-understand-nav-calayerhost
0630-nav-calayerhost-6
0631-continue-nav-calayerhost
0632-nav-flicker-calayerhost
0633-persistent-compositor
0634-calayerhost-audit
0635-multi-pane-calayerhost
0636-calayerhost-audit
0637-editable-url-bar
0638-page-title
0639-open-in-same-tab
0640-project-cleanup
0641-chromium-patches
0642-zig-profile-server
0643-zig-profile-server-2
0644-simplified-cpp
0645-audit-xdg
0646-normal-insert
0647-tui-restructure
0648-devtools-research
0649-control-mode
0650-installation
0651-bundle-identifier
0652-termsurf-cli
0653-xpc-gateway
0654-cmd-h
0655-substack-blank
0656-rename-script
0657-url-edit-color
0658-edtui-improvements
0659-command-mode
0660-lazyvim-tokyonight-colors
0661-title-spacing
0662-context-menu
0663-js-context-menu
0664-clap
0665-esc
0666-devils-esc
0667-active-pane
0668-fix-resize
0669-active-pane
0670-click-to-focus
0671-app-icon
0672-border-padding
0673-consolidate-scripts
0674-homepage
0675-hello-message
0676-url-normalization
0677-website-deps
0678-website-lint-format
0679-license
```

### Batch D: Direct browser, protocol, Roamium, and migration setup

Range: `0680`-`0714`. Count: 35.

```text
0680-dark-mode
0681-quitall
0682-direct-xpc
0683-visited-links
0684-devtools
0685-multi-profile-tracking
0686-chromium-crash
0687-one-devtools
0688-devtools-split
0689-tab-lifecycle
0690-devtools-split
0691-devtools-direct-command
0692-file-subcommand
0693-smart-resolve
0694-tab-id-chromium
0695-suppress-activation-drag
0696-double-click-suppression
0697-update-docs
0698-unix-sockets
0699-protobuf-build
0700-tui-gui-sockets
0701-chromium-sockets
0702-socket-cleanup
0703-remove-click-suppression
0704-browser-bindings
0705-browser-bindings
0706-plusium-devtools
0707-roamium
0708-roamium-only
0709-wezboard
0710-gecko-webkit-ladybird
0711-rename-ghostboard-webtui
0712-engine-label
0713-rename-homepage-website
0714-seven-digit-issues
```

### Batch E: Wezboard implementation and Ghostboard archive

Range: `0715`-`0742`. Count: 28.

```text
0715-wezboard
0716-wezboard-warnings
0717-remove-cocoa-crate
0718-finish-cocoa-removal
0719-wezboard-code-smells
0720-wezboard-manual-test
0721-wgpu-upgrade
0722-cargo-deps
0723-pane-borders
0724-wezboard-protocol
0725-wezboard-overlay
0726-wezboard-overlay-lifecycle
0727-wezboard-second-webview
0728-wezboard-remaining-protocol
0729-wezboard-reposition-and-protocol
0730-roamium-standalone-install
0731-wezboard-scroll-crash
0732-wezboard-reopen-tab
0733-ghostboard-shutdown
0734-build-scripts
0735-ghostboard-release-icon
0736-roamium-process-leak
0737-wezboard-icon
0738-wezboard-text-selection
0739-build-warnings
0740-wezboard-display-name
0741-protocol-split
0742-archive-ghostboard
```

### Batch F: Wezboard UX, overlays, popups, and regressions

Range: `0743`-`0788`. Count: 46.

```text
0743-cmd-r-reload
0744-website-icon
0745-self-hosted-git
0746-overlay-positioning
0747-multiscreen-overlay
0748-clipboard
0749-initial-overlay-flash
0750-target-blank
0751-blog-top-level
0752-scroll-inactive-pane
0753-blog-failures
0754-screenshots
0755-scroll-neovim
0756-surfari
0757-overlay-fade
0758-tui-message-routing
0759-link-hover-url
0760-cli-short-flags
0761-browser-label
0762-persistent-cookies
0763-scroll-initial
0764-viewport-profile-label
0765-terminfo-crash
0766-new-logo
0767-overlay-titlebar-offset
0768-cloudflare-website
0769-tab-id-collision
0770-browser-not-loading
0771-tab-id-collision
0772-command-shortcuts
0773-loading-screen
0774-zoom-webview-overlay
0775-devtools-multi-profile
0776-pdf-not-loading
0777-split-border-overlap
0778-back-nav-title-stale
0779-date-picker-popup-position
0780-link-drag-freeze
0781-chromium-upgrade
0782-native-popup-followups
0783-native-popup-remainders
0784-datalist-popup
0785-split-border-bottom-row
0786-grid-native-split-borders
0787-split-border-outer-margin
0788-native-popup-split-pane-y
```

### Batch G: PDF and browser API automation

Range: `0789`-`0799`. Count: 11.

```text
0789-electron-style-pdf-viewer
0790-pdf-viewer-mojo-bindings
0791-app-shell-foundation
0792-pdf-support
0793-pdf-iframe-size
0794-pdf-viewer-interactions
0795-pdf-native-print
0796-pdf-implementation-audit
0797-pdf-core-workflow-coverage
0798-pdf-advanced-features
0799-browser-api-automation-triage
```

### Batch H: Roastty and restored Ghostboard

Range: `0800`-`0809`. Count: 10.

```text
0800-roastty-architecture
0801-roastty-libghostty-rewrite
0802-libroastty-completion-and-mac-app
0803-roastty-debug-overlay
0804-roastty-gui-automation-readiness
0805-roastty-ghostty-parity
0806-roastty-input-latency
0807-restore-ghostboard-code
0808-recreate-ghostboard-from-ghostty-1-3-1
0809-ghostboard-viewport-geometry
```

### Verification

Commands run:

```bash
find issues -maxdepth 1 -type d -name '[0-9][0-9][0-9][0-9]-*' |
  sort |
  awk -F/ '{base=$2; num=substr(base,1,4); if (num < "0810") print base}' |
  wc -l

find issues -maxdepth 1 -type d -name '[0-9][0-9][0-9][0-9]-*' |
  sort |
  awk -F/ '{base=$2; num=substr(base,1,4); if (num < "0810") print num}' |
  sort -n |
  uniq |
  wc -l

find issues -maxdepth 1 -type d -name '[0-9][0-9][0-9][0-9]-*' |
  sort |
  awk -F/ '{base=$2; num=substr(base,1,4); if (num < "0810") print num}' |
  awk 'NR==1{prev=$1+0; printf "first=%04d\n", prev; next}
       {cur=$1+0; if (cur > prev+1) printf "gap %04d..%04d before %04d\n", prev+1, cur-1, cur; prev=cur}
       END{printf "last=%04d\n", prev}'

awk '/^## Open/{section="open"; next}
     /^## Closed/{section="closed"; next}
     /^\| \[[0-9][0-9][0-9][0-9]\]/ {
       if (match($0, /\[([0-9][0-9][0-9][0-9])\]/)) {
         print substr($0, RSTART+1, 4), section
       }
     }' issues/README.md

ruby -e 'Dir["issues/[0-9][0-9][0-9][0-9]-*/README.md"].sort.each do |p|
  base=p.split("/")[1]; num=base[0,4]; next unless num < "0810"
  txt=File.read(p); status=txt[/^status\s*=\s*"([^"]+)"/,1] || "MISSING"
  puts "#{num} #{status} #{base}"
end'

prettier --write --prose-wrap always --print-width 80 \
  issues/0810-ghostboard-preventive-parity-audit/README.md \
  issues/0810-ghostboard-preventive-parity-audit/04-historical-issue-inventory.md

git diff --check
```

Verification results:

- Prior folder count: `320`.
- Unique numeric id count: `318`.
- Duplicate numeric prefixes: `0401`, `0410`.
- Frontmatter status count: `315` closed, `5` open.
- Index/frontmatter/filesystem mismatches: none.
- Batch assignment cardinality: `{1=>320}`; every prior folder matched exactly
  one batch.
- Markdown formatting passed.
- Whitespace check passed.

## Conclusion

The historical issue audit now has a complete coverage map. Later experiments
can audit one batch at a time and prove coverage by checking against this
inventory instead of rediscovering the archive structure.

The next experiment should classify Batch H (`0800`-`0809`) first. It is the
newest and highest-signal batch for restored Ghostboard because it covers
Roastty, Ghostboard restoration, recreation from Ghostty, and Issue 809's
viewport-geometry proof.

## Completion Review

Fresh-context adversarial completion review returned **APPROVED**.

Reviewer checks confirmed:

- Counts match: `320` prior folders, `318` unique numeric ids, `315` closed, and
  `5` open.
- Duplicate prefixes and numbering gaps are correctly reported.
- Filesystem/index/frontmatter mismatch claim is credible: none found.
- All prior folders are assigned to exactly one batch.
- The README marks Experiment 4 as `Pass`.
- Only Issue 810 docs are changed.
- `git diff --check` passes.
- The result commit had not yet been made before review.

Findings: none.
