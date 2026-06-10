+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 57: Phase F — clipboard behavior config

## Description

Experiments 54-56 made the font and clipboard codepoint-map config slices
usable. The next narrow Phase-F slice is the remaining clipboard behavior fields
that Ghostty stores on `Surface` and uses in copy/paste actions:

- `clipboard-trim-trailing-spaces`
- `clipboard-paste-protection`
- `clipboard-paste-bracketed-safe`
- `selection-clear-on-copy`

Roastty currently has the lower-level copy formatter, paste safety, paste
encoding, and selection mutation machinery, but these behaviors are hardcoded or
not represented on `Config`. This experiment represents the fields and threads
them into the existing app action paths. It intentionally excludes unrelated
clipboard policy fields already present (`clipboard-read` / `clipboard-write`),
OSC52/Kitty clipboard policy changes, write-file actions, and any new C ABI.

## Changes

- `roastty/src/config/mod.rs`
  - Add the four config fields with upstream defaults:
    - `clipboard-trim-trailing-spaces = true`
    - `clipboard-paste-protection = true`
    - `clipboard-paste-bracketed-safe = true`
    - `selection-clear-on-copy = false`
  - Route the fields through `Config::set`, `format_config`, clone/equality, CLI
    loading, and diagnostics through the existing bool-field machinery.
  - Preserve upstream declaration order in `format_config`:
    `selection-clear-on-copy` belongs near the selection config group, and the
    three `clipboard-*` fields belong with the clipboard group after
    `clipboard-write`.
  - Add parser/default/formatter-order tests and config-load tests for CLI and
    file input.
- `roastty/src/lib.rs`
  - Store the fields in `App`/surface state or read them from the existing
    parsed app config snapshot, following the least invasive local pattern.
  - Replace hardcoded copy formatting trim behavior in
    `Surface::copy_to_clipboard` with `clipboard-trim-trailing-spaces`.
  - After a successful `copy_to_clipboard` action, clear the active selection
    and request render when `selection-clear-on-copy` is true. Do not clear
    selection for copy-on-select or URL copying.
  - Thread `clipboard-paste-protection` and `clipboard-paste-bracketed-safe`
    into paste completion so unsafe-paste confirmation behavior matches Ghostty:
    paste protection disabled always allows, explicitly confirmed pastes allow,
    bracketed pastes containing the closing bracket remain unsafe, and bracketed
    pastes otherwise follow `clipboard-paste-bracketed-safe`.
  - Keep `roastty_paste_is_safe` and `roastty_paste_encode` C helpers unchanged;
    this experiment is app/surface config wiring, not ABI expansion.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, add a durable operating note for the clipboard
    behavior config fields.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/config/mod.rs roastty/src/lib.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/57-clipboard-behavior-config.md`
- Run targeted tests:
  - `cargo test -p roastty clipboard_behavior_config`
  - `cargo test -p roastty config_format_config`
  - `cargo test -p roastty surface_binding_action_copy_to_clipboard`
  - `cargo test -p roastty surface_binding_action_copy_url_to_clipboard`
  - `cargo test -p roastty paste_from_clipboard`
  - `cargo test -p roastty clipboard_read_completion`
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the four clipboard behavior fields are represented on `Config`,
round-trip through config loading/formatting, and affect only the intended
copy/paste action paths; targeted and full tests pass.

**Partial** = config representation lands, but one runtime behavior exposes a
bounded missing prerequisite in paste completion or selection clearing; record
the exact gap and leave any hardcoded behavior explicit.

**Fail** = the current surface/app config ownership cannot safely route these
fields without a larger config-state refactor.

## Design Review

Reviewed by Codex adversarial reviewer (`Newton`,
`019eb316-3fd8-70e0-9205-a79632491736`).

**Verdict:** Approved.

No findings.
