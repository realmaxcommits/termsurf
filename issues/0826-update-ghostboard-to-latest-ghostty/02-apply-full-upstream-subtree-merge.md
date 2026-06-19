# Experiment 2: Apply the Full Upstream Subtree Merge

## Description

Apply the full upstream Ghostty range selected by Experiment 1 to `ghostboard/`
using a history-preserving subtree pull, then resolve the known conflicts into a
coherent working tree.

The selected range is:

```text
332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28..5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff
```

This experiment is the real merge implementation, not another dry run. Its goal
is to make the full upstream update present in `ghostboard/` with no unresolved
merge conflicts while preserving required TermSurf-specific behavior.

Build, launch, and runtime parity are separate gates. This experiment should not
expand into broad build debugging unless a minimal build-file conflict
resolution requires it.

## Changes

- `ghostboard/`
  - Run the history-preserving subtree pull:

```bash
git subtree pull --prefix=ghostboard ghostty \
  5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff \
  -m "Merge upstream Ghostty into ghostboard"
```

- Resolve the expected 17 conflicts from Experiment 1.
- Preserve upstream Ghostty changes by default.
- Preserve TermSurf-specific behavior where required for the app identity,
  protocol exports, socket/protobuf integration, browser overlay lifecycle,
  browser input forwarding, config path, CLI name, and poisoned-file cleanup.
- `issues/0826-update-ghostboard-to-latest-ghostty/README.md`
  - Link this experiment with status `Designed`, then update the status after
    the result is known.
- `issues/0826-update-ghostboard-to-latest-ghostty/02-apply-full-upstream-subtree-merge.md`
  - Record the plan, conflict-resolution result, verification, review, and
    conclusion.

Do not modify `webtui/` or `roamium/` in this experiment.

## Conflict Resolution Strategy

Resolve the known conflict groups as follows:

- `.agents/commands/gh-issue`
  - Delete this file. It is an upstream agent-helper command, upstream deleted
    it, and TermSurf has no current requirement to keep it inside `ghostboard/`.
- `.github/VOUCHED.td` and `.github/workflows/vouch-*`
  - Preserve TermSurf's poisoned-file cleanup. Do not reintroduce upstream
    poisoned/vouch files that were intentionally removed.
- `.github/workflows/test.yml`
  - Preserve any TermSurf-required CI behavior and adopt upstream path-filter
    improvements when they do not reintroduce poisoned/vouch behavior.
- `CONTRIBUTING.md`, `HACKING.md`, `README.md`
  - Keep the current TermSurf/Ghostboard-local versions as the baseline.
  - Do not import upstream agent instructions, contribution policy, or README
    text that conflicts with root `AGENTS.md`, TermSurf's frontend status, or
    poisoned-file cleanup.
  - Only retain upstream doc content if it is factual project/build information
    needed by the merged source tree, contains no active agent/developer
    instructions, and does not contradict TermSurf's current Ghostboard role.
- `build.zig`
  - Combine upstream `emit_lib_vt` behavior with TermSurf's existing build
    semantics.
  - Do not install the CLI merely because an executable is emitted.
- `include/ghostty.h`
  - Keep upstream `GHOSTTY_API` usage and new upstream declarations.
  - Preserve TermSurf protocol C exports.
- `TerminalController.swift`
  - Preserve TermSurf pane cleanup.
  - Preserve upstream pending-initial-presentation cleanup.
- `SurfaceView_AppKit.swift`
  - Preserve upstream copy/action behavior.
  - Preserve TermSurf copy-current-URL feedback and any TermSurf state needed by
    browser overlay behavior.
- `src/build/SharedDeps.zig`
  - Preserve TermSurf protobuf C sources.
  - Preserve upstream MSVC sanitizer handling for `stb.c`.
- `src/main_c.zig`
  - Preserve TermSurf exported protocol functions.
  - Preserve upstream Windows `DllMain` support.

If a conflict has an unexpected shape during the real merge, resolve it using
the same rule: upstream wins by default, TermSurf wins only for documented
TermSurf-specific behavior.

