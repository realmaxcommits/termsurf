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

# Experiment 486: the config Unicode range parser (config::unicode_range::UnicodeRangeParser)

## Description

The `font-codepoint-map` / `clipboard-codepoint-map` config keys map Unicode
codepoint ranges to a value (a font family / a clipboard policy). The shared
piece behind their parsers is upstream's `UnicodeRangeParser` (nested in
`Config.RepeatableCodepointMap`) — a small state machine that walks a key string
like `U+1234-U+5678, U+9ABC` and yields `[start, end]` codepoint ranges. This
experiment ports that parser into a new `roastty/src/config/unicode_range.rs`
module, unblocking `RepeatableCodepointMap` / `RepeatableClipboardCodepointMap`
(later slices, once the font/clipboard map storage is ported). The surrounding
`RepeatableCodepointMap` (which needs the font `CodepointMap`) stays deferred.

## Upstream behavior

In `config/Config.zig`, `RepeatableCodepointMap.UnicodeRangeParser`:

```zig
/// Valid syntax: "" (empty → null) | U+1234 | U+1234-5678 | U+1234,U+5678 | …
const UnicodeRangeParser = struct {
    input: []const u8,
    i: usize = 0,

    pub fn next(self: *UnicodeRangeParser) !?[2]u21 {
        if (self.eof()) return null;
        const start = try self.parseCodepoint();
        if (self.eof()) return .{ start, start };
        self.consumeWhitespace();
        switch (self.input[self.i]) {
            ',' => {
                self.advance();
                self.consumeWhitespace();
                if (self.eof()) return error.InvalidValue;
                return .{ start, start };
            },
            '-' => {
                self.advance();
                self.consumeWhitespace();
                if (self.eof()) return error.InvalidValue;
                const end = try self.parseCodepoint();
                self.consumeWhitespace();
                if (!self.eof() and self.input[self.i] != ',') return error.InvalidValue;
                self.advance();
                self.consumeWhitespace();
                if (start > end) return error.InvalidValue;
                return .{ start, end };
            },
            else => return error.InvalidValue,
        }
    }

    fn consumeWhitespace(self: *UnicodeRangeParser) void {
        while (!self.eof()) switch (self.input[self.i]) {
            ' ', '\t' => self.advance(),
            else => return,
        };
    }

    fn parseCodepoint(self: *UnicodeRangeParser) !u21 {
        if (self.input[self.i] != 'U') return error.InvalidValue;
        self.advance();
        if (self.eof()) return error.InvalidValue;
        if (self.input[self.i] != '+') return error.InvalidValue;
        self.advance();
        if (self.eof()) return error.InvalidValue;

        const start_i = self.i;
        while (true) {
            const current = self.input[self.i];
            const is_hex = (current >= '0' and current <= '9') or
                (current >= 'A' and current <= 'F') or
                (current >= 'a' and current <= 'f');
            if (!is_hex) break;
            self.advance();
            if (self.eof()) break;
        }
        if (start_i == self.i) return error.InvalidValue;
        return std.fmt.parseInt(u21, self.input[start_i..self.i], 16) catch return error.InvalidValue;
    }

    fn advance(self: *UnicodeRangeParser) void { self.i += 1; }
    fn eof(self: *const UnicodeRangeParser) bool { return self.i >= self.input.len; }
};
```

- `next` yields one range per call (or `null` at end): it parses a `U+XXXX`
  codepoint, and if that is the whole remaining input returns `[start, start]`;
  after optional whitespace, a `,` yields `[start, start]` (and requires more
  input after it — a trailing comma is `error.InvalidValue`), a `-` parses a
  second codepoint into `[start, end]` (requiring `start <= end` and that what
  follows is end-of-input or a `,`), and anything else is `error.InvalidValue`.
- `parseCodepoint` requires the literal `U+`, then a non-empty run of hex digits
  parsed as a base-16 `u21` (a missing `U`/`+`, no hex digits, or an overflow is
  `error.InvalidValue`).
- Whitespace (`" \t"`) is consumed around the `-`, the `,`, and the codepoints.

