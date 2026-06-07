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

# Experiment 802: zf Fuzzy Foundation

## Description

Port the core `zf` fuzzy-ranking dependency into Roastty as a self-contained
Rust helper module. Ghostty uses the Zig `zf` package for theme-list filtering,
and Issue 801 still marks the dependency replacement as not started.

The vendored `zf` library provides allocation-free ranking and highlighting over
a haystack and one or more query tokens. Its relevant library surface is:

- `rank` / `rankNeedle`
- `highlight` / `highlightNeedle`
- case-sensitive and case-insensitive matching
- plain matching versus path-aware filename matching
- strict path matching when a query token contains a path separator

This experiment should port the library foundation only. It should not add
Roastty CLI/list-theme UI, configuration plumbing, or theme search integration.

## Changes

- `roastty/src/zf.rs`
  - Add Rust equivalents for `RankOptions`, `RankNeedleOptions`, `rank`,
    `rank_needle`, `highlight`, and `highlight_needle`.
  - Port the helper behavior from `vendor/zf/src/zf/filter.zig`, including ASCII
    case-insensitive matching, filename-priority ranking, strict path matching,
    match-index highlighting, and multi-token sort/dedup.
  - Keep the implementation byte-oriented, matching upstream `zf`'s `[]const u8`
    behavior.
  - Add focused tests copied from the upstream library interface and filter
    tests, plus Ghostty's theme-list usage shape (`plain = true`, case
    insensitive tokens).
  - Add explicit rank-ordering tests that prove the scoring behavior, not only
    match/no-match:
    - filename matches outrank full-path fallback matches;
    - exact filename coverage improves score over partial filename coverage;
    - word-boundary starts score better than middle-of-word starts;
    - sequential matches score better than scattered matches;
    - strict path segment coverage prefers shorter matching path segments.
- `roastty/src/lib.rs`
  - Export the internal `zf` module.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - After implementation, update the `zf` dependency row from not started to
    partial: core ranking/highlighting exists, but CLI/list-theme integration is
    still open.

## Verification

- Inspect:
  - `vendor/zf/src/zf/zf.zig`
  - `vendor/zf/src/zf/filter.zig`
  - `vendor/ghostty/src/cli/list_themes.zig`
- Run:
  - `cargo fmt -p roastty`
  - `cargo test -p roastty zf -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/802-zf-fuzzy-foundation.md`
- Run:
  - `git diff --check`

The experiment passes if Roastty has a tested Rust replacement for the core `zf`
rank/highlight library behavior and the dependency row is updated without
claiming CLI/theme-list integration. It is Partial if only ranking or only
highlighting lands. It fails if the port cannot match upstream's byte-oriented
multi-token/path behavior.

## Design Review

Codex reviewed the design and found one blocking verification gap: copied
upstream tests mostly prove match/no-match and highlight indexes, but the plan
also claimed filename-priority ranking, strict-path ranking, and byte-oriented
multi-token/path scoring behavior. The review required focused rank-ordering
tests for filename priority, exact filename coverage, word-boundary starts,
sequential matches, and strict path segment coverage before implementation can
start. Those tests are now part of the planned changes, so the design requires
re-review before the plan commit. Codex re-reviewed the corrected design and
approved it with no blocking findings. The review approved the scope as a `zf`
helper-module foundation and noted one non-blocking implementation guard: only
describe the helper as allocation-free if the API preserves upstream's
caller-buffer highlight style; otherwise keep result wording to "byte-oriented
core ranking/highlighting."
