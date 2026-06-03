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

# Experiment 325: font discovery — the CoreText descriptor

## Description

Font **discovery** (finding a system font that matches a family/style/codepoint
request) is the subsystem that gates the resolver's discovery-based fallback and
its codepoint overrides — both still deferred. Discovery's foundation is
converting a [`Descriptor`] (`discovery.rs`) into a CoreText `CTFontDescriptor`,
the query object CoreText's matching APIs consume. This experiment ports that
first slice: `Descriptor::to_core_text_descriptor`, a faithful port of upstream
`Descriptor.toCoreTextDescriptor`. The actual `discover`/`discoverFallback`
iteration (which builds on this descriptor) is the next experiment.

## Upstream behavior (`discovery.zig` `toCoreTextDescriptor`)

Build a mutable attributes dictionary, set the keys that are present, and create
the descriptor:

```zig
const attrs = MutableDictionary.create(0);
if (family)   attrs.setValue(FontAttribute.family_name.key(), String(family));
if (style)    attrs.setValue(FontAttribute.style_name.key(),  String(style));
if (codepoint > 0) {
    const cs = CharacterSet.createWithCharactersInRange(.{ .location = cp, .length = 1 });
    attrs.setValue(FontAttribute.character_set.key(), cs);
}
if (size > 0)  attrs.setValue(FontAttribute.size.key(), Number(sint32, round(size)));
const traits = FontSymbolicTraits{ .bold = bold, .italic = italic, .monospace = monospace };
if (@bitCast(traits) > 0) {                       // any trait set
    const traits_dict = MutableDictionary.create(0);
    traits_dict.setValue(FontTraitKey.symbolic.key(), Number(sint32, @bitCast(traits)));
    attrs.setValue(FontAttribute.traits.key(), traits_dict);
}
return FontDescriptor.createWithAttributes(attrs);
```

Key points: only-present fields are set; the size is **rounded to an `i32`**;
the traits go in a **nested** dictionary under `kCTFontTraitsAttribute` keyed by
`kCTFontSymbolicTrait`, and only when at least one of bold/italic/monospace is
set; the symbolic-trait value is the bitwise OR of the requested trait bits.

## Rust mapping

- `roastty/Cargo.toml`: enable the binding features this needs — `CFDictionary`,
  `CFNumber`, `CFCharacterSet` on `objc2-core-foundation`, and `CTFontTraits` on
  `objc2-core-text` (the `CTFontDescriptor` attribute keys are already enabled).
- `roastty/src/font/discovery.rs`: add
  `pub(crate) fn to_core_text_descriptor(&self) -> CFRetained<CTFontDescriptor>`
  on `Descriptor`:
  - Create a `CFMutableDictionary` (`CFDictionary::new_mutable` / the objc2
    constructor).
  - `family` → `CFString::from_str`, set under `kCTFontFamilyNameAttribute`.
  - `style` → set under `kCTFontStyleNameAttribute`.
  - `codepoint != 0` →
    `CFCharacterSet::with_characters_in_range(CFRange { location: cp as isize, length: 1 })`,
    set under `kCTFontCharacterSetAttribute`.
  - `size > 0.0` → `CFNumber::new(kCFNumberSInt32Type, &(size.round() as i32))`,
    set under `kCTFontSizeAttribute`.
  - traits: build `CTFontSymbolicTraits` from `bold` (`TraitBold`), `italic`
    (`TraitItalic`), `monospace` (`TraitMonoSpace`); if the bitmask is non-zero,
    create a nested `CFMutableDictionary`, set the `i32` bitmask under
    `kCTFontSymbolicTrait`, and set the nested dict under
    `kCTFontTraitsAttribute`.
  - `CTFontDescriptor::with_attributes(&attrs)`.
  - The `unsafe` CoreText/CoreFoundation calls are wrapped with `// SAFETY:`
    notes, matching the existing `coretext.rs` style (live CF objects,
    documented null-allocator usage).
- The `variations` field of `Descriptor` is **not** set here (upstream's
  `toCoreTextDescriptor` does not set variations either — they are applied later
  when a font is instantiated); it stays carried on the `Descriptor`.

## Scope / faithfulness notes

- **Ported**: `Descriptor.toCoreTextDescriptor` — the attribute dictionary, the
  rounded size, the nested symbolic-traits dictionary, and the descriptor
  creation.