Upstream's `RepeatableCodepointMap.parseCLI` test drives this with the keys
`"U+ABCD"` → `[0xABCD, 0xABCD]`; `"U+0001 - U+0005"` → `[1, 5]`;
`"U+0006-U+0009, U+ABCD"` → `[6, 9]` then `[0xABCD, 0xABCD]`.

## Rust mapping (new `roastty/src/config/unicode_range.rs`)

```rust
//! Unicode codepoint-range parsing (port of upstream
//! `Config.RepeatableCodepointMap.UnicodeRangeParser`).
//!
//! Walks a key string like `U+1234-U+5678, U+9ABC` yielding `[start, end]`
//! codepoint ranges. The surrounding `RepeatableCodepointMap` (which needs the
//! font codepoint-map storage) is ported later.

/// A failure parsing a Unicode range (upstream `error.InvalidValue`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InvalidRange;

pub(crate) struct UnicodeRangeParser<'a> {
    input: &'a [u8],
    i: usize,
}

impl<'a> UnicodeRangeParser<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        UnicodeRangeParser { input, i: 0 }
    }

    /// Yield the next `[start, end]` range, `None` at end, or `InvalidRange`.
    pub(crate) fn next(&mut self) -> Result<Option<[u32; 2]>, InvalidRange> {
        if self.eof() {
            return Ok(None);
        }
        let start = self.parse_codepoint()?;
        if self.eof() {
            return Ok(Some([start, start]));
        }
        self.consume_whitespace();
        match self.byte() {
            b',' => {
                self.advance();
                self.consume_whitespace();
                if self.eof() {
                    return Err(InvalidRange); // trailing comma
                }
                Ok(Some([start, start]))
            }
            b'-' => {
                self.advance();
                self.consume_whitespace();
                if self.eof() {
                    return Err(InvalidRange);
                }
                let end = self.parse_codepoint()?;
                self.consume_whitespace();
                if !self.eof() && self.byte() != b',' {
                    return Err(InvalidRange);
                }
                self.advance();
                self.consume_whitespace();
                if start > end {
                    return Err(InvalidRange);
                }
                Ok(Some([start, end]))
            }
            _ => Err(InvalidRange),
        }
    }

    fn consume_whitespace(&mut self) {
        while !self.eof() {
            match self.byte() {
                b' ' | b'\t' => self.advance(),
                _ => return,
            }
        }
    }

    fn parse_codepoint(&mut self) -> Result<u32, InvalidRange> {
        if self.eof() || self.byte() != b'U' {
            return Err(InvalidRange);
        }
        self.advance();
        if self.eof() || self.byte() != b'+' {
            return Err(InvalidRange);
        }
        self.advance();
        if self.eof() {
            return Err(InvalidRange);
        }

        let start_i = self.i;
        loop {
            if !self.byte().is_ascii_hexdigit() {
                break;
            }
            self.advance();
            if self.eof() {
                break;
            }
        }
        if start_i == self.i {
            return Err(InvalidRange);
        }
        parse_hex_u21(&self.input[start_i..self.i]).ok_or(InvalidRange)
    }

    fn byte(&self) -> u8 {
        self.input[self.i]
    }

    fn advance(&mut self) {
        self.i += 1;
    }

    fn eof(&self) -> bool {
        self.i >= self.input.len
    }
}

/// Parse a hex run as a `u21`-range codepoint (upstream `parseInt(u21, _, 16)`):
/// each digit accumulates, and a value `> 0x10FFFF`-able `u21` max (`0x1FFFFF`) is
/// an overflow. The bytes are already known to be hex.
fn parse_hex_u21(bytes: &[u8]) -> Option<u32> {
    let mut value: u32 = 0;
    for &c in bytes {
        let digit = (c as char).to_digit(16)?;
        // Checked arithmetic so a long hex run can never wrap/panic; the
        // `> 0x1FFFFF` check then enforces the `u21` bound.
        value = value.checked_mul(16)?.checked_add(digit)?;
        if value > 0x1FFFFF {
            return None; // exceeds u21
        }
    }
    Some(value)
}
```

