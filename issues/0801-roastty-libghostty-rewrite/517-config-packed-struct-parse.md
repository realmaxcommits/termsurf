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

# Experiment 517: the packed-struct flag parser (parse_packed_flags + ScrollToBottom / FontShapingBreak parse_cli)

## Description

Continuing the config loader (Experiments 513–516, which finished the plain-enum
`from_keyword` sweep), this experiment ports the **packed-struct** parse — the
inverse of the `entry_flags` formatter (Experiment 499). Upstream parses a
packed-struct-of-bools field via `cli.args.parsePackedStruct`: a standalone
boolean sets every flag, otherwise a comma-list of `[no-]flag` keywords sets the
named flags. This experiment adds a reusable `parse_packed_flags` helper and
applies it as `parse_cli` to `ScrollToBottom` and `FontShapingBreak` (the two
packed structs that already have `entry_flags`).

## Upstream behavior

`parsePackedStruct` (`cli/args.zig:608`):

```zig
var result: T = .{};                    // start from struct defaults
bools: {                                // standalone bool sets ALL fields
    const b = parseBool(v) catch break :bools;
    inline for (info.fields) |field| @field(result, field.name) = b;
    return result;
}
var iter = std.mem.splitSequence(u8, v, ",");   // else comma-list
loop: while (iter.next()) |part_raw| {
    const part, const value = part: {
        const trimmed = std.mem.trim(u8, part_raw, whitespace);   // whitespace = " \t"
        if (std.mem.startsWith(u8, trimmed, "no-"))
            break :part .{ trimmed["no-".len..], false }
        else
            break :part .{ trimmed, true };
    };
    inline for (info.fields) |field| {
        if (std.mem.eql(u8, field.name, part)) { @field(result, field.name) = value; continue :loop; }
    }
    return error.InvalidValue;          // no field matched
}
return result;
```

Key points:

- The result starts from the struct **defaults**; fields not named in the list
  keep their defaults.
- The **standalone-bool** shortcut runs `parseBool` on the **raw** value
  (untrimmed) — `1` / `t` / `T` / `true` / `0` / `f` / `F` / `false` — and sets
  every field to that bool.
- Otherwise each comma part is trimmed of `" \t"`, an optional `no-` prefix
  means `false` (else `true`), and the remaining name must match a field — an
  unknown name is `error.InvalidValue`.

roastty's existing `parse_bool` already matches `parseBool` exactly (Experiment
482), and `whitespace` is `" \t"`.

## Rust mapping (`roastty/src/config/mod.rs`)

A shared helper (single closure, to avoid double-borrowing the result) drives
the parse; each packed struct supplies a closure mapping a token to a field:

```rust
pub(crate) enum FlagsParseError {
    InvalidValue,
}

enum FlagToken<'a> {
    All(bool),
    One(&'a str, bool),
}

/// Parse a packed-struct bool-flag value (upstream `cli.args.parsePackedStruct`):
/// a standalone bool yields `All(b)`; otherwise each comma part yields
/// `One(name, on)` (`no-` ⇒ `on = false`). `apply` sets the flag(s) and returns
/// `false` for an unknown name (upstream's `error.InvalidValue`).
fn parse_packed_flags(
    value: &str,
    mut apply: impl FnMut(FlagToken) -> bool,
) -> Result<(), FlagsParseError> {
    if let Some(b) = parse_bool(value) {
        apply(FlagToken::All(b));
        return Ok(());
    }
    for part in value.split(',') {
        let trimmed = part.trim_matches(|c| c == ' ' || c == '\t');
        let (name, on) = match trimmed.strip_prefix("no-") {
            Some(rest) => (rest, false),
            None => (trimmed, true),
        };
        if !apply(FlagToken::One(name, on)) {
            return Err(FlagsParseError::InvalidValue);
        }
    }
    Ok(())
}

impl ScrollToBottom {
    pub(crate) fn parse_cli(value: &str) -> Result<Self, FlagsParseError> {
        let mut result = ScrollToBottom::default();
        parse_packed_flags(value, |tok| match tok {
            FlagToken::All(b) => {
                result.keystroke = b;
                result.output = b;
                true
            }
            FlagToken::One("keystroke", on) => {
                result.keystroke = on;
                true
            }
            FlagToken::One("output", on) => {
                result.output = on;
                true
            }
            FlagToken::One(_, _) => false,
        })?;
        Ok(result)
    }
}

impl FontShapingBreak {
    pub(crate) fn parse_cli(value: &str) -> Result<Self, FlagsParseError> {
        let mut result = FontShapingBreak::default();
        parse_packed_flags(value, |tok| match tok {
            FlagToken::All(b) => {
                result.cursor = b;
                true
            }
            FlagToken::One("cursor", on) => {
                result.cursor = on;
                true
            }
            FlagToken::One(_, _) => false,
        })?;
        Ok(result)
    }
}
```