- **Deferred**: `discover`/`discoverFallback` (the `CTFontCollection` /
  `discoverCodepoint` matching that yields candidate faces) — the next discovery
  experiment; and the resolver wiring (discovery fallback, codepoint overrides)
  after that.
- No C ABI/header/ABI-inventory change (`Descriptor`/`CTFontDescriptor` are
  internal Rust); the only build change is enabling already-present objc2
  binding features.

## Changes

1. `roastty/Cargo.toml`: enable `CFDictionary`/`CFNumber`/`CFCharacterSet`
   (`objc2-core-foundation`) and `CTFontTraits` (`objc2-core-text`).
2. `roastty/src/font/discovery.rs`: add `Descriptor::to_core_text_descriptor`.
3. Tests (in `discovery.rs`):
   - `descriptor_family_round_trips`: a
     `Descriptor { family: Some("Menlo"), .. }` → `to_core_text_descriptor` →
     create a `CTFont` from the descriptor (or copy the
     `kCTFontFamilyNameAttribute` back) → the family name is `"Menlo"`.
   - `descriptor_traits_set`: a `Descriptor { bold: true, italic: true, .. }`
     produces a descriptor whose `kCTFontTraitsAttribute` →
     `kCTFontSymbolicTrait` value has the bold and italic bits set (and a
     no-traits descriptor omits the traits attribute / yields a zero/absent
     symbolic-trait value).
   - `descriptor_empty`: an all-default `Descriptor` (no family/style/codepoint/
     size/traits) still produces a valid (empty-attributes) descriptor without
     panicking.
   - `descriptor_size_rounded`: a `size: 12.6` descriptor's
     `kCTFontSizeAttribute` reads back as `13` (rounded `i32`), proving the
     round + `SInt32` encoding.
   - (Read-back uses `CTFontDescriptorCopyAttribute`; the exact assertions are
     finalized against the available CoreText readback APIs during
     implementation.)
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty descriptor
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `to_core_text_descriptor` reproduces upstream's attribute dictionary (the
  present-only fields, the rounded `SInt32` size, the nested symbolic-traits
  dictionary keyed by `kCTFontSymbolicTrait`, and the descriptor creation);
- the family round-trip, traits, empty, and rounded-size tests pass;
- `discover`/`discoverFallback` and the resolver wiring stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a CoreText read-back API needed by a test is
unavailable in the enabled bindings (the descriptor is still constructed and
exercised through font instantiation).

The experiment **fails** if the attribute dictionary, the size rounding, or the
symbolic-traits encoding diverges from upstream, or any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
findings**. It confirmed the design matches upstream
`Descriptor.toCoreTextDescriptor` on the fidelity points: the attributes are
present-only (family, style, codepoint, size, traits only when set; **variations
correctly omitted** here); the size is `round(size)` encoded as an `SInt32`
`CFNumber`; the traits go in the nested dictionary under
`kCTFontTraitsAttribute` keyed by `kCTFontSymbolicTrait`, only when the OR'd
`CTFontSymbolicTraits` bitmask is non-zero; the single-codepoint
`CFCharacterSet::with_characters_in_range(CFRange { location: cp, length: 1 })`
is faithful; and deferring `discover`/`discoverFallback` is a sensible first
slice (this descriptor conversion is the dependency those later paths need).

One **implementation note** (not Required): in `objc2-core-foundation 0.3.2` the
ergonomic constructor is `CFNumber::new_i32(value)` for an `SInt32` number; if a
lower-level `CFNumber::new(...)` is used, the type must be kept explicitly
`SInt32Type`. Folded into the implementation plan — use `CFNumber::new_i32` for
the rounded size and the symbolic-trait bitmask.

Review artifacts:

- Prompt: `logs/codex-review/20260603-111719-992826-prompt.md`
- Result: `logs/codex-review/20260603-111719-992826-last-message.md`

## Result

**Result:** Pass

The discovery descriptor lands — discovery's first stone.

- `roastty/Cargo.toml`: enabled `CFCharacterSet`/`CFDictionary`/`CFNumber`
  (`objc2-core-foundation`) and `CTFontTraits` (`objc2-core-text`).