`next` / `parse_codepoint` / `consume_whitespace` mirror upstream's state
machine exactly (the `U+` literal, the hex run, the `,` / `-` dispatch with its
whitespace and end-of-input rules, the `start <= end` check). `parse_hex_u21`
mirrors `parseInt(u21, _, 16)` for the already-isolated hex run (accumulate base
16, overflow at the `u21` max `0x1FFFFF`). The `[2]u21` becomes `[u32; 2]`
(codepoints fit `u32`). All failures collapse to `InvalidRange` (upstream's
single `error.InvalidValue`).

Note `parse_codepoint` indexes only when `!eof` (guarded), matching upstream's
caller contract (it is only invoked when input remains) while avoiding a Rust
out-of-bounds panic.

## Scope / faithfulness notes

- **Ported (bridged)**: the config `UnicodeRangeParser` (upstream
  `RepeatableCodepointMap.UnicodeRangeParser`) and its `parse_hex_u21`, plus
  `InvalidRange`.
- **Faithful**: the `next` state machine (single codepoint → `[start, start]`;
  `,` → `[start, start]` requiring more input; `-` → `[start, end]` with the
  whitespace, end-of-input/comma, and `start <= end` rules; else error); the
  `parseCodepoint` `U+` + hex-run logic; the `" \t"` whitespace consumption; the
  base-16 `u21` value with overflow → error — exactly upstream's parser.
- **Faithful adaptation**: `[]const u8` → `&[u8]`; `!?[2]u21` →
  `Result<Option<[u32; 2]>, InvalidRange>`; `parseInt(u21, _, 16)` →
  `parse_hex_u21` (accumulate + `0x1FFFFF` overflow); the single upstream error
  → `InvalidRange`. The `parse_codepoint` access is `eof`-guarded (upstream
  relies on its caller contract; the guard preserves behavior and avoids a
  panic).
