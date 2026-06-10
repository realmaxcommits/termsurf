+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 56: Phase F — clipboard codepoint map

## Description

Experiment 54 ported the parser/formatter type for `clipboard-codepoint-map`,
but the field is not yet part of `Config` and copy operations do not use it.
Ghostty threads `config.@"clipboard-codepoint-map"` into terminal formatting
when copying text to the clipboard, so configured replacements such as
box-drawing characters to ASCII are applied to copied text. URL copying is
explicitly excluded upstream.

This experiment makes `clipboard-codepoint-map` a first-class Roastty config
field and applies it to app copy-to-clipboard formatting. The scope is the copy
path only; it does not broaden the C formatter ABI, write-file actions, URL
copying, or unrelated clipboard policy fields.

## Changes

- `roastty/src/config/mod.rs`
  - Add `clipboard_codepoint_map: RepeatableClipboardCodepointMap` to `Config`,
    with the upstream default of an empty map.
  - Route `clipboard-codepoint-map` through `Config::set`, `format_config`, and
    config equality/clone through the normal derived behavior.
  - Keep the existing `RepeatableClipboardCodepointMap` parse/format semantics:
    repeated entries accumulate, an empty map formats as a void entry, `U+XXXX`
    replacements become codepoints, and all other valid UTF-8 replacements are
    literal strings.
  - Add config tests for default state, parser routing, formatter order, and
    round-trip output through `format_config`.
- `roastty/src/terminal/page_list.rs` / `roastty/src/terminal/terminal.rs`
  - Expose only the narrow terminal-internal API needed to construct formatter
    codepoint-map entries from config, or add a conversion helper inside the
    terminal module so `lib.rs` does not reach into private page-list internals.
  - Preserve existing formatter behavior: replacements are applied in reverse
    entry order so later overlapping mappings win, codepoint replacements update
    pin/point maps by the replacement character byte length, and string
    replacements map every emitted byte to the original cell.
  - Add a terminal-level test that formats a selection with a codepoint map and
    proves both codepoint and string replacements flow through
    `Terminal::selection_format` / the helper used by app copy.
- `roastty/src/lib.rs`
  - Store the parsed config's clipboard codepoint map in the app/surface copy
    path without changing the embedded ABI.
  - When `Surface::copy_to_clipboard` formats `plain`, `vt`, `html`, or `mixed`,
    pass the configured map into terminal formatting so both `text/plain` and
    `text/html` payloads use the same replacement policy. This matches Ghostty's
    practical formatter behavior: the `codepoint_map` option is carried into
    plain and styled formatter output.
  - Do not apply this map to `copy_url_to_clipboard`, matching upstream's
    documentation.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, update Phase F operating notes and the
    `font-codepoint-map` + `clipboard-codepoint-map` roadmap checkbox if the
    field is fully represented and used by copy formatting.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/config/mod.rs roastty/src/terminal/page_list.rs roastty/src/terminal/terminal.rs roastty/src/lib.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/56-clipboard-codepoint-map.md`
- Run targeted tests:
  - `cargo test -p roastty clipboard_codepoint_map`
  - `cargo test -p roastty config_format_config`
  - `cargo test -p roastty terminal_formatter_codepoint_map`
  - `cargo test -p roastty surface_binding_action_copy_to_clipboard`
  - `cargo test -p roastty surface_binding_action_copy_url_to_clipboard`
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = `clipboard-codepoint-map` is represented on `Config`, parsed and
formatted through normal config entry points, and app copy-to-clipboard
formatting applies configured replacements while URL copying remains unchanged;
targeted and full tests pass.

**Partial** = the config field lands and terminal formatting can consume the
map, but the app copy path exposes a bounded lifetime/API issue that needs a
follow-up experiment; record the exact missing integration.

**Fail** = the current formatter/config boundaries cannot apply configured
clipboard replacements without first redesigning the formatter ABI or surface
config ownership.

## Design Review

**Reviewer:** Codex-native adversarial subagent Bohr
(`multi_agent_v1.spawn_agent`, fresh context, read-only). **Verdict: APPROVED.**

The reviewer returned no Required findings. It found one Optional issue: the
verification list did not explicitly prove that `copy_url_to_clipboard` remains
unchanged. The fix adds
`cargo test -p roastty surface_binding_action_copy_url_to_clipboard` to targeted
verification. It also noted that Ghostty's comments around VT/HTML copy are
misleading; the design now states the behavior we are matching, namely that the
formatter's `codepoint_map` option is carried into both plain and styled output.
