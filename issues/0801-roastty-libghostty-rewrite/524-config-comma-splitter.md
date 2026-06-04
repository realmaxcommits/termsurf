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

# Experiment 524: the quote/escape-aware comma splitter (CommaSplitter)

## Description

Toward `Theme::parse_cli` (whose light/dark-pair branch calls upstream
`parseAutoStruct`), this experiment ports the building block `parseAutoStruct`
needs: the upstream `cli/CommaSplitter` — an iterator that splits a string into
comma-separated fields while respecting double-quotes and Zig-style escapes, so
a comma inside a quoted section (or an escape sequence) does **not** split. It
only finds field boundaries; quotes and escapes are **not** decoded (that is a
separate step). Ported ahead of its consumer (`parseAutoStruct`), the same
approach used for the other loader primitives.

## Upstream behavior

`CommaSplitter.next` (`cli/CommaSplitter.zig:48`) is a labeled-`switch` state
machine with states `normal` / `quoted` / `escape` / `hexescape` /
`unicodeescape`:

- **normal**: a `,` ends the field (consumed); a `"` enters `quoted`; a `\`
  enters `escape` (recording `last = normal`); any other byte is part of the
  field. End of input returns the field from `start`.
- **quoted**: a `"` returns to `normal`; a `\` enters `escape`
  (`last = quoted`); any other byte stays quoted. End of input ⇒
  `error.UnclosedQuote`.
- **escape**: `n` / `r` / `t` / `\` / `'` / `"` return to `last`; `x` enters
  `hexescape`; `u` enters `unicodeescape`; anything else ⇒
  `error.IllegalEscape`. End of input ⇒ `error.UnfinishedEscape`.
- **hexescape**: two hex digits, then return to `last`; a non-hex digit ⇒
  `IllegalEscape`; end of input ⇒ `UnfinishedEscape`.
- **unicodeescape**: `{`, then ≥1 hex digits, then `}` returns to `last`; an
  empty `{}` or a non-hex/`{` ⇒ `IllegalEscape`; end of input ⇒
  `UnfinishedEscape`.

The escape states only **consume** the sequence (to keep a comma/quote inside it
from splitting); the splitter returns the raw field substring (quotes/escapes
intact).

`escape_outside_quotes` is `os != windows`; on **macOS it is `true`**, so a
backslash is an escape character both inside and outside quotes. (Per the
macOS-only directive, the Rust port resolves this to `true`.)

Two faithful quirks to replicate exactly:

- The `unicodeescape` accumulator does `value += d - '0'` / `value += d - 'a'` /
  `value += d - 'A'` — i.e. hex `a`–`f` / `A`–`F` add `d - 'a'` / `d - 'A'`
  (`0`–`5`), **not** `+10`. The accumulator is used _only_ for the `> 0x10ffff`
  overflow guard (the splitter never decodes the value), so this is copied
  verbatim to match upstream's exact `IllegalEscape` behavior.
- The `unicodeescape` overflow guard
  `if (value > 0x10ffff) return IllegalEscape` runs after each digit (in the
  `.digits` sub-state, not on the `}`).

## Rust mapping (`roastty/src/config/comma_splitter.rs`, new module)

A new `mod comma_splitter;` (added to `config/mod.rs`) with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommaSplitError {
    UnclosedQuote,
    UnfinishedEscape,
    IllegalEscape,
}

pub(crate) struct CommaSplitter<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> CommaSplitter<'a> {
    pub(crate) fn new(input: &'a str) -> Self { … }

    /// The next comma-separated field (quotes/escapes intact), or `None` when
    /// exhausted (upstream `CommaSplitter.next`).
    pub(crate) fn next(&mut self) -> Result<Option<&'a str>, CommaSplitError> { … }
}
```

`next` ports the state machine with a `state` variable and a
`loop { match state { … } }`, indexing `input.as_bytes()` and returning
`&input[start..end]` str slices (the split points are always at ASCII boundaries
— `,` / `"` / `\` and the hex digits are all ASCII, so multibyte UTF-8 field
content is consumed one continuation byte at a time in the `normal`/`quoted`
"other" arms, and the returned slices stay on char boundaries).
`escape_outside_quotes` is the constant `true` (macOS).

## Scope / faithfulness notes

- **Ported (bridged)**: `cli/CommaSplitter` (`new` + `next`), as
  `config::comma_splitter::CommaSplitter`.
- **Faithful**: the full state machine — comma splitting, double-quote
  protection, Zig escapes (`\n \r \t \\ \' \"`, `\xNN`, `\u{…}`), the
  `UnclosedQuote` / `UnfinishedEscape` / `IllegalEscape` errors, the macOS
  `escape_outside_quotes = true`, and the exact `unicodeescape`
  accumulator/overflow quirks.
- **Faithful adaptation**: the Zig labeled-`switch` `continue :loop .state` → a
  Rust `loop { match state { … } }` with a `state` variable; byte indexing over
  `input.as_bytes()` returning `&str` slices; the error set → `CommaSplitError`.