The `FlagToken::All` arm always returns `true` (it can't fail); the `One(_, _)`
catch-all returns `false`, which the helper turns into `InvalidValue` — exactly
upstream's "no field matched" error. Unmentioned fields keep the `Default`
values (upstream's `result: T = .{}`).

## Scope / faithfulness notes

- **Ported (bridged)**: `parsePackedStruct`, as the reusable
  `parse_packed_flags` helper, applied to `ScrollToBottom` and
  `FontShapingBreak` `parse_cli`.
- **Faithful**: the standalone-bool shortcut (raw value, `parse_bool`), the
  `[no-]flag` comma-list with `" \t"` trimming, defaults for unmentioned fields,
  and `InvalidValue` for an unknown flag — exactly upstream.
- **Faithful adaptation**: the comptime `inline for (fields)` field match → an
  explicit per-struct closure; `error.InvalidValue` →
  `FlagsParseError::InvalidValue`.
- **Deferred**: `parse_cli` for the other packed structs
  (`ShellIntegrationFeatures`, `NotifyOnCommandFinishAction`); the bool / int /
  float / string magic paths; the empty-string reset rule; the per-field
  `parseIntoField` dispatch and the `loadCli` / file loader.
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`: add `FlagsParseError`, the `FlagToken` enum, the
   `parse_packed_flags` helper, and `parse_cli` for `ScrollToBottom` and
   `FontShapingBreak`.
2. Tests (in `config/mod.rs`): standalone `true` / `false` / `1` / `0` set all
   flags; `no-cursor` / `keystroke,no-output` set named flags with defaults for
   the rest; round-trips with `format_entry`/`entry_flags`; an unknown flag is
   `Err(InvalidValue)`.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty packed_flags
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `parse_packed_flags` + the two `parse_cli` match upstream `parsePackedStruct`:
  standalone bool sets all flags, `[no-]flag` comma-list sets named flags with
  defaults for the rest, unknown flag → `InvalidValue`;
- the tests pass (standalone, comma-list, no- prefix, unknown, round-trip), and
  the existing tests still pass;
- the remaining packed structs and loader pieces stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the parse diverges from upstream, an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the design matches upstream `parsePackedStruct` — start
from struct defaults, try `parseBool(v)` on the raw untrimmed value, then
comma-split, `" \t"` trim, optional `no-` stripping, and exact field-name
matching (`args.zig:607`/`:616`/`:624`/`:631`), with the trim set exactly
`whitespace = " \t"` (`args.zig:19`). `ScrollToBottom` and `FontShapingBreak`
are packed bool structs with the proposed fields/defaults
(`Config.zig:10206`/`:8563`). The single-closure `FlagToken` design is a
faithful adaptation of Zig's comptime field loop — `All` cannot fail, and an
unknown `One` mapping to `InvalidValue` matches upstream — and the planned tests
cover the observable behavior (standalone bools, defaults for omitted flags,
`no-`, unknown names, formatter round-trips).

Review artifacts:

- Prompt: `logs/codex-review/20260604-171804-d517-prompt.md` (design)
- Result: `logs/codex-review/20260604-171804-d517-last-message.md` (design)

## Result

**Result:** Pass

The packed-struct parse was implemented: `FlagsParseError`, the `FlagToken`
enum, the `parse_packed_flags` helper, and `parse_cli` for `ScrollToBottom` and
`FontShapingBreak`. A standalone bool (`parse_bool` on the raw value) sets every
flag; otherwise each `[no-]flag` comma part (trimmed of `" \t"`) sets a named
flag, with `Default` values for the rest, and an unknown flag returns
`FlagsParseError::InvalidValue`. The new test `packed_flags_parse_cli` covers
standalone bools, the comma-list with `no-` and whitespace, omitted-flag
defaults, the unknown-flag error, both structs, and a `format_entry` →
`parse_cli` round-trip.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3003 passed, 0 failed (one new test; no regressions).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + lib.rs/header/abi_harness.c)
  clean; `git diff --check` clean.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no
findings**: the implementation matches upstream `parsePackedStruct` — raw
`parseBool` shortcut, comma-list fallback, `" \t"` trimming, `no-` negation
without re-trimming, exact field matching, defaults for omitted fields, and
`InvalidValue` for unknown flags; the test coverage is adequate (all-flags
bools, omitted-default behavior, whitespace, negation, errors, both target
structs, formatter round-trips); the gates are clean and the deferred items
stayed out of scope. "Approved with no findings."

Review artifacts:

- Prompt: `logs/codex-review/20260604-172109-r517-prompt.md` (result)
- Result: `logs/codex-review/20260604-172109-r517-last-message.md` (result)

## Conclusion

The packed-struct flag parser (`parse_packed_flags`) is the parse inverse of
`entry_flags`, now applied to `ScrollToBottom` and `FontShapingBreak`. The
remaining loader work is: `parse_cli` for the other two packed structs
(`ShellIntegrationFeatures`, `NotifyOnCommandFinishAction`) via the same helper;
the bool / int / string "magic" parse paths (float stays blocked); the
empty-string reset-to-default rule; and the per-field `parseIntoField` dispatch
(`Config::set(key, value)`) + the `loadCli` / file loader.