- **Deferred**: the surrounding `RepeatableCodepointMap` /
  `RepeatableClipboardCodepointMap` (which need the font / clipboard
  codepoint-map storage and their `formatEntry`), and the broader config
  parser/formatter. (Consumed by later slices; this experiment lands the range
  parser.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/unicode_range.rs` (new): the module doc, `InvalidRange`,
   `UnicodeRangeParser` (`new` / `next` / `consume_whitespace` /
   `parse_codepoint` / `byte` / `advance` / `eof`), `parse_hex_u21`, and the
   tests.
2. `roastty/src/config/mod.rs`: add `mod unicode_range;`.
3. Tests (in `config/unicode_range.rs`): a helper collecting all `next()` calls
   into `Result<Vec<[u32; 2]>, InvalidRange>`, asserting:
   - the upstream keys: `"U+ABCD"` → `[[0xABCD, 0xABCD]]`; `"U+0001 - U+0005"` →
     `[[1, 5]]`; `"U+0006-U+0009, U+ABCD"` → `[[6, 9], [0xABCD, 0xABCD]]`.
   - the comma list `"U+1234,U+5678"` → `[[0x1234, 0x1234], [0x5678, 0x5678]]`;
     and the empty input `""` → `[]`.
   - lowercase hex (design-review Low): `"U+abcd"` → `[[0xABCD, 0xABCD]]`.
   - errors: `"U+1234,"` (trailing comma); `"X+1"` (no `U`); `"U1"` (no `+`);
     `"U+"` (no hex); `"U+GG"` (non-hex); `"U+5-U+1"` (`start > end`); `"U+1-2"`
     (range end not `U+`); `"U+200000"` (exceeds `u21`); and a long overflow run
     `"U+FFFFFFFFFFFFFFFF"` (design-review Low — proves no panic/wrap) — each
     `Err(InvalidRange)`.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty unicode_range
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `UnicodeRangeParser` yields the correct `[start, end]` ranges for single
  codepoints, ranges, and comma lists (with the whitespace and `start <= end`
  rules), and returns `InvalidRange` on every upstream failure case — faithful
  to upstream's parser;
- the tests pass (the upstream keys; the comma/empty cases; the error cases),
  and the existing tests still pass;
- `RepeatableCodepointMap` and the broader config parser/formatter stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a range is parsed wrong (wrong codepoint/overflow,
wrong `,`/`-` handling, wrong whitespace or `start <= end` handling, a failure
case accepted), an unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation across two rounds.

**Round 1 — one Required finding (fixed) + two Low (folded in).** Codex
confirmed the `next` / `parse_codepoint` state machine faithful (empty → `None`,
single → `[start, start]`, commas require following input, ranges require a
parsed end and `start <= end`, whitespace is only space/tab, the unconditional
post-range `advance()` matches upstream's EOF behavior) but flagged that
`parse_hex_u21`'s `value = value * 16 + digit` could wrap/panic before the
`> 0x1FFFFF` check on a long hex run. (In fact the per-iteration check bounds
`value` to ≤ 0x1FFFFF before each multiply, so the intermediate never exceeds
0x1FFFFFF < `u32::MAX` — but checked arithmetic is strictly safer.) Fixed with
`checked_mul(16)?.checked_add(digit)?`, preserving the overflow → `InvalidRange`
behavior. The two Low test additions (lowercase `U+abcd`; a long-overflow
`U+FFFFFFFFFFFFFFFF`) were folded into the test plan.

**Round 2 — approved, no findings.** Codex confirmed the checked `parse_hex_u21`
matches `parseInt(u21, _, 16) catch InvalidValue` (invalid hex / overflow →
`InvalidRange`, no wrap/panic), the added lowercase and long-overflow tests
cover the edge cases, and the state machine remains faithful
(`Config.zig:8074`/`:8147`). "Approved with no findings."

Review artifacts:

- Round 1 prompt: `logs/codex-review/20260604-142803-d486-prompt.md`
- Round 1 result: `logs/codex-review/20260604-142803-d486-last-message.md`
- Round 2 prompt: `logs/codex-review/20260604-142938-d486b-prompt.md`
- Round 2 result: `logs/codex-review/20260604-142938-d486b-last-message.md`

## Result

**Result:** Pass

The new `roastty/src/config/unicode_range.rs` module was implemented exactly as
the (Round-2-approved) design: `InvalidRange`, `UnicodeRangeParser` (`new` /
`next` / `consume_whitespace` / `parse_codepoint` / `byte` / `advance` / `eof`),
and `parse_hex_u21` (checked, overflow → `None`). It is wired into
`config/mod.rs` via `mod unicode_range;`. Two tests cover the upstream keys, the
comma list, the empty input, lowercase hex, and the error cases (trailing comma,
bad `U+`, no hex, non-hex, `start > end`, range end not `U+`, and the short/long
overflows).

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 2970 passed, 0 failed (two new tests; no
  regressions).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + lib.rs/header/abi_harness.c)
  clean; `git diff --check` clean.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no
findings**: the parser matches upstream `UnicodeRangeParser` (EOF → `None`,
codepoint parsing requires `U+` plus ≥1 hex digit, whitespace is only space/tab,
the comma/range state transitions match, trailing comma and reversed ranges
error, and `parse_hex_u21` safely maps invalid/overflowing `u21` values to
`InvalidRange` — `Config.zig:8074`/ `:8123`); the tests cover the upstream
examples, lowercase hex, empty input, syntax errors, range ordering, and
overflow; gates are clean. "Approved with no findings."

Review artifacts:

- Prompt: `logs/codex-review/20260604-143213-r486-prompt.md` (result)
- Result: `logs/codex-review/20260604-143213-r486-last-message.md` (result)

## Conclusion

The Unicode range parser is ported as a reusable `config::unicode_range` module
— the `U+XXXX[-U+YYYY][, …]` state machine that the codepoint-map config keys
use. This unblocks `RepeatableCodepointMap` / `RepeatableClipboardCodepointMap`
(once the font / clipboard codepoint-map storage is ported). The config layer
now has three reusable parsing helpers (`config::string`,
`config::unicode_range`, and the integer/bool parsers) alongside eleven value
types. The next slice can port the font `CodepointMap` storage (toward
`RepeatableCodepointMap`), another self-contained value type, or begin the
per-field parser dispatch, continuing toward the full config loader.