- `roastty/src/font/discovery.rs`:
  `Descriptor::to_core_text_descriptor(&self) -> CFRetained<CTFontDescriptor>`
  builds a `CFMutableDictionary`, sets the present-only family/style
  (`CFString`), the single-codepoint character set
  (`CFCharacterSet::with_characters_in_range`), the rounded `SInt32` size
  (`CFNumber::new_i32(size.round() as i32)`), and the bold/italic/monospace
  symbolic traits OR'd into a **nested** dictionary under
  `kCTFontTraitsAttribute` keyed by `kCTFontSymbolicTrait` (only when non-zero);
  then `CTFontDescriptor::with_attributes(attrs.as_opaque())`. Variations are
  not set (faithful to upstream). A small `ct_ptr(&T)` helper produces the CF
  object pointers for the raw `set_value` calls.

Tests: `descriptor_family_round_trips` (`"Menlo"` read back via
`attribute(kCTFontFamilyNameAttribute).downcast::<CFString>()`),
`descriptor_size_rounded` (`12.6 → 13`, downcast `CFNumber::as_i32`),
`descriptor_traits_symbolic_bits` (the nested traits dict's
`kCTFontSymbolicTrait` `CFNumber` has the bold + italic bits set, monospace
unset), `descriptor_codepoint_charset_contains` (`0x00C0` is a member, `0x41` is
not), `descriptor_builds_empty` (an all-default descriptor builds without
panic).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2700 passed, 0 failed (+5, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

### Implementation note — a SIGSEGV caught and fixed

The first implementation passed `ct_ptr(&s)` where `s: CFRetained<CFString>` —
the address of the Rust smart-pointer **wrapper**, not the CF object — so
`set_value` received garbage and the test process crashed with `SIGSEGV`. Fixed
by dereferencing the `CFRetained` to the CF object at every value site
(`ct_ptr(&*s)`, `ct_ptr(&*cs)`, `ct_ptr(&*n)`) and using
`ct_ptr(traits_dict.as_opaque())` for the nested dictionary. The completion
review confirmed the fix is correct (the CF temporaries live across the
`set_value` call that retains them).

### A note on the tests

The design's absence-based assertions (an unset field ⇒ no attribute) were
**dropped**: `CTFontDescriptorCopyAttribute` reflects CoreText's _resolved_
attributes (it may infer values that were never in the input dictionary), so an
absence assertion tests CoreText's normalization rather than this code. The
tests were switched to **positive content** assertions (read back the family
string, the rounded size, the symbolic-trait bits, the charset membership),
which are unambiguous. This is the test-expressibility caveat the design
anticipated; the result is **Pass**, not Partial.

## Conclusion

A `Descriptor` now converts to the CoreText `CTFontDescriptor` query object,
faithfully reproducing upstream's attribute dictionary (present-only fields,
rounded `SInt32` size, nested symbolic traits). This is the dependency the rest
of discovery needs.

The next discovery experiment is `discover`/`discoverFallback` — building a
`CTFontCollection` from this descriptor (and the `discoverCodepoint` fallback
matching) to yield candidate faces — followed by the resolver wiring (the
discovery-based fallback in `get_index`, then codepoint overrides).

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no Required findings**. It confirmed the implementation is faithful to
upstream `toCoreTextDescriptor` (present-only
family/style/codepoint/size/traits, no variation attributes, rounded `SInt32`
size via `CFNumber::new_i32`, the symbolic traits correctly OR'd in a nested
dictionary under `kCTFontTraitsAttribute` keyed by `kCTFontSymbolicTrait`). It
confirmed the **pointer fix is correct** —
`ct_ptr(&*s)`/`ct_ptr(&*cs)`/`ct_ptr(&*n)` pass the CF object address (not the
`CFRetained` wrapper), the inserted objects live through each `set_value` call,
and the dictionaries' CFType callbacks retain the keys/values before the
temporaries drop; passing `traits_dict.as_opaque()` is likewise sound (the outer
dict retains it). It confirmed
`CTFontDescriptor::with_attributes(attrs.as_opaque())` is sound and that the
positive-content tests are appropriate (avoiding absence assertions is correct
because readback reflects CoreText normalization). No Optional findings.

Review artifacts:

- Result review: `logs/codex-review/20260603-112834-640302-last-message.md`
