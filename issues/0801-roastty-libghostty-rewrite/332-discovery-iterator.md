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

# Experiment 332: font discovery — the iterator (deferred faces)

## Description

`discover_descriptors` (Experiment 331) yields the matching descriptors ranked
best-first. Upstream's `discover` returns a **`DiscoverIterator`** over those
descriptors whose `next()` turns each into a **`DeferredFace`** — lazily: it
**removes the character-set attribute** (which was a _search filter_, not a
render constraint) and creates a `CTFont`. This experiment ports that `next()`
step as a lazy iterator of roastty `Face`s, completing `CoreText.discover`
end-to-end.

## Upstream behavior (`discovery.zig` `DiscoverIterator.next`)

```zig
pub fn next(self) !?DeferredFace {
    if (self.i >= self.list.len) return null;

    // Remove the character-set attribute: it was used to filter, but we don't
    // want it now because it would restrict the available characters.
    const desc = desc: {
        const attrs = MutableDictionary.create(0);
        attrs.setValue(FontAttribute.character_set.key(), kCFNull); // null ⇒ drop
        break :desc FontDescriptor.createCopyWithAttributes(self.list[self.i], attrs);
    };

    const font = Font.createWithFontDescriptor(desc, 12); // size altered later
    defer self.i += 1;
    return DeferredFace{ .ct = .{ .font = font, .variations = self.variations } };
}
```

Setting an attribute to `kCFNull` in a `createCopyWithAttributes` **drops** it
from the copy. The resulting descriptor (no character-set filter) is turned into
a `CTFont` at size 12 (resized later when actually used). A `DeferredFace` wraps
that font plus the requested variations.

## Rust mapping (`roastty/src/font/discovery.rs`)

roastty has no second font backend and its `Face` is the CoreText-backed face,
so the `DeferredFace` wrapper collapses to producing a `Face` directly — but
**lazily**, via an iterator, preserving the key property that the (expensive)
face creation happens per-`next()`, not for every candidate up front.

- `roastty/src/font/face/coretext.rs`: make `Face::from_ct_font` `pub(crate)`
  (it currently exists, private) so discovery can wrap a `CTFont` in a `Face`.
- `roastty/src/font/discovery.rs`:
  `pub(crate) fn discover_faces(&self) -> impl Iterator<Item = Face>`:
  ```rust
  self.discover_descriptors().into_iter().map(deferred_face)
  ```
  where `fn deferred_face(desc: CFRetained<CTFontDescriptor>) -> Face`:
  - builds a `CFMutableDictionary` with
    `kCTFontCharacterSetAttribute → kCFNull`,
  - `let desc = desc.copy_with_attributes(attrs.as_opaque())` (drops the
    character set),
  - `let font = CTFont::with_font_descriptor(&desc, 12.0, null)`,
  - `Face::from_ct_font(font)`. The `.map` is lazy: each `Face` is created only
    when the iterator advances.

## Scope / faithfulness notes

- **Ported**: `DiscoverIterator.next` — the character-set removal, the `CTFont`
  creation at size 12, and the lazy per-candidate face production. The
  descriptor list is materialized (matching upstream, which sorts the list
  eagerly then lazily creates faces in `next()`).
- **Simplification**: `DeferredFace` collapses to roastty's `Face` (no second
  backend; the face is created eagerly per `next()` rather than wrapping an
  un-resolved font) — documented.
- **Deferred**: applying the requested **variations** to the produced face
  (carried on the `Descriptor` but not applied — its own concern), the
  **variation-axis score** refinement, `discoverFallback`/`discoverCodepoint`,
  and the resolver wiring.
- No C ABI/header/ABI-inventory change (internal Rust; only `Face::from_ct_font`
  visibility widens to `pub(crate)`).

## Changes

1. `roastty/src/font/face/coretext.rs`: `Face::from_ct_font` → `pub(crate)`.
2. `roastty/src/font/discovery.rs`: add `discover_faces` and the `deferred_face`
   helper. Import `Face`, `CTFont`, `kCFNull`, `kCTFontCharacterSetAttribute`
   (already imported).
3. Tests (in `discovery.rs`):
   - `discover_faces_first_renders`:
     `Descriptor { family: Some("Menlo"), .. } .discover_faces().next()` is
     `Some(face)`, and the face renders `'M'`
     (`face.glyph_index('M' as u32).is_some()`).
   - `discover_faces_charset_removed`: a codepoint-filtered request
     (`Descriptor { family: Some("Menlo"), codepoint: 'M' as u32, .. }`) yields
     a first face that renders **codepoints beyond** the search codepoint — e.g.
     `glyph_index('A' as u32).is_some()` and `glyph_index('z' as u32).is_some()`
     — proving the produced face is the full font, not a
     character-set-restricted one (the attribute was dropped).
   - `discover_faces_lazy_count`: `discover_faces()` for a `monospace` request
     can be `.take(1)` without creating every face (a smoke test that the
     iterator is lazy and the first element is a valid face). (Exact laziness is
     structural; the test confirms `.next()` yields a usable face.)
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty discover
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `discover_faces` reproduces `DiscoverIterator.next` — the character-set
  removal, the `CTFont` creation, and the lazy per-candidate face — yielding
  usable `Face`s best-first;
