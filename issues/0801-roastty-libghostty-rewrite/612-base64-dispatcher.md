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

# Experiment 612: base64 dispatcher (scalar path)

## Description

The natural companion to Exp 611: upstream `simd/base64.zig`, the public base64
entry point (`maxLen` / `decode`) that dispatches to either a C++ SIMD decoder
or the scalar `base64_scalar.scalar_decoder`. roastty has no SIMD C++ decoder,
so this ports the **scalar path** — the canonical `base64::max_len` /
`base64::decode` API the terminal uses for OSC 52 and Kitty Graphics, built
directly on Exp 611's `Base64Decoder`.

The SIMD branches (`if comptime options.simd`) and the `extern "c"`
`ghostty_simd_base64_*` declarations drop — roastty's build has no SIMD base64
artifact, so the scalar path is the only path (a future SIMD port can add the
branch).

## Upstream behavior (`base64.zig`)

```zig
pub fn maxLen(input) usize {
    if (options.simd) return ghostty_simd_base64_max_length(...);
    return maxLenScalar(input);
}
fn maxLenScalar(input) usize {
    return scalar_decoder.calcSizeForSlice(scalarInput(input)) catch { log.warn(...); return 0; };
}

pub fn decode(input, output) error{Base64Invalid}![]const u8 {
    if (options.simd) { ... ghostty_simd_base64_decode ... }
    return decodeScalar(input, output);
}
fn decodeScalar(input_raw, output) error{Base64Invalid}![]const u8 {
    const input = scalarInput(input_raw);
    const size = maxLenScalar(input);
    if (size == 0) return "";
    assert(output.len >= size);
    scalar_decoder.decode(output, scalarInput(input)) catch return error.Base64Invalid;
    return output[0..size];
}

/// Trim trailing '=' padding so the scalar path matches the SIMD path's output.
fn scalarInput(input) []const u8 {
    var i = 0;
    while (input[input.len - i - 1] == '=') i += 1;
    return input[0 .. input.len - i];
}
```

## Rust mapping (`roastty/src/terminal/base64.rs`, new file)

```rust
//! Public base64 decode entry point (port of upstream `simd/base64`, scalar path). Dispatches —
//! here, only to the scalar `base64_scalar::scalar_decoder`, since roastty has no SIMD base64
//! artifact yet. Used for OSC 52 and Kitty Graphics payloads.

use super::base64_scalar::scalar_decoder;

/// Invalid base64 input (upstream `error{Base64Invalid}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Base64Invalid;

/// The maximum decoded length for `input` (upstream `maxLen`/`maxLenScalar`). Returns 0 on a sizing
/// error (upstream logs and returns 0).
pub(crate) fn max_len(input: &[u8]) -> usize {
    scalar_decoder()
        .calc_size_for_slice(scalar_input(input))
        .unwrap_or(0)
}

/// Decode `input` into `output` (which must be at least `max_len(input)` bytes), returning the
/// decoded prefix (upstream `decode`/`decodeScalar`).
pub(crate) fn decode<'o>(input: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], Base64Invalid> {
    let stripped = scalar_input(input);
    let size = max_len(stripped);
    if size == 0 {
        return Ok(&[]);
    }
    assert!(output.len() >= size);
    scalar_decoder()
        .decode(output, stripped)
        .map_err(|_| Base64Invalid)?;
    Ok(&output[..size])
}

/// Trim trailing `=` padding so the scalar decoder (no-pad alphabet) accepts padded input and
/// matches the SIMD path's output (upstream `scalarInput`).
fn scalar_input(input: &[u8]) -> &[u8] {
    let trailing = input.iter().rev().take_while(|&&b| b == b'=').count();
    &input[..input.len() - trailing]
}
```

Registered in `terminal/mod.rs` as `#[allow(dead_code)] mod base64;`
(alphabetically between `array_list_collection` and `base64_scalar`).

### Notes / deviations

- The SIMD branches and `extern "c"` decls drop (no SIMD artifact); the scalar
  path is the sole implementation, matching upstream's non-SIMD build.
- `scalar_input` counts trailing `=` safely (`take_while`), so empty / all-pad
  input yields an empty slice instead of upstream's index underflow — a faithful
  result for valid input and a safe one for the degenerate case.
- `decode` passes the full `output` to `Base64Decoder::decode` (as upstream
  does) and returns `&output[..size]`; the decoder writes exactly `size`
  meaningful bytes regardless of `output.len()`.
- `size == 0` → empty slice (upstream `return ""`).
- `scalar_decoder()` is called per-entry (it builds small lookup tables); a
  future optimization could cache it, but per-call matches the simple upstream
  shape.