- **Deferred**: `parseAutoStruct` (its consumer) and `Theme::parse_cli`; the
  `theme` `Config::set` arm; the `loadCli` / file loader.
  `background-image-opacity` stays float-blocked.
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/comma_splitter.rs` (new): `CommaSplitError`,
   `CommaSplitter`.
2. `roastty/src/config/mod.rs`: add `mod comma_splitter;`.
3. Tests (in `comma_splitter.rs`, mirroring upstream's `splitter N`): `a,b,c` ⇒
   `a` / `b` / `c`; `""` ⇒ `None`; `a` ⇒ `a`; `\x5a` ⇒ `\x5a` (escape consumed,
   not decoded); `'a',b` ⇒ `'a'` / `b` (single quotes not special); `'a,b',c` ⇒
   `'a` / `b'` / `c`; `"a,b",c` ⇒ `"a,b"` / `c` (double quotes protect the
   comma); `a , b` ⇒ `a` / `b` (whitespace preserved); the error cases `\x` /
   `\x5` / `\u` / `\u{` ⇒ `UnfinishedEscape`, `\u{}` ⇒ `IllegalEscape`, an
   unclosed `"a` ⇒ `UnclosedQuote`.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty comma_split
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `CommaSplitter::next` reproduces upstream's state machine — comma splitting
  with double-quote protection, Zig escapes, the three error kinds, and the
  macOS `escape_outside_quotes = true` — including the exact `unicodeescape`
  quirks;
- the tests pass (the mirrored `splitter` cases + the error cases), and the
  existing tests still pass;
- `parseAutoStruct` / `Theme::parse_cli` and the loader stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the splitting diverges from upstream, an unrelated
item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Low** finding (folded into this experiment's tests): add targeted tests for
the unicode overflow/quirk and generic illegal escape — `\u{10ffff}` succeeds,
`\u{110000}` errors, `\u{afffff}` succeeds under the `d - 'a'` accumulator
quirk, and `\d` ⇒ `IllegalEscape` — to make those behaviors executable, not just
documented.

Codex found **no required issues**. It confirmed: replicating the unicode
accumulator quirk is the faithful choice (upstream uses `d - 'a'` / `d - 'A'`,
not `+10`, and the accumulator feeds only the `> 0x10ffff` guard, never
decoded/returned — `CommaSplitter.zig:175`/`:181`/`:187`); the overflow guard
runs after each digit in the `.digits` sub-state before looping
(`CommaSplitter.zig:195`); `&str` slices are safe (input is valid UTF-8 and
boundaries are only at the original start/end or ASCII comma positions,
multibyte content consumed byte-by-byte but never split mid-codepoint); and the
macOS `escape_outside_quotes = true` is correct (non-Windows; normal-state `\`
enters escape when the constant is true — `CommaSplitter.zig:23`/`:88`).

Review artifacts:

- Prompt: `logs/codex-review/20260604-183504-d524-prompt.md` (design)
- Result: `logs/codex-review/20260604-183504-d524-last-message.md` (design)

## Result

**Result:** Pass

The new module `roastty/src/config/comma_splitter.rs` (declared
`mod comma_splitter;`) ports upstream `cli/CommaSplitter`: `CommaSplitError` and
`CommaSplitter` (`new` + `next`), the full state machine as a Rust
`loop { match state { … } }` over `input.as_bytes()` returning `&str` slices,
with `ESCAPE_OUTSIDE_QUOTES = true` (macOS) and the exact `unicodeescape`
accumulator/overflow quirks. Three tests cover the basic splitter behavior
(mirroring upstream `splitter 1`–`8`), the error cases (`\x` / `\x5` / `\u` /
`\u{` ⇒ `UnfinishedEscape`; `\u{}` / `\d` ⇒ `IllegalEscape`; `"a` ⇒
`UnclosedQuote`), and the folded unicode overflow/quirk cases (`\u{10ffff}` /
`\u{afffff}` succeed, `\u{110000}` ⇒ `IllegalEscape`).

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3012 passed, 0 failed (three new tests; no
  regressions).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + lib.rs/header/abi_harness.c)
  clean; `git diff --check` clean.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no
findings**: the completed slice matches the approved design — the state-machine
shape, error categories, raw-slice return behavior, macOS
`ESCAPE_OUTSIDE_QUOTES = true`, last-state restoration after escapes, and the
unicode accumulator quirk/overflow guard are all faithful to upstream; the
folded tests cover the main splitter behavior plus the edge cases
(unfinished/illegal escapes, unclosed quotes, unicode max/overflow, the `a-f`
quirk); gates are clean and the consumer/parser work remains deferred. "Approved
with no findings."

Review artifacts:

- Prompt: `logs/codex-review/20260604-183808-r524-prompt.md` (result)
- Result: `logs/codex-review/20260604-183808-r524-last-message.md` (result)

## Conclusion

`CommaSplitter` — the quote/escape-aware comma iterator — is ported. The next
building block is **`parseAutoStruct`** (the colon-keyed `key:value` comma-list
parser that drives `CommaSplitter`, decodes double-quoted values, and tracks
required fields), then `Theme::parse_cli` (its light/dark-pair branch) and the
`theme` `Config::set` arm — the last parseable field. Then the `loadCli` /
config-file loader splits `key = value` lines and drives `Config::set`.
