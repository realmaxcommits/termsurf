+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.result]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 838: Compose the prepared frame rebuild sequence

## Description

Feature work resumes here (the test suite is green again after Exps 829–837).
Experiments 815–828 built every adapter and driver of the renderer's frame
rebuild as independent, individually-tested pieces in
`roastty/src/renderer/frame_rebuild.rs`:

- adapters on `FrameTerminalSnapshot`: `collect`, `build_plan`,
  `row_format_input`, `text_overlay_input`, `cursor_uniform_input`;
- drivers on `FrameRebuildPlan`: `format_rows`, `draw_text_overlays`,
  `apply_rebuild_uniforms`, `refine_padding_extend_rows`,
  `apply_cursor_uniforms`.

Exp 828's conclusion names the next step exactly: "compose a single prepared
frame rebuild sequence that collects a snapshot, builds a plan, formats rows,
draws overlays, updates rebuild/cursor uniforms, refines padding extension rows,
and then **stops before Metal presentation or renderer-thread orchestration**."

This experiment adds that one composition entry point. It introduces **no new
rendering behavior** — every validation and mutation stays in the existing
drivers; the composition only sequences them and threads the snapshot-derived
and caller-supplied inputs, so a caller no longer has to hand-wire six calls in
the right order.

### Ordering (driven by uniform data dependencies)

1. `format_rows` — rebuilds dirty row contents (`Contents`, `SharedGrid`,
   `row_dirty`).
2. `draw_text_overlays` — cursor/preedit into `Contents`/`SharedGrid`.
3. `apply_rebuild_uniforms` — grid-size + **reset** padding-extend on full
   rebuild (`MetalUniforms`).
4. `refine_padding_extend_rows` — **refines** the padding-extend the previous
   step reset, so it must run **after** `apply_rebuild_uniforms`.
5. `apply_cursor_uniforms` — block-cursor uniform (independent of 3–4).

`present_metal_frame` and `apply_custom_shader_frame` are intentionally **not**
called — that is the renderer-thread orchestration the sequence stops before.

## Changes

`roastty/src/renderer/frame_rebuild.rs` (production code — the composition, plus
tests).

- Add a mutable-target bundle (so the signature stays readable):

  ```rust
  pub(crate) struct FramePreparedRebuildTargets<'a> {
      pub(crate) contents: &'a mut Contents,
      pub(crate) grid: &'a mut SharedGrid,
      pub(crate) row_dirty: &'a mut [bool],
      pub(crate) uniforms: &'a mut MetalUniforms,
  }
  ```

- Add a caller-supplied input bundle, mixing the snapshot-adapter inputs
  (827/828) with the two drivers whose inputs are not snapshot-derived (rebuild
  uniforms, padding extend):

  ```rust
  pub(crate) struct FramePreparedRebuildInput<'a> {
      pub(crate) row_format: FrameSnapshotRowFormatInput<'a>,
      pub(crate) text_overlay: FrameSnapshotTextOverlayInput,
      pub(crate) cursor_uniform: FrameSnapshotCursorUniformInput,
      pub(crate) rebuild_uniform: FrameRebuildUniformInput,
      pub(crate) padding_extend: FramePaddingExtendInput<'a>,
  }
  ```

- Add the composition on `FrameTerminalSnapshot`:

  ```rust
  pub(crate) fn rebuild_frame(
      &self,
      targets: FramePreparedRebuildTargets<'_>,
      input: FramePreparedRebuildInput<'_>,
  ) -> Result<FramePreparedRebuildApplication, FramePreparedRebuildError>
  ```

  which: builds the plan (`self.build_plan()?`), then calls the five drivers in
  the order above — passing `self.row_format_input(input.row_format)`,
  `self.text_overlay_input(input.text_overlay)`,
  `self.cursor_uniform_input(input.cursor_uniform)` for the snapshot-derived
  stages and `input.rebuild_uniform` / `input.padding_extend` for the other two
  — reborrowing `targets.contents`/`grid`/`uniforms` across the calls.

