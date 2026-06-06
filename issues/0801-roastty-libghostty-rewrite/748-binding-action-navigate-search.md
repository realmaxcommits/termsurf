+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 748: Binding Action Navigate Search

## Description

Port Ghostty's `navigate_search:next` and `navigate_search:previous` binding
action enough for Roastty's C ABI and default macOS key dispatch. Ghostty keeps
search navigation as a core surface binding action rather than an apprt action;
Roastty does not yet own an internal search worker, so this experiment exposes a
Roastty runtime callback action that carries the navigation direction.

This closes the unsupported Command-G and Shift-Command-G gap left by Experiment
747 while preserving the performable default-binding behavior: the key is a
binding for query purposes, but it falls through when the runtime callback
cannot perform the search navigation.

## Changes

- `roastty/include/roastty.h`
  - Add a Roastty extension action tag for navigate-search callback dispatch:
    `ROASTTY_ACTION_NAVIGATE_SEARCH = 1000`.
  - Document that `1000..` is reserved for Roastty-owned public action
    extensions that have no upstream Ghostty C action tag.
  - Add a small direction enum matching Ghostty's `NavigateSearch` enum order:
    `ROASTTY_NAVIGATE_SEARCH_PREVIOUS = 0` and
    `ROASTTY_NAVIGATE_SEARCH_NEXT = 1`.
  - Document `roastty_action_s.storage[0] = roastty_navigate_search_e` and
    `storage[1..7] = 0` for the new action.
- `roastty/src/lib.rs`
  - Add matching Rust constants for the new public C ABI values.
  - Parse `navigate_search:next` and `navigate_search:previous`.
  - Reject missing, empty, unknown, whitespace-padded, or extra-colon
    `navigate_search` parameters.
  - Return default config triggers for `navigate_search:next` and
    `navigate_search:previous` matching Ghostty's macOS defaults.
  - Change default Command-G and Shift-Command-G from query-only matches into
    performable dispatches for the new action.
  - Add focused Rust tests for parser forwarding, callback-result propagation,
    null/detached/no-callback false paths, config trigger lookup, default key
    query flags, and default key dispatch.
  - Cover performable fallback semantics for default Command-G and
    Shift-Command-G: callback `true` consumes and suppresses the matching
    release; callback `false` or a missing callback falls through to terminal
    encoding.
- `roastty/tests/abi_harness.c`
  - Assert the new C ABI constants.
  - Add C harness checks for parser rejection, parser forwarding, and default
    Command-G / Shift-Command-G dispatch through the callback action.
  - Assert callback target shape for forwarded actions:
    `ROASTTY_TARGET_SURFACE`, the triggering surface pointer, action tag
    `ROASTTY_ACTION_NAVIGATE_SEARCH`, the expected direction in `storage[0]`,
    and zeroed unused storage.

## Verification

- `cargo test -p roastty surface_binding_action_search -- --nocapture --test-threads=1`
- `cargo test -p roastty config_trigger -- --nocapture --test-threads=1`
- `cargo test -p roastty surface_key_default -- --nocapture --test-threads=1`
- `cargo test -p roastty surface_key_is_binding -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness -- --nocapture`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the initial Experiment 748 design and found real gaps: the
Roastty-only action tag needed an explicit extension range and numeric value,
the direction enum values needed to be stable, storage zeroing and callback
target shape needed to be required in tests, performable fallthrough false paths
needed to be explicit, and workflow metadata needed to be recorded before the
plan commit.

The design was updated to reserve `ROASTTY_ACTION_NAVIGATE_SEARCH = 1000` as a
Roastty extension tag, define previous/next direction values, require
`storage[1..7] = 0` assertions, cover surface-target callback shape, and cover
callback-true consumption plus callback-false/no-callback fallthrough for
Command-G and Shift-Command-G.

Codex re-reviewed the corrected design and approved it for the plan commit. The
review agreed that the runtime-callback approach is appropriate for Roastty's
current architecture because Ghostty's internal search worker is not present,
and confirmed that the design now preserves performable Command-G /
Shift-Command-G semantics.
