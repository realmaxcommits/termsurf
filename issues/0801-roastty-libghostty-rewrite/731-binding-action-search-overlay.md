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

# Experiment 731: Binding Action Search Overlay

## Description

Experiment 730 completed the remaining small app/surface runtime forwarding gap
for `new_window`. The next compact upstream binding-action gap is the
parameterless search overlay controls:

- `start_search`
- `end_search`

Upstream Ghostty treats `start_search` as a surface-target runtime action with
an empty initial search needle. It does not start the search worker directly; it
notifies the app runtime with `.start_search` and `needle = ""` so the GUI can
open search UI lazily. `end_search` clears local search state if present, but
always sends `.end_search` to the app runtime so the GUI can hide stale search
UI.

Roastty does not yet expose full search state, search navigation, or selection
search through the binding-action parser. This experiment intentionally adds
only the parameterless overlay actions above. Full `search:<needle>`,
`search_selection`, `navigate_search:<direction>`, and in-terminal search state
remain out of scope for later experiments.

## Changes

- `roastty/include/roastty.h`
  - Add upstream-aligned action tags:
    - `ROASTTY_ACTION_START_SEARCH = 59`
    - `ROASTTY_ACTION_END_SEARCH = 60`
  - Document `ROASTTY_ACTION_START_SEARCH` storage:
    - `storage[0] = borrowed const char*` search needle valid only during
      `action_cb`.
    - For parameterless `start_search`, the needle is an empty C string.
  - Document `ROASTTY_ACTION_END_SEARCH` as zero-storage.

- `roastty/src/lib.rs`
  - Add matching Rust action constants.
  - Add a parsed binding-action variant, or equivalent handling, for
    `start_search` so the borrowed empty `CString` remains alive during the
    callback.
  - Extend `parse_binding_action` to accept parameterless `start_search` and
    `end_search`.
  - Reject `start_search:`, `start_search:needle`, `end_search:`, and
    `end_search:now`.
  - Forward `start_search` through the existing surface-target runtime callback
    with action tag `ROASTTY_ACTION_START_SEARCH`, `storage[0]` pointing to an
    empty C string, and all other storage slots zeroed.
  - Forward `end_search` through the existing surface-target runtime callback
    with action tag `ROASTTY_ACTION_END_SEARCH` and zeroed storage.
  - Return `false` for null surfaces, detached surfaces, missing callbacks, and
    false callbacks.

- `roastty/tests/abi_harness.c`
  - Assert the two new ABI action tags.
  - Add malformed parser rejection checks.
  - Add valid no-callback coverage returning `false`.

- Tests in `roastty/src/lib.rs`
  - Add action constant assertions.
  - Cover parser false paths for empty-colon and non-empty parameters.
  - Cover null, detached, and missing-callback cases returning `false`.
  - Cover surface-target forwarding for `start_search`, including action tag,
    target shape, a non-null empty needle string, and zeroed storage after
    `storage[0]`.
  - Cover surface-target forwarding for `end_search`, including action tag,
    target shape, and zeroed storage.
  - Cover callback result propagation.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty search_overlay -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 731 design and found one workflow blocker: the
design-review result had not yet been recorded in the experiment frontmatter,
this section, or the README tuple. This section and the `[review.design]`
frontmatter now record the review outcome, and the README tuple is
`Codex/Codex/-`.

The review found no technical design blockers. It approved limiting the scope to
parameterless overlay controls, using action tags `59` and `60`, documenting the
borrowed empty search needle lifetime for `start_search`, keeping `end_search`
zero-storage, rejecting parameterized forms, and covering runtime forwarding,
ABI tags, parser failures, and callback result propagation.

## Result

**Result:** Pass

Experiment 731 added parameterless search overlay binding actions:
`start_search` and `end_search`.

`start_search` now forwards to the runtime action callback with
`ROASTTY_TARGET_SURFACE`, `ROASTTY_ACTION_START_SEARCH`, and a borrowed empty
C-string needle in `storage[0]` that remains alive for the callback duration.
`end_search` forwards to the same surface-target runtime callback with
`ROASTTY_ACTION_END_SEARCH` and zeroed storage.

The parser rejects colon parameters for both actions, and null surfaces,
detached surfaces, missing callbacks, and false callback results return `false`.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty search_overlay -- --nocapture --test-threads=1`
  - 3 passed
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
  - 107 passed
- `cargo test -p roastty --test abi_harness`
  - 1 passed
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Roastty can now notify embedders about the two parameterless search overlay
controls using upstream-shaped runtime action tags and storage. This keeps the
small overlay-control path separate from larger future search work such as
`search:<needle>`, `search_selection`, `navigate_search`, and internal search
match state.

## Completion Review

Codex reviewed the completed Experiment 731 diff and found one workflow blocker:
the result was recorded, but completion-review provenance had not yet been added
to the experiment frontmatter, this section, or the README tuple. This section,
the `[review.result]` frontmatter, and the README tuple now record that review.

The review found no implementation blockers. It approved `start_search`
forwarding with surface target, tag `59`, and a borrowed empty C string that
stays alive for the synchronous callback; `end_search` forwarding with surface
target, tag `60`, and zeroed storage; parser rejection; callback false paths;
ABI constants; tests; and the verification record.
