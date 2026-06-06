+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 704: Binding Action Text

## Description

Experiments 702 and 703 added `roastty_surface_binding_action` support for split
actions and `close_surface`. Upstream Ghostty also supports terminal input
binding actions:

- `text:<string>`;
- `csi:<string>`;
- `esc:<string>`.

The `text` action is the best next slice because Roastty already has:

- `roastty_surface_text`;
- active termio worker write plumbing;
- a port of upstream `config.string.codepointIterator`;
- child-PTY tests proving surface text reaches the process.

Upstream `performBindingAction` parses `text` action parameters with
`config.string.parse`, which accepts unquoted Zig string-literal escape
sequences such as `\n`, `\t`, `\\`, `\x15`, and `\u{2502}`. If parsing fails,
upstream logs the failure and still returns `true`, treating the parsed binding
action as consumed even though no bytes were queued.

This experiment adds `text:` support to the binding-action foundation:

- parse `text:<string>` as a `Binding.Action` string parameter;
- decode unquoted Zig string-literal escapes into UTF-8 bytes;
- dispatch decoded bytes through a raw termio write path, not
  `roastty_surface_text` / `Surface::text`, because upstream binding actions
  queue raw write requests and do not apply paste encoding or control-byte
  sanitization;
- return `true` for attached surfaces after a parsed `text:` action, including
  invalid string-literal escapes and empty decoded text, matching upstream's
  action-consumed behavior;
- return `false` only for null/detached surfaces or malformed action syntax
  before a valid `text` action exists.

This does not implement `csi`, `esc`, cursor-key actions, clipboard actions,
terminal reset, full keybind storage/lookup, or app-scoped actions.

## Changes

- `roastty/src/config/string.rs`
  - Add the upstream byte-array parse equivalent for unquoted string literals:
    copy non-escape bytes as-is, decode escape sequences into UTF-8 bytes, and
    return `InvalidString` only on malformed escapes or invalid Unicode
    codepoints.
  - Keep the existing codepoint iterator and quoted-string parser behavior
    unchanged.

- `roastty/src/lib.rs`
  - Extend the internal parsed binding-action enum with a `Text` variant.
  - Change binding-action parsing to operate on byte slices so `text:`
    parameters can contain arbitrary literal bytes. Split and close action names
    remain ASCII byte matches.
  - Extend `parse_binding_action` to accept `text:<bytes>` while rejecting
    missing `text` parameters.
  - Decode the text parameter with the new config string parser during dispatch.
  - Add/use a raw surface input helper that writes decoded bytes directly to the
    active termio worker without bracketed-paste wrapping or control-byte
    sanitization.
  - Return `true` after a parsed `text:` action for attached surfaces, even when
    decoding fails, the decoded byte slice is empty, or no termio worker exists.
  - Keep split and `close_surface` binding-action semantics unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage that `text` without a parameter is rejected and a
    `text:` binding action can be invoked through the public ABI.

- Tests in `roastty/src/config/string.rs`
  - Cover unquoted parse cases matching upstream examples:
    - empty string;
    - plain ASCII;
    - `\n`;
    - `\x15`;
    - `\u{2502}`;
    - literal non-UTF-8 bytes copied through unchanged;
    - malformed escapes and invalid Unicode escapes.

- Tests in `roastty/src/lib.rs`
  - Cover `text` without a parameter returning false.
  - Cover `text:` returning true for attached surfaces and no-oping successfully
    when no worker exists.
  - Cover invalid text escapes returning true without writing to the child.
  - Cover decoded escaped text reaching a child PTY through the binding-action
    ABI.
  - Cover raw control bytes, such as `text:\x15`, reaching the child unchanged
    rather than being replaced by paste sanitization.
  - Cover null/detached surfaces returning false.
  - Re-run existing binding-action tests to prove split and close semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty string -- --nocapture`
- `cargo test -p roastty binding_action -- --nocapture`
- `cargo test -p roastty surface_text -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the initial Experiment 704 design and blocked the plan commit
until two technical mismatches were fixed:

- `text:` binding actions must write raw decoded bytes to the termio worker.
  They cannot dispatch through `Surface::text`, because `Surface::text` is a
  paste-oriented path that applies bracketed paste encoding and control-byte
  sanitization, while upstream queues decoded `text:` bytes directly.
- Binding-action parsing must be byte-oriented for `text:` parameters. Upstream
  `config.string.parse` copies literal non-escape bytes directly without UTF-8
  validation; only escape sequences are interpreted.

This revised design fixes those findings by using byte-slice action parsing, a
raw surface input helper, a byte-array config string parser that preserves
literal bytes, and explicit tests for raw control-byte delivery. The review also
required recording this section and updating the README provenance tuple before
the plan commit.

## Result

**Result:** Pass

Implemented `text:` support in the binding-action ABI:

- Added `config::string::parse_string_literal`, the byte-array parser equivalent
  of upstream `config.string.parse`.
- Made the config string module available to the crate so binding-action
  dispatch can reuse the parser.
- Changed binding-action parsing to operate on byte slices, preserving arbitrary
  literal bytes in `text:` parameters.
- Kept split and close action names as ASCII byte matches.
- Added a raw surface text helper that writes directly to the active termio
  worker without paste encoding or control-byte sanitization.
- Extended parsed binding actions with `Text`.
- Made parsed `text:` actions return `true` for attached surfaces even when the
  decoded text is empty, invalid, or no termio worker exists.
- Kept null and detached surfaces returning `false`.
- Added parser tests for empty/plain/escaped/unicode/literal non-UTF-8 bytes and
  malformed escapes.
- Added Rust binding-action tests for no-worker behavior, detached behavior,
  decoded text reaching a child PTY, raw `\x15` delivery, invalid escape
  consumption without writing, and literal non-UTF-8 action parameters.
- Added C ABI harness smoke coverage for `text` and `text:hello`.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty string -- --nocapture`
- `cargo test -p roastty binding_action -- --nocapture`
- `cargo test -p roastty surface_text -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Roastty binding-action invocation now supports split actions, `close_surface`,
and raw `text:` input with upstream-style unquoted string-literal escape
parsing. Remaining binding-action parity still requires `csi`, `esc`, cursor-key
actions, clipboard actions, terminal reset and scrolling actions, complete
keybind storage/lookup, app-scoped actions, and frontend split/tab/window
mutation.

One upstream detail remains deferred for this slice: Ghostty scrolls to bottom
after queueing a `text:` binding-action write. Roastty's raw write path
currently queues the bytes only; the scroll-to-bottom side effect should be
handled with the later terminal scrolling / renderer state integration.

## Completion Review

Codex reviewed the staged completed Experiment 704 result. The review found no
implementation blockers: binding-action parsing is byte-oriented, `text:`
preserves literal non-UTF-8 bytes, escape decoding matches upstream's
byte-copy-plus-UTF-8 escape behavior, dispatch bypasses `Surface::text` paste
encoding, and the raw `\x15` PTY test proves control-byte delivery.

The review accepted the parser, binding-action, surface text, and C harness
coverage for this slice. It initially blocked the result commit only because the
README provenance tuple still showed the result review as pending. This section
and the README tuple update resolve that workflow finding. The review also noted
the stale `config::string` module comment and the deferred upstream
scroll-to-bottom side effect; both are recorded in this result.
