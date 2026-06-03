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

# Experiment 267: Synthetic face methods — Face::synthetic_bold / synthetic_italic

## Description

`completeStyles` (Experiment 266) currently always _aliases_ a missing style. To
_synthesize_ one — a faux-bold or faux-italic of an existing face — the `Face`
needs the two instance methods upstream uses (`font/face/coretext.zig`):
`syntheticBold` (a copy marked with the bold line width) and `syntheticItalic`
(a copy with an oblique skew matrix). This experiment ports those two primitives
and the `italic_skew` transform; wiring them into `completeStyles` (with the
`FontSyntheticStyle` config) is the next experiment.

### Upstream behavior (`font/face/coretext.zig`)

- `italic_skew` (line 42):
  `CGAffineTransform{ a=1, b=0, c=0.267949 (≈ tan 15°), d=1, tx=0, ty=0 }` — a
  horizontal shear that obliques the glyphs.
- `syntheticBold(opts)` (lines ~180–198): `copyWithAttributes(0.0, null, null)`
  (a copy at the same size), then
  `synthetic_bold = max(opts.size.points/14, 1)`.
- `syntheticItalic(opts)` (lines ~174–178):
  `copyWithAttributes(0.0, &italic_skew, null)` — a copy with the skew matrix.
- `copyWithAttributes(size, matrix, attributes)`: `size = 0.0` preserves the
  source font's size; `matrix` applies a transform; `attributes` is a
  descriptor.

### Rust mapping (`roastty/src/font/face/coretext.rs`)

- `const ITALIC_SKEW: CGAffineTransform = CGAffineTransform { a: 1.0, b: 0.0, c: 0.267949, d: 1.0, tx: 0.0, ty: 0.0 }`
  (import `CGAffineTransform` from `objc2_core_foundation`).
- Extract a private `from_ct_font(font: CFRetained<CTFont>) -> Face` that builds
  a `Face` (`synthetic_bold: None`) and runs `detect_color`; `new` uses it.
