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

# Experiment 698: Surface Quicklook ABI

## Description

Upstream Ghostty exposes two macOS Quicklook surface functions:
`ghostty_surface_quicklook_font` and `ghostty_surface_quicklook_word`. Roastty
does not expose Roastty-named equivalents yet, so the Swift/frontend side has no
Quicklook ABI to call.

The full upstream font function depends on the CoreText font stack and renderer
font grid. Roastty does not have that font subsystem yet. The word path is more
tractable: Roastty already stores the latest surface mouse position, has surface
cell geometry, can convert viewport coordinates into terminal grid refs, can
select a word from a grid ref, and can return selected text with the
`roastty_text_s` ownership and viewport metadata path added in Experiment 696.

This experiment adds the missing Quicklook ABI shape and implements the word
read path from the current surface mouse position. It returns null for
`roastty_surface_quicklook_font` until the CoreText/font-grid subsystem exists.

This does not implement CoreText font resolution, renderer font-grid access,
Quicklook UI presentation, configurable `selection-word-chars`, or frontend
integration beyond the ABI.

## Changes

- `roastty/include/roastty.h`
  - Add Roastty-named equivalents of the upstream macOS Quicklook exports:
    - `void* roastty_surface_quicklook_font(roastty_surface_t)`;
    - `bool roastty_surface_quicklook_word(roastty_surface_t, roastty_text_s*)`.
  - Keep `roastty_surface_free_text` as the owner-side free path for successful
    Quicklook word results.

- `roastty/src/lib.rs`
  - Add `roastty_surface_quicklook_font` returning null for all inputs, with
    tests documenting that the CoreText/font-grid path is not available yet.
  - Add `roastty_surface_quicklook_word` that:
    - validates null result pointers and writes empty text before attempting a
      read;
    - rejects null/detached surfaces, surfaces without a worker, missing mouse
      position, missing cell geometry, and invalid cursor coordinates;
    - converts the latest surface mouse position to a viewport cell using the
      existing surface mouse report geometry contract;
    - asks the attached worker terminal for a viewport grid ref at that cell;
    - selects the word at that grid ref using the terminal's current default
      word boundaries;
    - returns the word through the existing `try_surface_selection_text` path so
      allocation, ownership, free behavior, and viewport metadata match normal
      surface text reads.
  - Do not mutate the terminal's active selection.
  - Preserve the raw `roastty_text_s` ownership contract: callers free
    successful results with `roastty_surface_free_text`.

- `roastty/tests/abi_harness.c`
  - Add compile/link smoke coverage for the Quicklook prototypes, null/default
    cases, and freeing a result struct through `roastty_surface_free_text`.

- Tests in `roastty/src/lib.rs`
  - Cover null and no-worker/no-position failure cases.
  - Cover successful word reads from a surface worker using the current mouse
    position and explicit geometry.
  - Cover that Quicklook word reads preserve the active selection.
  - Cover viewport metadata and ownership/free reset behavior on successful
    Quicklook word results.
  - Cover `roastty_surface_quicklook_font` returning null for null and valid
    surfaces until the font subsystem exists.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty quicklook -- --nocapture`
- `cargo test -p roastty surface_read -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the staged Experiment 698 design and approved it with no blocking
findings. The review accepted the ABI shape against upstream Ghostty, the
documented null font placeholder until Roastty has the CoreText/font-grid path,
the word-selection path from surface mouse position and geometry, the use of the
existing surface text allocation/metadata contract, and the proposed C ABI and
Rust test coverage.

## Result

**Result:** Pass

Implemented the Roastty-named Quicklook surface ABI:

- `roastty_surface_quicklook_font` is exported and returns null for all inputs,
  documenting the missing CoreText/font-grid integration point.
- `roastty_surface_quicklook_word` is exported and reads the word under the
  current surface mouse position when the surface is attached, has a worker, has
  valid geometry, and the pointer is inside the viewport.
- Quicklook word reads convert the current mouse position through the same
  surface mouse geometry used by mouse reporting, obtain a viewport grid ref,
  select a word with terminal default boundaries, and return text through the
  existing surface text allocation and metadata path.
- The active terminal selection is not mutated.
- The C ABI harness now compiles and links the new prototypes and covers
  null/default failure cases plus result cleanup through
  `roastty_surface_free_text`.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty quicklook -- --nocapture`
- `cargo test -p roastty surface_read -- --nocapture`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Roastty now has the macOS Quicklook ABI foundation needed by the frontend for
word lookup. The implemented word path is useful and test-covered, while the
font path remains an explicit null placeholder until a later experiment connects
CoreText font-grid state and Quicklook presentation.

## Completion Review

Codex reviewed the staged completed Experiment 698 result. The review found no
implementation blockers: the ABI shape matches upstream, the null font
placeholder matches the documented foundation, word lookup uses stored surface
mouse position plus surface geometry, active selection is preserved, successful
results use the existing `roastty_text_s` allocation/metadata/free path, and the
C harness covers the new prototypes and null/default behavior.

The review initially blocked the result commit only because completion-review
provenance had not yet been recorded. This section, the `[review.result]`
frontmatter, and the README tuple update resolve that workflow finding.
