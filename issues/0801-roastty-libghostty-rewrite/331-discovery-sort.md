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

# Experiment 331: font discovery — sorting the candidates

## Description

`discover_descriptors` (Experiment 326) returns its candidates in CoreText's own
order. Upstream's `discover` then **sorts** them best-first with
`sortMatchingDescriptors`, using the now-complete `Score` (Experiments 327–330).
This experiment wires the ranking in: each candidate is scored against the
request and the list is ordered by `Score::int()` descending, so the best match
comes first — the last piece of `CoreText.discover` before the
`DiscoverIterator`/`DeferredFace`.

## Upstream behavior (`discovery.zig`)

```zig
fn sortMatchingDescriptors(desc, list) {
    std.mem.sortUnstable(*FontDescriptor, list, desc, struct {
        fn lessThan(d, lhs, rhs) bool {
            // Higher score is "less" (earlier).
            return Score.score(d, lhs).int() > Score.score(d, rhs).int();
        }
    }.lessThan);
}
// … called by discover() right after copyMatchingDescriptors, before returning
// the DiscoverIterator over the sorted list.
```

The comparator ranks a higher `int()` earlier (an unstable sort; ties are
unordered). The score for each candidate is `Score.score(desc, candidate)`.

## Rust mapping (`roastty/src/font/discovery.rs`)

- In `discover_descriptors`, after collecting the candidate `Vec`, sort it
  best-first by the request's score for each:
  ```rust
  // Score each candidate once, then order best-first by Score::int().
  let mut scored: Vec<(u32, CFRetained<CTFontDescriptor>)> =
      out.into_iter().map(|d| (self.score(&d).int(), d)).collect();
  scored.sort_by(|a, b| b.0.cmp(&a.0)); // descending
  scored.into_iter().map(|(_, d)| d).collect()
  ```
- This differs from upstream only in **mechanism**: upstream recomputes the
  score inside the comparator (each comparison re-loads the font); the port
  computes each candidate's `int()` **once** and sorts by it. The resulting
  order is identical (the comparator only depends on the per-candidate `int()`),
  and it avoids the O(n log n) font reloads. (A second, minor refinement:
  `sort_by` is **stable**, so equal-score ties keep CoreText's input order — a
  valid total order under the same comparator, where upstream's `sortUnstable`
  leaves ties unspecified.)
- The doc comment on `discover_descriptors` is updated: the list is now returned
  **best-first**.

## Scope / faithfulness notes

- **Ported**: `sortMatchingDescriptors` — `discover_descriptors` now returns the
  candidates ranked best-first by the request's `Score`.
- **Deferred**: the **variation-axis** bold/italic refinement (still — it
  sharpens `score()` for variable fonts but does not change the sort wiring),
  the `DiscoverIterator`/`DeferredFace` (the next experiment — turning the
  sorted descriptors into lazily-loaded faces with the character-set attribute
  removed), `discoverFallback`/`discoverCodepoint`, and the resolver wiring.