- Add `FramePreparedRebuildApplication` collecting each stage's existing
  application struct (`FrameRowRebuildApplication<FrameRowRenderError>`,
  `FrameTextOverlayApplication`, `FrameRebuildUniformApplication`,
  `FramePaddingExtendApplication`, `FrameCursorUniformApplication`).

- Add `FramePreparedRebuildError` — one variant per stage wrapping that stage's
  existing error (`Plan(FrameRebuildPlanError)`,
  `FormatRows(FrameRowFormatValidationError)`,
  `TextOverlays(FrameTextOverlayError)`,
  `RebuildUniforms(FrameRebuildUniformValidationError)`,
  `PaddingExtend(FramePaddingExtendValidationError)`,
  `CursorUniforms(FrameCursorUniformValidationError)`), with `From` impls so the
  body can use `?`. **Fail-fast:** the first failing stage returns its error and
  later stages do not run (the early stages' mutations have already landed in
  `targets`, exactly as if the caller had hand-sequenced them — the composition
  changes ordering ergonomics, not failure semantics).

No change to any driver/adapter, and no Metal-presentation or renderer-thread
wiring.

## Verification

Per the bounded-run convention (15-min cap, Central-stamped, single tracked
task, no poll-watcher); these are fast unit tests in `frame_rebuild.rs`:

- **Happy path:** a dirty-row snapshot drives a full sequence — assert the
  returned `FramePreparedRebuildApplication` reports the rows formatted, the
  overlay drawn, the rebuild/cursor uniforms applied, and padding rows refined,
  and that the order is correct (e.g. padding-extend reflects
  refine-after-reset, not reset-after-refine).
- **Equivalence:** the composed sequence produces the **same** `Contents`,
  `SharedGrid`, `MetalUniforms` mutations as calling the five drivers by hand in
  the same order (a golden side-by-side on identical inputs).
- **Fail-fast per stage:** inject a validation failure at each stage in turn and
  assert (a) the matching `FramePreparedRebuildError` variant is returned, and
  (b) later stages did not run (observable via the unmutated later-stage
  target).
- **Stops before presentation:** assert (by construction / no call) that
  `present_metal_frame` and `apply_custom_shader_frame` are not invoked.
- `cargo build -p roastty` — no warnings (production code).
  `cargo fmt -p roastty -- --check` — clean. The full suite via
  `scripts/bounded-run.sh` (default parallelism) stays green. No-ghostty grep on
  changed lines — clean. `git diff --check` — clean.

**Pass** = the new composition tests pass, the equivalence test shows identical
mutations, fail-fast works per stage, and the full suite stays green.
**Partial/Fail** = any composition test fails or the suite regresses.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Verified every type name, signature, error variant, the borrow plan,
and the ordering against `frame_rebuild.rs` and `metal/shaders.rs`.

**Verdict:** APPROVED, no Required/Optional/Nit findings. Confirmed: the
reset-then-refine dependency is real (`apply_rebuild_uniforms` calls
`reset_padding_extend`, `refine_padding_extend_rows` then refines), so step 3
before step 4 is correct; `apply_cursor_uniforms` touches disjoint uniform
fields (`cursor_pos`/`wide`/`color`), so it is genuinely order-independent and
safe last. All five input bundle types and the six error types match the drivers
exactly (`draw_text_overlays` → `FrameTextOverlayError`, correctly _not_ the
`...ValidationError`). The mixed-lifetime input bundle and the `&mut` reborrows
across the five sequential calls compile cleanly. "No new behavior" + fail-fast
is honest (composing with `?` is identical to hand-sequencing). Scope correctly
stops before `present_metal_frame` / `apply_custom_shader_frame`.

**Note on ordering vs Exp 828's prose:** 828's conclusion listed "rebuild/cursor
uniforms, refines padding extension rows" (cursor before padding); this design
reorders to rebuild → padding → cursor, justified by the data dependency. The
reviewer confirmed both orders are functionally identical because
`apply_cursor_uniforms` mutates uniform fields disjoint from padding-extend and
grid size — so the reorder is safe and better expresses the real dependency.

## Conclusion

_(to be written after the run)_
