+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"

[review.result]
agent = "codex"
+++

# Experiment 55: Phase F — SharedGridSet config assembly

## Description

Experiment 54 made the first font config surface representable in Roastty:
font-family fields, `font-size`, `font-synthetic-style`, and
`font-codepoint-map`. The live renderer still builds its font grid from a
hardcoded Menlo collection in `roastty/src/lib.rs`, while
`roastty/src/font/shared_grid_set.rs` is only a generic refcounted cache.

This experiment ports the next dependency slice from Ghostty's
`font/SharedGridSet.zig`: a config-derived font key and collection builder that
can construct a `SharedGrid` from the represented config fields. It should
replace the live renderer's hardcoded Menlo collection with the config-derived
path, while preserving the prior Retina sizing, discovery fallback, CJK/emoji
behavior, and font-size actions.

This is not the full final font subsystem. It intentionally defers font
variation config, metric modifiers, freetype flags, embedded fallback fonts, and
theme/config completeness fields that are not represented yet.

## Changes

- `roastty/src/font/shared_grid_set.rs`
  - Replace the generic `SharedGridSet<K>` surface with a concrete font-grid set
    that owns `Key`-derived cached grids, or add a concrete config-derived layer
    on top of the existing generic cache if that produces the smaller local
    change.
  - Add a `DerivedConfig` snapshot containing the represented fields from Exp
    54:
    - `font-family`
    - `font-family-bold`
    - `font-family-italic`
    - `font-family-bold-italic`
    - `font-style`
    - `font-style-bold`
    - `font-style-italic`
    - `font-style-bold-italic`
    - `font-codepoint-map`
    - `font-synthetic-style`
  - Add a `Key` that builds style-ordered `font::discovery::Descriptor`s from a
    `DerivedConfig` and a requested physical point size. Match upstream's
    descriptor rules:
    - regular descriptors use `font-style` as an exact style name when present;
    - bold/italic/bold-italic descriptors use the corresponding exact
      `font-style*` name when present;
    - when a styled exact style is not present, set the descriptor's bold and/or
      italic search bits for that style;
    - style offsets preserve upstream `regular`, `bold`, `italic`, `bold_italic`
      ordering;
    - `font-codepoint-map` is cloned into the key so resolver overrides can live
      as long as the grid.
  - Add stable hash/equality behavior for `Key` that includes physical font
    size, descriptor order/content, and the codepoint map. Omit metric modifiers
    and freetype flags until those configs exist in Roastty.
  - Build a `Collection` from the key:
    - discover configured primary font families per style and add deferred faces
      in upstream style order;
    - call `complete_styles` with `font-synthetic-style`;
    - preserve the current Menlo fallback behavior when no configured font
      family is available, so existing app behavior does not regress before
      embedded fallback fonts are ported;
    - keep Apple Color Emoji / CoreText discovery fallback behavior for emoji
      and CJK where the current live renderer already succeeds;
    - set point size to the physical font size before capturing metrics, as Exp
      29 made load-bearing for CJK wide-pitch behavior.
  - Add focused tests for deterministic key hashing, style offsets, descriptor
    construction, codepoint-map participation in key identity, cache ref/deref
    reuse, style-disable/synthetic-style propagation, and building a usable
    `SharedGrid` for the default config.