- The score-once-then-sort mechanism and the stable-tie order are documented
  refinements that produce a faithful (identical-or-valid) ordering.
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/font/discovery.rs`: sort the candidate `Vec` in
   `discover_descriptors`; update its doc comment.
2. Tests (in `discovery.rs`):
   - `discover_sorted_descending`: `discover_descriptors` returns a list that is
     non-increasing in `self.score(candidate).int()` (each element scores `>=`
     the next) — proving the sort is applied. Uses a `monospace`-trait request
     (which returns several candidates).
   - `discover_bold_ranks_bold_first`: a
     `Descriptor { family: Some("Menlo"), bold: true, .. }` discovery puts a
     **bold** candidate first — the first result, scored against the bold
     request, has `bold == true` (the bold variant's `bold`/`exact_style` bits
     rank it above the regular face). (If the host returns a single Menlo
     candidate, the assertion still holds for that one; the test tolerates a
     one-element list.)
   - The existing `discover_descriptors_*` tests still pass (sorting preserves
     membership).
3. Format and test (`cargo fmt`, accept output).

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

- `discover_descriptors` returns its candidates ordered best-first by the
  request's `Score::int()`, faithful to `sortMatchingDescriptors`;
- the sorted-descending and bold-first tests pass, and the existing discovery
  tests still pass;
- the variation-axis refinement, the iterator/deferred-face, and the resolver
  wiring stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the host returns too few candidates to observe
ordering (the sort is still applied and the per-element score check holds).

The experiment **fails** if the ordering diverges from `Score::int()`
descending, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
findings**. It confirmed the design is faithful: scoring each descriptor
**once** and sorting by the cached `Score::int()` gives the **same** non-tie
ordering as upstream's score-per-comparison comparator (the comparator is a pure
function of each candidate's `int()`, and `score()` is deterministic per
candidate), and `sort_by(|a, b| b.0.cmp(&a.0))` correctly implements
higher-int-first. It agreed the stable `sort_by` is an acceptable refinement
(upstream's `sortUnstable` leaves tie order unspecified, so preserving
CoreText's input order for ties is valid under the same comparator), that the
Menlo bold-first test reasoning is sound (a bold request gives the bold face the
`bold` match bit and likely `exact_style` for `"Bold"` while regular loses
those, with `monospace` equal across Menlo faces), and that scoring every
candidate is a performance consideration only — the score-once plan is in fact
cheaper than upstream's recompute-per-comparison.

Review artifacts:

- Prompt: `logs/codex-review/20260603-120452-746623-prompt.md`
- Result: `logs/codex-review/20260603-120452-746623-last-message.md`

## Result

**Result:** Pass

The candidate sort lands — `discover_descriptors` now returns its matches
**best-first**.

- `roastty/src/font/discovery.rs`: after collecting the candidate `Vec`,
  `discover_descriptors` scores each candidate once against the request and
  sorts descending by `Score::int()` (`scored.sort_by(|a, b| b.0.cmp(&a.0))`).
  The doc comment now describes the best-first ordering.

Tests: `discover_sorted_descending` (the returned list is non-increasing in
`req.score(c).int()` for a monospace request — the public ordering invariant),
`discover_bold_ranks_bold_first` (a bold Menlo request ranks a bold candidate
first). The existing `discover_descriptors_*` tests still pass (sorting
preserves membership).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2718 passed, 0 failed (+2, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

`CoreText.discover` is now ported end-to-end up to the iterator: a `Descriptor`
yields its candidate font descriptors ranked best-first by the full `Score`
(glyph count, codepoint coverage, monospace, head/OS-2-refined bold/italic, and
the style match). Only the variation-axis score refinement remains deferred.

The next discovery experiment is the **`DiscoverIterator`/`DeferredFace`** —
upstream turns each sorted descriptor into a lazily-loaded face, **removing the
character-set attribute** first (it was a filter, not a render constraint) via
`createCopyWithAttributes` with `kCFNull`, then
`CTFontCreateWithFontDescriptor`. After that:
`discoverFallback`/`discoverCodepoint`, then the resolver's discovery-based
fallback and codepoint overrides in `get_index`, and finally the shaper.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no Required findings**. It confirmed the wiring matches upstream ordering
(each candidate's `Score::int()` computed once, `b.0.cmp(&a.0)` sorting higher
first — equivalent to upstream's score-per-comparison comparator for all
non-ties, with the stable tie behavior acceptable since upstream leaves
equal-score order unspecified), that ownership is correct (`out` moved into
`(score, descriptor)` tuples, sorted without dropping the retained descriptors,
mapped back into the returned `Vec`), that the doc update is accurate, and that
the tests are meaningful (`discover_sorted_descending` verifies the public
ordering invariant; `discover_bold_ranks_bold_first` exercises an observable
ranking effect, within the experiment's stated system-font test tolerance). No
Optional findings.

Review artifacts:

- Result review: `logs/codex-review/20260603-120734-443077-last-message.md`