- the first-renders, charset-removed, and lazy tests pass;
- variations application, the variation-axis score, the fallback, and the
  resolver wiring stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the character-set-removed assertion is not
host-observable (the face is still produced and renders the search codepoint).

The experiment **fails** if the character-set removal or the face production
diverges from upstream, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
findings**. It confirmed the design matches upstream's `DiscoverIterator.next`
shape (descriptors collected and sorted eagerly, while `CTFont`/`Face` creation
happens per iterator advance), that `kCTFontCharacterSetAttribute → kCFNull` in
the copied descriptor is the faithful port of the character-set removal, that
wrapping the resulting `CTFont` with `Face::from_ct_font` is the right
single-backend simplification, that making `Face::from_ct_font` `pub(crate)` is
a minimal boundary change, and that deferring variation application is
acceptable (upstream carries variations in `DeferredFace`, but the actual
handling is separately scoped here).

One **Optional** note: the charset-removed test is an observable-behavior test
rather than a formal proof the attribute is absent; a direct
descriptor-attribute assertion would be a possible extra. It is **not** adopted
— as Experiment 325 established, `CTFontDescriptorCopyAttribute` reflects
CoreText's _resolved_ attributes (it can re-infer a character set), so an
absence assertion would be unreliable. The behavioral test (the produced face
renders codepoints beyond the search one) is the robust check.

Review artifacts:

- Prompt: `logs/codex-review/20260603-121042-701325-prompt.md`
- Result: `logs/codex-review/20260603-121042-701325-last-message.md`

## Result

**Result:** Pass

The discovery iterator lands — `CoreText.discover` is now ported end-to-end.

- `roastty/src/font/face/coretext.rs`: `Face::from_ct_font` widened to
  `pub(crate)`.
- `roastty/src/font/discovery.rs`:
  `discover_faces(&self) -> impl Iterator<Item = Face>` maps the best-first
  descriptor list through `deferred_face`, which copies each descriptor with
  `kCTFontCharacterSetAttribute → kCFNull` (dropping the search filter), creates
  a `CTFont` at size 12, and wraps it in a `Face`. The `.map` is lazy — each
  face is built only when the iterator advances. The module doc was updated.

Tests: `discover_faces_first_renders` (the first Menlo face renders `'M'`),
`discover_faces_charset_removed` (a codepoint-filtered request yields a first
face that renders `'A'`/`'z'` too — the full font, not a
character-set-restricted one), `discover_faces_lazy_smoke` (the first face of a
monospace search is usable).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2721 passed, 0 failed (+3, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

`Descriptor::discover_faces` completes `CoreText.discover`: a `Descriptor` now
yields usable `Face`s, best-first, with the character-set filter removed — the
full pipeline from query to ranked, loadable faces. `DeferredFace`'s laziness is
preserved by the iterator; its wrapper collapses to roastty's single-backend
`Face`.

The remaining discovery work is `discoverFallback`/`discoverCodepoint` (the
codepoint-driven fallback search — a `discoverCodepoint` that scores candidates
by codepoint coverage), the deferred **variation-axis** score refinement, and
applying **variations** to the produced face. After discovery: the resolver's
discovery-based fallback and codepoint overrides in `get_index` (where a
codepoint with no loaded face triggers a `discover` and the result is added to
the collection), then the shaper.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no Required findings**. It confirmed the implementation is faithful to
upstream's `DiscoverIterator.next` (copy each sorted descriptor with
`kCTFontCharacterSetAttribute → kCFNull`, create a `CTFont` at size 12, wrap as
a `Face` only when the iterator advances), that the `kCFNull` usage is sound
(the singleton is a live CF object passed as a raw value pointer into a
value-retaining CF dictionary that stays live through `copy_with_attributes`,
the copied descriptor is retained, and the `CTFont` is retained by `Face` — no
lifetime hazards after `attrs`/the copy go out of scope), that the lazy iterator
preserves upstream's eager-discover/lazy-create structure, and that widening
`Face::from_ct_font` to `pub(crate)` is the minimal access change. It agreed the
behavioral charset test is appropriate (a direct attribute inspection would be
unreliable given CoreText's resolved-attribute behavior). No Optional findings.

Review artifacts:

- Result review: `logs/codex-review/20260603-121345-528363-last-message.md`