## Verification

Before implementation:

```bash
git status --short
git rev-parse ghostty/main
test "$(git rev-parse ghostty/main)" = \
  "5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff"
git rev-list --count 332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28..ghostty/main
```

During implementation:

```bash
git diff --name-only --diff-filter=U
rg -n '^(<<<<<<<|=======|>>>>>>>)' ghostboard
```

After resolving conflicts, before result review:

```bash
test -z "$(git diff --name-only --diff-filter=U)"
test "$(git rev-parse MERGE_HEAD)" = \
  "5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff"
! rg -n '^(<<<<<<<|=======|>>>>>>>)' ghostboard
zig fmt ghostboard/build.zig ghostboard/src/build/SharedDeps.zig ghostboard/src/main_c.zig
(cd ghostboard && swiftlint lint --strict --fix)
git diff --check
```

If additional conflicted Zig files are introduced by the merge, run `zig fmt` on
those files as well. If additional Swift files are modified by conflict
resolution, they are covered by the `swiftlint lint --strict --fix` command from
`ghostboard/AGENTS.md`.

Result recording and review:

1. Append `## Result` and `## Conclusion` to this file.
2. Record each resolved conflict group and the chosen resolution.
3. Update the issue README status for this experiment to `Pass`, `Partial`, or
   `Fail`.
4. Run Prettier on this experiment file and the issue README:

```bash
prettier --write --prose-wrap always --print-width 80 \
  issues/0826-update-ghostboard-to-latest-ghostty/README.md \
  issues/0826-update-ghostboard-to-latest-ghostty/02-apply-full-upstream-subtree-merge.md
```

5. Request the required result review before completing the merge/result commit.
6. Commit the reviewed merge result.
7. After the merge/result commit, verify that the commit preserves the upstream
   parent:

```bash
git rev-list --parents -n 1 HEAD
git rev-list --parents -n 1 HEAD | rg '5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff'
```

Because this experiment is a real merge, the final result commit may be the
merge commit created after conflict resolution and result review. Use the
standard subtree merge message rather than a poetic commit message for that
merge commit.

Pass criteria:

- The full selected upstream range is applied with history preserved.
- Before the result review, `MERGE_HEAD` resolves to
  `5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff`.
- After the result commit, `HEAD` has `5d0a82ba337368f5632ffa6ce4d7c558fa2de9ff`
  as a merge parent.
- No unmerged paths remain.
- No conflict markers remain under `ghostboard/`.
- The known 17 conflict files are resolved and documented.
- Required TermSurf-specific behavior is preserved in the resolved files.
- The result has been reviewed before the merge/result commit is completed.

Fail criteria:

- The real merge uses `git merge -X subtree`, copy-over replacement, or another
  non-history-preserving mechanism.
- The merge cannot be resolved into a coherent working tree.
- TermSurf-specific protocol, overlay, config, CLI, or poisoned-file cleanup
  behavior is knowingly dropped without a documented reason.
- The result attempts to absorb broad build, launch, or runtime parity work that
  should be handled by later experiments.

## Design Review

An adversarial Codex subagent reviewed the initial design with fresh context.

**Verdict:** Changes required.

Required findings and fixes:

- History preservation was not directly verified. Fixed by requiring
  `MERGE_HEAD` to equal the upstream target before result review, and requiring
  the final result commit's parents to include the upstream target.
- Swift formatting hygiene was omitted. Fixed by adding
  `(cd ghostboard && swiftlint lint --strict --fix)`, as required by
  `ghostboard/AGENTS.md`.
- Conflict strategy for `.agents/commands/gh-issue` and the doc files was too
  ambiguous. Fixed by explicitly deleting `.agents/commands/gh-issue`, keeping
  TermSurf/Ghostboard-local docs as the baseline, and allowing upstream doc
  content only when it is factual, non-instructional, and compatible with
  TermSurf's current Ghostboard direction.

The re-review approved the design with no required findings. The optional
suggestions were adopted by adding an explicit `ghostty/main` target assertion
and spelling out the Prettier command.