## Verification

- `cargo build -p roastty` — no warnings.
- `cargo test -p roastty` — no regressions; new tests mirror upstream + edges:
  - `max_len_of_padded_input` — `max_len(b"aGVsbG8gd29ybGQ=")` == 11.
  - `decode_padded_input` — `decode(b"aGVsbG8gd29ybGQ=", buf)` ==
    `b"hello world"`.
  - `decode_unpadded_input` — `decode(b"TWFu", buf)` == `b"Man"`.
  - `decode_strips_multiple_padding` — `decode(b"TWE==", buf)` == `b"Ma"` (both
    `=` trimmed before the no-pad decode).
  - `decode_invalid_input_errors` — a non-alphabet byte → `Base64Invalid`.
  - `max_len_of_empty_is_zero` / `decode_empty_is_empty` — degenerate inputs are
    safe (no panic).
- `cargo fmt -p roastty -- --check` — clean.
- no-ghostty grep on the new file / `terminal/mod.rs` — clean.
- `git diff --check` — clean.

Pass = `max_len` and `decode` reproduce upstream's scalar-path output (including
the `=`-stripping), error on invalid input, and handle degenerate inputs safely.

## Design Review

Codex reviewed the design and **APPROVED** it with **no Required, Optional, or
Nit findings**, confirming it is faithful to upstream's scalar-path dispatcher:
`max_len`'s `scalar_input` stripping matches `maxLenScalar` (and the padded
`aGVsbG8gd29ybGQ=` correctly yields 11); `decode` mirrors `decodeScalar` (strip,
size, empty-on-zero, capacity assert, no-pad scalar decode, return the decoded
prefix); passing the full `output` and returning `[..size]` is faithful (the
decoder may use the extra capacity for its over-wide fast writes); the safe
`scalar_input` for empty/all-`=` input is a good adaptation of an upstream edge
that can underflow; dropping the SIMD branches / `extern` decls is the right
scoped port; and `Base64Invalid` as a unit error with a private
`terminal/base64` placement is appropriate.

Review artifacts:

- Prompt: `logs/codex-review/20260605-d612-prompt.md`
- Result: `logs/codex-review/20260605-d612-last-message.md`

## Result

**Result:** Pass

Implemented `roastty/src/terminal/base64.rs` (registered in `terminal/mod.rs`),
porting upstream `simd/base64`'s scalar path: `max_len` (strip `=` →
`calc_size_for_slice`, 0 on error), `decode` (strip → size → empty-on-zero →
capacity assert → no-pad scalar decode → `&output[..size]`), and the safe
`scalar_input` (`take_while` over trailing `=`, so empty / all-`=` input is safe
rather than underflowing). The SIMD branches and C++ `extern` declarations drop
(no SIMD artifact). An initial module-doc reference to the upstream extern
symbol name was rephrased to keep the no-ghostty gate clean on the source.

Seven tests: the upstream padded case (`max_len`==11, decode→`hello world`),
unpadded, multiple-`=` stripping, invalid-char error, and the degenerate
empty/all-`=` inputs (size 0, empty, no panic). Gates: `cargo fmt --check`
clean, `cargo build -p roastty` no warnings, `cargo test -p roastty` **3372
passed / 0 failed** (3365 → 3372, +7), no-ghostty grep clean, `git diff --check`
clean.

## Completion Review

Codex reviewed the completed experiment and **APPROVED** it with **no Required,
Optional, or Nit findings**, confirming the port is faithful: `max_len` strips
before sizing and returns 0 on error; `decode` matches `decodeScalar` (strip,
size, empty-on-zero, capacity assert, decode into the caller buffer, return
`&output[..size]`); the `decode` lifetime correctly ties the returned slice to
the output buffer; the safe `scalar_input` preserves valid-input behavior while
avoiding the degenerate underflow; and dropping the SIMD branch / extern decls
is correct for the current build.

Review artifacts:

- Prompt: `logs/codex-review/20260605-r612-prompt.md`
- Result: `logs/codex-review/20260605-r612-last-message.md`

## Conclusion

The base64 subsystem (`simd/base64_scalar` + `simd/base64`'s scalar path) is now
ported — the canonical decoder and its public `max_len` / `decode` entry point.
The remaining `simd/` companions are SIMD-intrinsic or width-table modules
(`index_of`, `codepoint_width`, `vt`) that roastty largely covers with scalar
equivalents; the base64 C++ SIMD fast path and the larger dependency boundaries
(libxev, oniguruma, `std.Uri`) remain. Issue 801 stays open and broad.