- `roastty/src/lib.rs`
  - Retain a parsed config snapshot in `App` (and update it on
    `roastty_app_update_config`) so surfaces can build renderers from the app's
    current config, not just `font_size`.
  - Pass that config snapshot into `build_live_renderer`.
  - Replace the hardcoded Menlo `Collection` construction with the
    config-derived `SharedGridSet` / `DerivedConfig` path.
  - Preserve font-size actions by continuing to pass the surface's current
    physical point size into the grid key; changing font size must still rebuild
    the renderer and produce a distinct grid key.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, update the Phase F roadmap and operating notes with
    the durable config→font assembly facts.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/font/shared_grid_set.rs roastty/src/lib.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/55-shared-grid-config-assembly.md`
- Run targeted tests:
  - `cargo test -p roastty shared_grid_set`
  - `cargo test -p roastty config_font`
  - `cargo test -p roastty codepoint_override`
  - `cargo test -p roastty surface_binding_action_font_size`
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run the live A/B smoke because this experiment replaces the live renderer's
  font-grid construction path:
  - `scripts/roastty-app/live-ab-smoke.sh --max-mismatch-ratio 1 --max-mean-channel-delta 255`
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = The live renderer builds its font grid through the config-derived
`SharedGridSet` path instead of a hardcoded Menlo-only collection; default
behavior remains working; configured font-family/codepoint-map/synthetic-style
inputs affect the generated key/resolver; targeted and full tests pass; and any
live A/B smoke has no new regression.

**Partial** = `DerivedConfig`/`Key` and collection construction land and are
tested, but the live renderer integration exposes a bounded blocker that needs a
separate experiment; record the exact gap and keep the hardcoded path if
necessary.

**Fail** = the current font/config abstractions cannot build a config-derived
grid without first porting larger missing prerequisites such as variation
config, metric modifiers, or embedded fallback fonts.

## Design Review

**Reviewer:** Codex-native adversarial subagent Helmholtz
(`multi_agent_v1.spawn_agent`, fresh context, read-only). **Initial verdict:
CHANGES REQUIRED.**

The reviewer found one Required issue: the design made live-renderer regression
proof optional even though the experiment replaces the live renderer's font-grid
construction path. The fix changed Verification so
`scripts/roastty-app/live-ab-smoke.sh --max-mismatch-ratio 1 --max-mean-channel-delta 255`
is mandatory, and changed the Pass criteria to require that live A/B smoke to
complete with no new regression.

**Re-reviewer:** Codex-native adversarial subagent Harvey
(`multi_agent_v1.spawn_agent`, fresh context, read-only). **Final verdict:
APPROVED.**

The re-review returned no findings and confirmed the required finding was fixed.

## Result

**Result:** Pass

The live renderer now builds its `SharedGrid` through
`font::shared_grid_set::build_grid_from_config`, using a config-derived
`DerivedConfig`/`Key` instead of constructing a hardcoded Menlo collection in
`roastty/src/lib.rs`.

Implemented:

- `DerivedConfig` snapshots the represented font-family, font-style,
  font-codepoint-map, and font-synthetic-style config fields.
- `Key` builds style-ordered discovery descriptors for regular, bold, italic,
  and bold-italic faces, preserves the codepoint map, and hashes/equates
  descriptor content, style offsets, physical point size, and codepoint-map
  entries.
- The collection builder discovers configured primary descriptors by style,
  completes missing styles through `font-synthetic-style`, keeps the temporary
  Menlo default-primary fallback when no configured primary is available, keeps
  Apple Color Emoji as the current explicit emoji fallback, enables discovery
  fallback for broader coverage, and attaches configured codepoint overrides to
  the resolver.
- `App` stores the parsed config snapshot, updates it through
  `roastty_app_update_config`, and invalidates live renderers after config
  updates so the next presentation rebuilds through the new config-derived path.
- `roastty_config_finalize` finalizes the parsed config before syncing the C ABI
  snapshot, so inherited font-family fields are visible to app-created configs.

Verification:

- `cargo fmt -- roastty/src/font/shared_grid_set.rs roastty/src/lib.rs roastty/src/config/mod.rs roastty/src/font/codepoint_map.rs`
  passed.
- `cargo test -p roastty shared_grid_set` passed: 9 tests.
- `cargo test -p roastty config_font` passed: 4 tests.
- `cargo test -p roastty codepoint_override` passed: 4 tests.
- `cargo test -p roastty surface_binding_action_font_size` passed: 4 tests.
- `cargo test -p roastty` passed: 4456 unit tests, 1 ABI harness integration
  test, and 0 doc-tests. The ABI harness still emits its pre-existing enum-cast
  warnings, but links and passes.
- `scripts/roastty-app/live-ab-smoke.sh --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  passed. Content-region metrics: `mismatch_ratio=0.00804861111111111`,
  `mean_channel_delta=0.6435878472222222`, `mismatched_pixels=11590`,
  `compared_pixels=1440000`; full-window metrics:
  `mismatch_ratio=0.06265130537974684`, `mean_channel_delta=1.6896117236946202`.
- `git diff --check` passed.

## Conclusion

The first config-to-font assembly path is now live. Roastty can build a usable
font grid from the represented Phase-F font config fields, and the live app
still passes the permissive Ghostty/Roastty A/B smoke after replacing the old
hardcoded renderer font path.

This remains a slice, not the final font subsystem. Font variations, metric
modifiers, freetype flags, embedded fallback fonts, broader fallback ordering,
and the remaining unrepresented config fields are still deferred to later
experiments.

## Completion Review

**Reviewer:** Codex-native adversarial subagent Poincare
(`multi_agent_v1.spawn_agent`, fresh context, read-only). **Final verdict:
APPROVED.**

The reviewer returned no findings.
