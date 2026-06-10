+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 37: Phase C — in-band size reports (DECSET 2048)

## Description

Mode **2048** (`DECSET ?2048`, in-band size reports) lets a program receive the
terminal's grid/pixel size **without** an `ioctl` — the terminal sends
`CSI 48 ; rows ; cols ; height_px ; width_px t` when 2048 is **enabled** and
again on every **resize**. roastty registers the mode (`modes.rs:160`) and has
the encoder (`size_report::Style::Mode2048`, `size_report.rs:43`) — but
`Style::Mode2048` is **never emitted**. Upstream `stream_handler.zig:751` emits
on enable (`if (enabled) … size_report = .mode_2048`) and on resize.

## Approach

Reuse the existing size-fetch (the `effects.size` callback) +
`size_report::encode`:

1. **`terminal.rs` (`TerminalStreamHandler`)** — extract
   `emit_size_report(&mut self, style)` from `size_report()` (the callback
   fetch + `encode` + `write_pty_response_bytes`); `size_report()` (the CSI
   14/16/18 t query path) calls it — unchanged behavior.
2. **emit-on-enable** — in `set_mode_basic`, after
   `self.modes.set(mode, enabled)`:
   `if enabled && matches!(mode, Mode::InBandSizeReports) { self.emit_size_report(Style::Mode2048); }`
   (mirrors upstream's enable emit).
3. **emit-on-resize** — add **`Terminal::report_in_band_size(&mut self)`**
   (force=false analogue):
   `if !self.modes.get(Mode::InBandSizeReports) { return; }` then fetch the size
   via `self.effects.size`
   - `encode(Style::Mode2048, size)` + write to `pty_response` + the `write_pty`
     callback (Terminal owns `modes`/`effects`/`pty_response`, like
     `report_color_scheme_change` in Exp 36).
4. **`lib.rs`** — `Surface::set_size`: detect a pixel-size change **before the
   store**
   (`changed = self.size.width_px != width || self.size.height_px != height`),
   store, run the existing (unconditional) `resize_pty`, then if `changed` call
   `report_in_band_size` via the worker. **Faithful divergence:** upstream emits
   on _every_ resize (even a grid-unchanged fluid resize); the guard emits on
   every pixel-size change too (matching the fluid-resize case) but suppresses
   an exact-same-pixel-size redundant `set_size` (which the app does not issue
   in practice) — so behavior matches upstream while avoiding redundant
   identical reports. The `changed`-before-store ordering is
   correctness-critical (read before `self.size.width_px = width`) and
   `resize_pty` stays unconditional — both pinned with a comment.

`Style::Mode2048` encode is already `\x1b[48;rows;cols;height;width t`. **Only
`libroastty`** (`lib.rs`

- `terminal.rs`). No app change.

## Verification

1. **Headless terminal-level tests** (deterministic — no worker, so
   `pty_response()` is stable):
   - **emit-on-enable** (mirror `lib.rs:34007`'s size-callback setup): set the
     `size` callback; write `\x1b[?2048h` → `pty_response` contains
     `\x1b[48;{rows};{cols};{h};{w}t` for the callback's size; write
     `\x1b[?2048l` → **no** report.
   - **`report_in_band_size`** (the resize emit): mode 2048 **off** → no report;
     **on** → the `Mode2048` report; no `size` callback → no report (graceful).
     Fails pre-fix (`Style::Mode2048` was never emitted), passes after.
2. **No regression:** the CSI 14/16/18 t query path (`size_report()` →
   `emit_size_report`) still emits the same reports — the existing
   `terminal_query_callbacks_abi_option_values_and_size_reports`
   (`lib.rs:34007`) still passes (the refactor only extracts the shared
   fetch+encode+write).
3. **Surface resize wiring** verified by the terminal-level
   `report_in_band_size` test (the `set_size`→worker→terminal emit is the same
   async-pty-drain pattern as Exp 36, so the deterministic proof is at the
   terminal level; the change-detect is a thin reviewed guard).
4. **No live confirmation needed** — a pty protocol emission, observable in the
   model. **Completes fully while the screen is locked.**
5. Faithful to upstream `stream_handler.zig:751` (enable) + the resize report.

**Pass** = `Mode2048` is emitted on `?2048h` enable and on a resize
(mode-gated), the query path is unchanged, the headless tests pass, and the
suite is green. Fully headless — no Partial-pending-live.

**Partial** = enable works but the resize wiring needs more (documented).

**Fail** = `Mode2048` can't be emitted from the enable/resize sites
(documented).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED.** Verified against code + vendored upstream:
the **gap is real** (`Style::Mode2048` is produced only by the ABI shim + unit
tests; the live query path never returns it); the **encode is byte-identical**
to `size_report.zig:47-53`; the **enable trigger routes correctly** (`?2048h` →
`Action::SetMode{InBandSizeReports}` → `set_mode_basic`; not an alt-screen mode
→ clean post-`modes.set` emit point; `?2048l`/`enabled=false` correctly
suppresses, matching upstream `if (enabled)`); the **resize trigger mirrors the
accepted Exp-36 pattern exactly** (`Terminal` owns `effects.size`/`modes`/
`pty_response`; the two sequential `&self.termio_worker` borrows don't
conflict); **size is not stale** (set_size stores px first, then reports via the
same `effects.size` callback the CSI 18t query uses; upstream likewise reports
the post-resize size); **tests non-vacuous** (both fail pre-fix, exercise the
gate; the `emit_size_report` extraction preserves the CSI 14/16/18t query
asserts). Two Optional folded in: (1) the pixel-change guard is a **faithful
divergence** (upstream emits on every resize incl. grid-unchanged fluid resize;
the guard matches that on pixel change but skips an exact-same-px redundant
`set_size`, which the app doesn't issue) — documented; (2) pin the
`changed`-before-store ordering + unconditional `resize_pty` with a comment. Nit
(no fix): the fetch+encode appears in both `emit_size_report` (StreamHandler)
and `report_in_band_size` (Terminal) — inherent to the borrow boundary,
mirroring the Exp-36 query/change split.

## Result

_(to be added after the run.)_

## Conclusion

_(to be added after the run.)_
