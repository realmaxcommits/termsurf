+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 641: Search And StringMap Audit

## Description

Audit the Issue 801 terminal checklist line for scrollback `search` and
`StringMap`.

The README currently says scrollback search plus `StringMap` are missing and
need `oniguruma`. Current code suggests that line is stale: `StringMap` exists
using Rust's `regex` byte engine in place of Oniguruma, and the search subsystem
contains `SlidingWindow`, active/page-list/screen/viewport searchers, the
multi-screen `Search` aggregator, and a std-concurrency adaptation of upstream's
search `Thread`.

This experiment should verify that evidence against vendored Ghostty and update
the checklist wording only if the current state supports it. It is intended as a
documentation-only audit unless the verification uncovers a small missing test
that should be added immediately.

## Audit Targets

1. `vendor/ghostty/src/terminal/StringMap.zig` vs.
   `roastty/src/terminal/string_map.rs` and `roastty/src/terminal/screen.rs`:
   - per-byte string-to-pin mapping;
   - search iterator match-to-selection conversion;
   - URL-like regex matching;
   - multibyte byte-map invariants;
   - Rust `regex` substitution for Oniguruma and the removed retry-budget path.
2. `vendor/ghostty/src/terminal/search.zig` and
   `vendor/ghostty/src/terminal/search/*.zig` vs.
   `roastty/src/terminal/search/*.rs`:
   - `SlidingWindow` page encoding, forward/reverse matching, overlap handling,
     and highlight construction;
   - active, page-list, screen, and viewport searchers;
   - selected-match indexing, next/prev selection, search-all, feed/tick, and
     pruning behavior;
   - `Search` multi-screen aggregator and search `Thread` message handling /
     spawn loop.
3. Surface/app boundaries:
   - confirm that any remaining search work belongs to Surface/App integration
     or UI event plumbing, not the terminal-core checklist line.

## Changes

1. Update `issues/0801-roastty-libghostty-rewrite/README.md`:
   - if verification supports it, mark the terminal-core search/StringMap line
     complete and mention the Rust `regex` substitution;
   - otherwise refine the open item to name the specific missing terminal-core
     behavior.
2. If the audit uncovers a small missing test that should be added immediately,
   update the relevant `roastty/src/terminal/*.rs` or
   `roastty/src/terminal/search/*.rs` test module.
3. Update this experiment file with the result and review records.

## Verification

- `cargo test -p roastty terminal::string_map`
- `cargo test -p roastty terminal::search`
- `cargo test -p roastty page_list::tests::search_encode`
- `cargo test -p roastty terminal::search::thread::tests::spawn`
- `cargo test -p roastty terminal::search::thread::tests::select`
- compare/read audited Rust files against:
  - `vendor/ghostty/src/terminal/StringMap.zig`
  - `vendor/ghostty/src/terminal/search.zig`
  - `vendor/ghostty/src/terminal/search/sliding_window.zig`
  - `vendor/ghostty/src/terminal/search/active.zig`
  - `vendor/ghostty/src/terminal/search/pagelist.zig`
  - `vendor/ghostty/src/terminal/search/screen.zig`
  - `vendor/ghostty/src/terminal/search/viewport.zig`
  - `vendor/ghostty/src/terminal/search/Thread.zig`
  - `vendor/ghostty/src/Surface.zig` search integration sections
- `cargo fmt -p roastty` if Rust tests are added
- `cargo fmt -p roastty -- --check`
- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/641-search-stringmap-audit.md`
- `git diff --name-only` shows only issue docs unless the audit uncovers a small
  missing test
- `git diff --check`

Pass = the checklist accurately reflects the current terminal-core search and
StringMap state, completed items are checked only with direct test evidence, and
any remaining Surface/App integration work is not mislabeled as missing
terminal-core behavior.

Fail = the audit relies on vague coverage, marks unverified search-thread or
StringMap behavior complete, or discovers a behavioral gap that needs a
dedicated implementation experiment before the checklist can be closed.

## Design Review

Codex design review session `019e9a9a-ee48-7ec2-bb17-ea152a97b42d` approved the
design with no blocking findings. The reviewer confirmed that the upstream/local
audit scope is coherent and that the verification commands match real test
modules or prefixes. The only nit was for the result to state explicitly that
Rust `regex` is a deliberate terminal-core substitution for Oniguruma, not
evidence that the broader `oniguruma` dependency is implemented, and to keep
Surface link/search UI plumbing classified outside terminal core unless direct
local evidence says otherwise.