- `synthetic_bold(&self) -> Face`: `copy_with_attributes(0.0, null, None)` (a
  size-preserving copy), `from_ct_font`, then
  `synthetic_bold = Some((self.size() / 14.0).max(1.0))` (upstream's heuristic,
  using the source face's point size).
- `synthetic_italic(&self) -> Face`:
  `copy_with_attributes(0.0, &ITALIC_SKEW, None)`, `from_ct_font` (no bold).
- Re-express the existing `new_synthetic_bold(name, size)` as
  `Face::new(name, size).synthetic_bold()` (removing the duplicated line-width
  logic; behavior is unchanged at the default size, keeping the Experiment 259
  tests valid).

### Scope / faithfulness notes

- These are the synthetic-face **primitives**. Their use in `completeStyles`
  (and the `FontSyntheticStyle` config that decides synthesize-vs-alias, plus
  the bold-italic synthesize-from-bold/italic preference) is the next
  experiment.
- `copy_with_attributes` size `0.0` preserves the source size (so the synthetic
  face inherits the regular face's size).
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/face/coretext.rs`:
   - Import `CGAffineTransform`; add the `ITALIC_SKEW` constant.
   - Extract `from_ct_font`; have `new` use it.
   - Add `synthetic_bold(&self)` and `synthetic_italic(&self)` instance methods.
   - Re-express `new_synthetic_bold` via `synthetic_bold`.
2. Tests in `coretext.rs` (live CoreText, macOS):
   - `synthetic_bold_method_sets_width`:
     `Face::new("Menlo", 28.0).synthetic_bold()` has
     `synthetic_bold == Some((28.0 / 14.0).max(1.0))`, and its `'M'` renders
     heavier (more total ink) than the plain Menlo `'M'` (instance-method analog
     of Experiment 259).
   - `synthetic_italic_renders`: `Face::new("Menlo", 32.0).synthetic_italic()`
     has `synthetic_bold == None`, still resolves `glyph_index('M')`, and
     renders `'M'` into a grayscale atlas with ink (the skew matrix doesn't
     break rendering). It also **asserts the skew was applied** by reading the
     face's transform via `CTFont::matrix()` and checking it equals
     `ITALIC_SKEW` (`c ≈ 0.267949`, `a == d == 1`, `b == tx == ty == 0`) — so
     the test fails if a null matrix were used by mistake.
   - `synthetic_face_inherits_color_detection`:
     `Face::new("Menlo", 32.0).synthetic_italic().has_color()` is `false` (color
     state is re-detected from the copied font's tables).
   - The Experiment 259 `new_synthetic_bold` tests still pass.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty face
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `synthetic_bold` returns a size-preserving copy marked with the
  `max(size/14, 1)` line width (rendering heavier), and `synthetic_italic`
  returns a copy with the `italic_skew` matrix that still renders;
- `from_ct_font` re-detects color so synthetic faces report color correctly;
- `new_synthetic_bold` keeps its Experiment 259 behavior via the new method;
- wiring into `completeStyles` is cleanly deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if `copy_with_attributes`'s matrix/size shape
needs adjustment against `cargo build`.

The experiment **fails** if the skew matrix or the bold line-width heuristic
diverges from upstream, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and raised one **Low** finding:
the `synthetic_italic_renders` test (resolve `'M'` + render ink) would also pass
for a plain copy with a null matrix, so it didn't prove the skew was applied.
The design was updated to additionally assert the face's transform via
`CTFont::matrix()` equals `ITALIC_SKEW` (`a=1, b=0, c≈0.267949, d=1, tx=ty=0`),
which fails if the skew matrix were omitted. No other findings — the
`italic_skew` values, the `self.size()`-based bold heuristic (size preserved by
`copy_with_attributes(0.0, …)`), the `from_ct_font` color re-detection, the
`new_synthetic_bold` re-expression, and the `&ITALIC_SKEW` FFI use are sound.

Review artifacts:

- Prompt: `logs/codex-review/20260602-221813-458858-prompt.md`
- Result: `logs/codex-review/20260602-221813-458858-last-message.md`

## Result

**Result:** Pass

`coretext.rs` gained the `ITALIC_SKEW` `CGAffineTransform`, a `from_ct_font`
helper (color-detecting `CTFont` → `Face`), and the `synthetic_bold` /
`synthetic_italic` instance methods (size-preserving
`copy_with_attributes(0.0, …)`). `new` and `new_synthetic_bold` are re-expressed
via the helper/method.

Tests (live CoreText):

- `synthetic_bold_method_sets_width` —
  `Face::new("Menlo", 28.0).synthetic_bold()` has
  `synthetic_bold == Some((28/14).max(1))` and renders `'M'` with more ink than
  plain Menlo.
- `synthetic_italic_renders` — the italic face has no bold, resolves
  `glyph_index('M')`, renders ink, and its `CTFont::matrix()` equals
  `ITALIC_SKEW` (proving the skew was applied, not a null matrix).
- `synthetic_face_inherits_color_detection` — Menlo's synthetic italic/bold are
  non-color (color re-detected from the copy).
- The Experiment 259 `new_synthetic_bold` tests still pass.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty face` → 37 passed, 0 failed.
- `cargo test -p roastty` → 2409 passed, 0 failed (no regressions; +3).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The synthetic-face primitives are ported. The next experiment wires them into
`completeStyles`: a `FontSyntheticStyle`-style config (`italic`/`bold`/
`bold-italic` toggles) selects synthesize-vs-alias per missing style, with the
bold-italic preference (synthesize italic-on-bold, else bold-on-italic, else
alias). After that the Collection's remaining work is the per-entry
`scale_factor`

- `load_options`/`setSize` size normalization and `setSize`/`updateMetrics`;
  then the `DeferredFace` + `discovery` lazy-loading sub-area, the
  `CodepointResolver`, the shaper, and the Nerd Font attribute table.

## Completion Review

Codex reviewed the completed implementation and result and found **no required
changes**.

Review artifacts:

- Prompt: `logs/codex-review/20260602-222157-734571-prompt.md`
- Result: `logs/codex-review/20260602-222157-734571-last-message.md`

Codex confirmed the implementation matches upstream: `synthetic_bold` makes a
size-preserving copy with a null matrix and applies `max(self.size()/14, 1)`,
`synthetic_italic` passes the exact `ITALIC_SKEW`, and `from_ct_font` re-runs
color detection on copied fonts. The `&ITALIC_SKEW` raw-pointer use is sound for
the call duration and `CTFont::matrix()` is used safely in the test;
`new_synthetic_bold` preserves prior behavior; and the matrix-equality assertion
closes the earlier test gap by proving the skew was applied.
