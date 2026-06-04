+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"
+++

# Experiment 397: the link underline override

## Description

A cell that is part of a **hovered link** is underlined to signal it is
clickable. Upstream overrides the cell's underline: a link cell gets a
**single** underline, unless it **already** has a single underline — in which
case it gets a **double** underline, to distinguish the link from the cell's own
underline. A non-link cell keeps its SGR underline. This experiment ports that
override as `link_underline`, a pure function of `(is_link, underline)`. The
link-membership source (which cells are hovered links) is not yet modeled in
roastty, so `is_link` is a parameter and the integration into the underline pass
is deferred — this experiment is the override itself, unit-tested (mirroring the
highlight pattern, Experiment 390).

## Upstream behavior

In `rebuildCells` (`renderer/generic.zig`), the effective underline:

```zig
// Give links a single underline, unless they already have an underline, in
// which case use a double underline to distinguish them.
const underline: terminal.Attribute.Underline = underline: {
    if (links.contains(.{ .x = @intCast(x), .y = @intCast(y) })) {
        break :underline if (style.flags.underline == .single)
            .double
        else
            .single;
    }
    break :underline style.flags.underline;
};
// …then addUnderline(x, y, underline, …) if underline != .none
```

So for a cell in the hovered-link set: if its SGR underline is `.single` → use
`.double`; otherwise (`.none`/`.double`/`.curly`/`.dotted`/`.dashed`) → use
`.single`. A non-link cell uses its SGR underline unchanged. The resulting
underline then feeds the underline decoration (only drawn when `!= .none`).

## Rust mapping (`roastty/src/renderer/cell.rs`)

```rust
/// The effective underline for a cell, applying the hovered-link override: a link
/// cell gets a single underline, unless it already has a **single** underline, in
/// which case it gets a **double** underline to distinguish the link from the
/// cell's own underline. A non-link cell keeps its SGR `underline`. Faithful port
/// of upstream's link underline logic. The link-membership source is deferred, so
/// `is_link` is supplied by the caller.
fn link_underline(is_link: bool, underline: Underline) -> Underline {
    if !is_link {
        return underline;
    }
    if matches!(underline, Underline::Single) {
        Underline::Double
    } else {
        Underline::Single
    }
}
```

`Underline` (`terminal::sgr::Underline`: `None`/`Single`/`Double`/`Curly`/
`Dotted`/`Dashed`) is already imported. The override returns the effective
underline; the underline pass (deferred) computes `is_link` from the
hovered-link set and feeds `link_underline(is_link, flags.underline)` to
`add_underline`.

## Scope / faithfulness notes

- **Ported (bridged)**: the hovered-link underline override (upstream's link
  underline `switch`) as `link_underline` — a link cell's single underline
  becomes double, any other underline (incl. none) becomes single, a non-link
  cell is unchanged.
- **Faithful**: `is_link && Single → Double`;
  `is_link && (None/Double/Curly/ Dotted/Dashed) → Single`;
  `!is_link → underline` — upstream's exact logic
  (`underline == .single ? .double : .single` for a link, else the SGR
  underline). The `None → Single` link case matches upstream's "give links a
  single underline".
- **Faithful adaptation**: `is_link` is a `bool` parameter (upstream's
  `links.contains({x, y})`); roastty has no hovered-link/`Set` state yet, so the
  membership source and the wiring into the underline pass are deferred (the
  same shape as the search highlights, Experiment 390, before plumbing).
- **Deferred**: the hovered-link set (the OSC 8 / regex link membership) and
  wiring `link_underline` into `rebuild_row`'s underline pass; the
  column-ordered decoration merge; the Metal upload. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`: add the `link_underline` function.
2. Tests (in `cell.rs`): a `link_underline_*` test —
   - **not a link**: each variant (`None`/`Single`/`Double`/`Curly`/`Dotted`/
     `Dashed`) → unchanged;
   - **a link, `Single`** → `Double`;
   - **a link, `None`** → `Single` (a link with no SGR underline gets one);
   - **a link, `Double`/`Curly`/`Dotted`/`Dashed`** → `Single`.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty link_underline
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `link_underline` returns the effective underline — `Double` for a link's
  single underline, `Single` for a link's other underline (incl. none), and the
  SGR underline unchanged for a non-link cell — faithful to upstream's link
  underline override;
- the tests pass (the non-link passthrough for every variant, and the link
  Single→Double / other→Single cases), and the existing tests still pass;
- the hovered-link set, the underline-pass wiring, and the Metal upload stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the override is wrong (a non-link cell changed, the
Single→Double / other→Single mapping inverted, the `None` link case not becoming
single), or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the override is faithful to upstream: non-links pass
through unchanged, linked cells convert `Single → Double`, and every other
underline state (including `None`) becomes `Single` — correctly giving
un-underlined links a single underline and distinguishing links that already had
a single underline. It agreed that `is_link: bool` is a sound bounded slice
while the hovered-link membership source is deferred, and that the draw gating
belongs where it already is (the underline pass's `underline != None` check), so
`link_underline` does not need to handle it. It judged the tests sufficient (all
underline variants for both the passthrough and the link-override behavior).

Review artifacts:

- Prompt: `logs/codex-review/20260603-211415-445086-prompt.md` (design)
- Result: `logs/codex-review/20260603-211415-445086-last-message.md` (design)

## Result

**Result:** Pass

The link underline override is now computable.

- `roastty/src/renderer/cell.rs`:
  `link_underline(is_link, underline) -> Underline` — a non-link cell returns
  its `underline` unchanged; a link cell turns `Single` into `Double` and every
  other underline (including `None`) into `Single`. `pub(crate)` and not yet
  called in production (the hovered-link membership and the underline-pass
  wiring are deferred), reachable in the library crate, so no dead-code warning.

Test (in `cell.rs`): `link_underline_applies_the_hovered_link_override` — a
non-link cell keeps all six variants (`None`/`Single`/`Double`/`Curly`/`Dotted`/
`Dashed`); a link cell maps `Single → Double`, `None → Single`, and
`Double`/`Curly`/`Dotted`/`Dashed → Single`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2856 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

The hovered-link underline override is now ported faithfully as
`link_underline`: a link cell's single underline becomes a double underline (to
distinguish it from the cell's own underline), any other underline (including
none) becomes single, and a non-link cell is unchanged. Like the search
highlights (Experiment 390), the link-membership source and the wiring into
`rebuild_row`'s underline pass are deferred until a hovered-link set is modeled.

The remaining renderer-bridge work: the hovered-link set (OSC 8 / regex link
membership) and wiring `link_underline` into the underline pass; the
column-ordered decoration merge; and the **Metal upload** of `Contents`.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
upstream logic exactly: non-link cells return their original underline, link
cells turn `Single` into `Double`, and every other underline variant (including
`None`) becomes `Single`. It confirmed the test covers the full variant matrix
for both the passthrough and the link-override behavior, that deferring the
hovered-link membership and the underline-pass wiring is reasonable for this
scoped helper, and that there is no public C ABI/header impact. Nothing needed
to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-211612-122555-last-message.md`
