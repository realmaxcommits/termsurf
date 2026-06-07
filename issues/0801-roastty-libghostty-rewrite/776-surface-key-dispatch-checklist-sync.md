+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 776: Surface Key Dispatch Checklist Sync

## Description

Audit and sync the Issue 801 C ABI checklist wording for surface key dispatch
and binding-action parsing.

The C ABI checklist still says the app/surface item is missing
`keybinding/action dispatch` and `full binding-action parsing`. Current code and
tests appear to have moved beyond that text: `roastty_surface_key` dispatches
configured and default bindings through `parse_binding_action` /
`perform_parsed_binding_action`, and `roastty_surface_binding_action` exposes
the same parser/executor through the public C ABI.

This experiment only updates issue documentation if verification confirms those
specific missing-work phrases are stale. It does not mark the whole app/surface
C ABI item complete, because frontend selection routing, split tree/frontend
mutations, and other surface lifecycle work are still listed as missing.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Remove or rewrite the stale `keybinding/action dispatch` and
    `full binding-action parsing` missing-work phrases only if current code and
    tests prove those areas are complete enough for the checklist.
  - Keep the app/surface C ABI item unchecked and preserve the remaining missing
    work that is still true.

## Verification

- Inspect `roastty/include/roastty.h` to confirm the public C ABI still exposes
  `roastty_surface_key`, `roastty_surface_key_is_binding`, and
  `roastty_surface_binding_action`.
- Inspect `roastty/src/lib.rs` to confirm:
  - `Surface::key` dispatches configured keybinds and static default keybinds;
  - `dispatch_configured_binding` and `dispatch_default_binding` route through
    `parse_binding_action` / `perform_parsed_binding_action`;
  - `roastty_surface_binding_action` uses the same parser/executor surface;
  - tests exist for configured dispatch, default dispatch, binding-action parser
    false paths, and supported action families.
- Run:
  - `cargo test -p roastty surface_key_default -- --nocapture --test-threads=1`
  - `cargo test -p roastty surface_key_configured -- --nocapture --test-threads=1`
  - `cargo test -p roastty surface_key_is_binding -- --nocapture --test-threads=1`
  - `cargo test -p roastty surface_binding_action_ -- --nocapture --test-threads=1`
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/776-surface-key-dispatch-checklist-sync.md`
  - `git diff --check`

The experiment passes if the README update is documentation-only, removes only
verified-stale missing-work text, leaves the app/surface C ABI item unchecked,
and all verification commands pass.

## Design Review

Codex reviewed the design and found no blockers. The review confirmed the scope
is narrow and documentation-only, the app/surface C ABI checklist item remains
unchecked, and the planned inspections and test filters are sufficient for
auditing the `keybinding/action dispatch` and `full binding-action parsing`
phrases.

The design is approved for the plan commit.

## Result

**Result:** Partial

The code and header inspections supported the checklist hypothesis:

- `roastty/include/roastty.h` exposes `roastty_surface_key`,
  `roastty_surface_key_is_binding`, and `roastty_surface_binding_action`.
- `Surface::key` dispatches configured keybinds and static default keybinds.
- `dispatch_configured_binding` and `dispatch_default_binding` route through
  `parse_binding_action` / `perform_parsed_binding_action`.
- `roastty_surface_binding_action` exposes the same parser/executor path through
  the public C ABI.

Three focused test filters passed:

- `cargo test -p roastty surface_key_default -- --nocapture --test-threads=1`
- `cargo test -p roastty surface_key_configured -- --nocapture --test-threads=1`
- `cargo test -p roastty surface_key_is_binding -- --nocapture --test-threads=1`

The broad proof command did not complete:

- `cargo test -p roastty surface_binding_action_ -- --nocapture --test-threads=1`

That run progressed through most of the binding-action tests, but spent several
minutes in the final PTY-backed file paste tests and was terminated manually.
Because the planned verification required that broad filter to pass, the README
checklist was not changed.

Documentation checks passed after recording the partial result:

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/776-surface-key-dispatch-checklist-sync.md`
- `git diff --check`

## Conclusion

The checklist text is likely stale, but Experiment 776 did not prove it under
its own verification plan. The next experiment should either investigate the
slow/hanging broad `surface_binding_action_` filter or use a narrower reviewed
verification strategy that exercises the parser/action families without relying
on the entire PTY-heavy filter as one command.

## Completion Review

Codex reviewed the completed Partial result and agreed that Partial is the
appropriate status because the broad `surface_binding_action_` proof command did
not complete. The review also confirmed that leaving the checklist unchanged was
correct because the planned update was conditional on full verification.

The initial completion review asked to record the post-result `prettier` and
`git diff --check` status. Those checks were recorded, and the follow-up review
found no blockers and approved the Experiment 776 result commit.
