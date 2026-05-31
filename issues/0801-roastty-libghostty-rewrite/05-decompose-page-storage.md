# Experiment 5: Decompose Page Storage Port

## Description

Analyze Ghostty's terminal page storage stack and choose the next implementation
slice for Roastty.

Experiments 3 and 4 ported small foundations (`Tabstops` and `size`). The next
major area is `page.zig`, but it is not a single safe implementation step:
`page.zig` is large, pointer-heavy, layout-sensitive, and tightly coupled to
bitmap allocation, offset hash maps, styles, hyperlinks, graphemes, rows, cells,
and page-list behavior. This experiment should decompose that area before any
more code is ported.

The output should be a concrete page-storage roadmap: dependencies, risk
classification, test groups, unsafe boundaries, and the next implementation
experiment.

## Changes

1. Inspect the page-storage source set.
   - Use `vendor/ghostty/` as source of truth.
   - Inspect at least:
     - `vendor/ghostty/src/terminal/page.zig`
     - `vendor/ghostty/src/terminal/PageList.zig`
     - `vendor/ghostty/src/terminal/bitmap_allocator.zig`
     - `vendor/ghostty/src/terminal/hash_map.zig`
     - `vendor/ghostty/src/terminal/ref_counted_set.zig`
     - `vendor/ghostty/src/terminal/style.zig`
     - `vendor/ghostty/src/terminal/hyperlink.zig`
     - `vendor/ghostty/src/terminal/color.zig`
     - `vendor/ghostty/src/terminal/kitty.zig`
   - Do not modify `vendor/ghostty/`.

2. Build a direct import inventory.
   - For both `page.zig` and `PageList.zig`, enumerate every direct import and
     classify it as:
     - required for the next slice;
     - deferred;
     - replaced by Rust standard/library behavior;
     - omitted because Roastty is macOS-only or because the path is not needed.
   - The inventory must explicitly cover:
     - `terminal_options` / `slow_runtime_safety`;
     - Kitty graphics gates;
     - `fastmem` copy semantics;
     - `quirks.inlineAssert`;
     - `tripwire` failure hooks;
     - intrusive list dependencies;
     - `point`;
     - `highlight`;
     - PageList-only dependencies.

3. Classify `page.zig`.
   - Break it into coherent implementation areas:
     - page-aligned allocation and layout;
     - `Capacity`, `Size`, and layout calculations;
     - `Row` and `Cell` packed storage;
     - basic row/cell access;
     - grapheme allocation and lookup;
     - style set integration;
     - hyperlink set/map integration;
     - clone/cloneFrom/partial-row copy;
     - move/erase behavior;
     - integrity checking;
     - exact row capacity calculation.
   - For each area, record:
     - upstream functions/types;
     - dependencies;
     - unsafe requirement, if any;
     - tests that prove behavior;
     - whether it can be implemented now or requires a prerequisite port.

4. Classify dependency modules.
   - Determine which dependency should be ported before `Page` itself.
   - Evaluate at least:
     - `bitmap_allocator.zig`
     - `hash_map.zig`
     - `ref_counted_set.zig`
     - minimal `color`, `style`, `hyperlink`, and `kitty` types needed by `Page`
   - For each dependency, record:
     - whether it is required for the first useful `Page` slice;
     - whether it is safe Rust, unsafe Rust, or mixed;
     - upstream tests available;
     - expected implementation size.

5. Define the unsafe boundary for page storage.
   - Decide whether Roastty should keep Ghostty's contiguous page backing memory
     model for `Page`, or stage through safe Rust containers first.
   - If the contiguous model is required, record which modules own unsafe
     pointer arithmetic and which APIs remain safe.
   - The unsafe boundary plan must explicitly name:
     - ownership and deallocation invariants;
     - zeroed page allocation assumptions;
     - packed `Row` / `Cell` layout assertions;
     - aliasing rules for copied and moved cells;
     - which functions/modules may contain pointer arithmetic;
     - which functions/modules must remain safe wrappers over those internals.
   - Do not implement the unsafe boundary in this experiment.

6. Classify tests.
   - Assign every `page.zig` test to a named test group.
   - For PageList and dependency-module tests, mark each test group as:
     - required for the next implementation slice;
     - deferred until a later slice;
     - not applicable to Roastty with a reason.
   - The result must identify which tests prove the chosen next implementation
     slice and which tests will remain red/deferred after that slice.

7. Choose the next implementation experiment.
   - The result must name exactly one next implementation slice.
   - The next slice should be small enough to implement and review in one
     experiment.
   - It should make page storage more real, not just add unrelated terminal
     helpers.
   - It should include clear tests from upstream or direct equivalents.

8. Verify the diagnostic-only boundary.
   - Before recording the result, run:

     ```bash
     git status --short
     ```

   - Expected changed files are limited to Issue 801 documentation and
     gitignored review logs under `logs/`.
   - This experiment must not modify `roastty/`, `vendor/ghostty/`,
     `Cargo.toml`, `Cargo.lock`, scripts, build configuration, or source code.

9. Record the result.
   - Append `## Result` and `## Conclusion` to this file.
   - Include these tables:
     - `Page Storage Areas`
     - `Direct Import Inventory`
     - `Dependency Port Order`
     - `Unsafe Boundary Plan`
     - `Page Test Groups`
     - `Next Implementation Slice`
   - Update the Issue 801 README experiment index from `Designed` to `Pass`,
     `Partial`, or `Fail`.

## Verification

The experiment passes if:

- the result cites concrete upstream files and tests;
- each major `page.zig` behavior area is classified;
- required dependency modules are ordered;
- unsafe page-storage boundaries are explicit;
- the next implementation experiment is exactly one named slice;
- the diagnostic-only boundary is preserved;
- Codex reviews the completed result and approves it or all real findings are
  fixed.

The experiment is partial if:

- the broad decomposition is useful, but one dependency relationship remains too
  uncertain to choose the next implementation slice safely.

The experiment fails if:

- it starts porting code instead of decomposing the page-storage work;
- it leaves the unsafe boundary vague;
- it recommends a broad or multi-subsystem next implementation experiment;
- it ignores the upstream tests.

## Codex Review

This experiment design must be reviewed by Codex before implementation. Any real
design issues must be fixed before committing the plan or running the audit.
