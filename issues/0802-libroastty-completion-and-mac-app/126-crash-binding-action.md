# Experiment 126: Phase G — crash binding action

## Description

Port the remaining upstream `crash` binding action into Roastty's configured
binding parser and surface action runtime.

Upstream `input.Binding.Action.crash` accepts one of three locations:
`crash:main`, `crash:io`, or `crash:render`. The action is explicitly a hard
crash used to test crash handling. In Ghostty, `main` panics immediately, `io`
queues a crash message to the surface IO thread, and `render` queues a crash
message to the renderer thread. Roastty does not yet have separate
renderer-thread crash plumbing, and panicking out of `extern "C"` entry points
is process-terminating, which is consistent with the action but unsuitable for
direct C ABI unit tests.

This experiment adds the action as a surface-scoped parsed action and implements
all three locations as intentional panics with location-specific messages in the
current Rust action path. That gives configured bindings, command-palette action
validation, canonical formatting, and Rust-internal dispatch the upstream action
surface now. Future renderer/IO thread parity can replace the `io` and `render`
panic bodies with thread mailbox crashes once those thread-specific crash
channels exist in Roastty.

## Changes

- `roastty/src/lib.rs`
  - Add a `CrashThread` enum or equivalent representation for `main`, `io`, and
    `render`.
  - Extend `ParsedBindingAction` with a `Crash(CrashThread)` variant.
  - Extend action parsing so:
    - `crash:main`, `crash:io`, and `crash:render` parse successfully;
    - bare `crash`, unknown values, empty parameters, and extra malformed forms
      are rejected.
  - Extend canonical formatting so parsed crash actions round-trip to
    `crash:main`, `crash:io`, or `crash:render`.
  - Treat crash as surface-scoped for app-key classification, matching upstream.
  - Dispatch crash by intentionally panicking with a location-specific message.
    This should be tested through internal Rust action paths, not through
    `extern "C"` ABI calls that would abort the process.
- `roastty/src/lib.rs` tests
  - Add parser/canonical tests for all valid crash locations and representative
    invalid forms.
  - Add internal runtime tests that call `perform_parsed_binding_action` inside
    `catch_unwind` for `main`, `io`, and `render`, proving each one panics with
    the expected location-specific message.
  - Add configured binding tests through the Rust internal `Surface::key` path,
    again using `catch_unwind`, to prove keybindings dispatch crash actions
    without using the C ABI boundary.
  - Add app-key focused/global classification coverage to prove crash remains
    surface-scoped: focused non-global app-key returns `false`, while a global
    crash binding fans out to the first live surface and panics.

Out of scope:

- Thread-specific IO and renderer crash mailboxes.
- C ABI tests that invoke `crash`, because unwinding through `extern "C"` aborts
  and the hard-crash behavior is intentional.
- Crash report persistence or Sentry integration.
- Native keymaps/global shortcut registration.
- Broader `all:` routing and full upstream default binding table completion.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/lib.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/126-crash-binding-action.md issues/0802-libroastty-completion-and-mac-app/README.md`
- Run targeted tests:
  - `cargo test -p roastty crash_binding`
  - `cargo test -p roastty app_key`
  - `cargo test -p roastty command_palette`
- Run full Roastty tests:
  - `cargo test -p roastty -- --test-threads=1`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run the same Prettier command with `--check`.

**Pass** = all three crash locations parse, canonicalize, and intentionally
panic through internal surface/configured binding paths; app-key classification
stays upstream-aligned; targeted plus full tests pass.

**Partial** = parser/canonical support lands, but thread-location runtime
behavior needs follow-up once IO/render crash channels exist.

**Fail** = crash support requires a larger runtime/thread redesign before it can
be safely represented in Roastty.

## Design Review

**Reviewer:** Codex-native adversarial reviewer, fresh context
(`multi_agent_v1.spawn_agent`, agent `019eb824-c5a6-7711-ac85-2098e7e2df6a`)

**Verdict:** Approved

**Findings:** None.

**Notes:** The reviewer confirmed that upstream defines `crash` with `main`,
`io`, and `render` locations, scopes it as a surface action, and keeps it out of
the embedded app-runtime action enum. The reviewer also noted that app-key
surface-scope coverage should be mandatory during implementation and that the
temporary all-location panic behavior is a bounded divergence until Roastty has
thread-specific IO/render crash channels.

## Result

**Result:** Pass

Roastty now parses, canonicalizes, and dispatches the upstream `crash` binding
action for all three locations: `crash:main`, `crash:io`, and `crash:render`.
The action is surface-scoped and remains outside the embedded app-runtime C
action enum.

The current runtime intentionally panics with location-specific messages in the
Rust action path for all three locations. This gives configured keybindings and
command-palette validation access to the action now. It is a bounded divergence
from upstream's thread-specific IO/render crash mailboxes, which Roastty can add
once those crash channels exist.

Verified behavior:

- valid `crash:{main,io,render}` actions canonicalize exactly;
- bare `crash`, empty crash parameters, unknown locations, and malformed extra
  parameters are rejected;
- internal surface runtime dispatch intentionally panics for all three locations
  with location-specific messages;
- configured surface keybindings dispatch crash actions through the internal
  Rust key path;
- focused non-global app-key leaves containing crash return `false` without
  dispatching;
- global app-key crash bindings fan out to live surfaces and panic through the
  internal Rust app-key path.

Verification run:

- `cargo fmt`
- `cargo test -p roastty crash_binding` — 5 passed
- `cargo test -p roastty app_key` — 27 passed
- `cargo test -p roastty command_palette` — 2 passed
- `cargo test -p roastty -- --test-threads=1` — 4721 unit tests passed, ABI
  harness passed with the existing 10 C enum-conversion warnings, doc tests
  passed
- `cargo fmt --check`
- `git diff --check`

Still out of scope:

- thread-specific IO and renderer crash mailboxes;
- C ABI tests that invoke `crash`, because the hard-crash action intentionally
  aborts if it unwinds through an `extern "C"` boundary;
- crash report persistence or Sentry integration;
- native keymaps/global shortcut registration;
- broader `all:` routing and full upstream default binding table completion.

## Conclusion

The remaining `crash` binding action is now represented in Roastty's parser,
canonical action surface, configured keybinding dispatch, app-key scoping, and
command-palette validation path. The only retained divergence is where the
intentional crash happens for `io` and `render`, pending thread-specific crash
mailboxes.

## Completion Review

Codex-native adversarial reviewer `019eb832-a8e0-7421-95a5-ac65eb01bede`
approved the completed experiment with no required findings. The reviewer
independently reran the focused `crash_binding`, `app_key`, and
`command_palette` tests plus `cargo fmt --check`, `git diff --check`, and the
Prettier check.
