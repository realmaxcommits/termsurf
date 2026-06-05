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

# Experiment 567: magic-byte file-type detection (FileType)

## Description

This experiment ports upstream `file_type.zig` — `FileType`, a helper that
detects image file types from their leading **magic bytes** (or guesses from a
file extension). It supports JPEG, PNG, GIF, BMP, QOI, and WebP. roastty's
`terminal/kitty/graphics_image.rs` handles kitty-graphics transmission formats
and PNG decoding but has **no** magic-byte detector, so this port is
non-duplicating. Upstream homes it at the crate root (`src/file_type.zig`), so
roastty's lands at the crate root too: `crate::file_type`.

## Upstream behavior

`file_type.zig` holds a table of `{ typ, sigs, exts }` entries. Each signature
is a `[]const ?u8` where a `null` byte is a **wildcard** (matches any byte);
`exts` are file extensions.

```zig
pub const FileType = enum {
    jpeg, png, gif, bmp, qoi, webp, unknown,

    pub fn detect(contents: []const u8) FileType {
        inline for (type_details) |typ| {
            inline for (typ.sigs) |signature| {
                if (contents.len >= signature.len) {
                    for (contents[0..signature.len], signature) |f, sig| {
                        if (sig) |s| if (f != s) break;   // concrete byte mismatch → next sig
                    } else {
                        return typ.typ;                   // all bytes matched (wildcards skipped)
                    }
                }
            }
        }
        return .unknown;
    }

    pub fn guessFromExtension(extension: []const u8) FileType {
        inline for (type_details) |typ| {
            inline for (typ.exts) |ext| {
                if (std.ascii.eqlIgnoreCase(extension, ext)) return typ.typ;
            }
        }
        return .unknown;
    }
};
```

The signature table (order matters — the first match wins):

| Type | Signatures (hex; `··` = wildcard)                                                                                             | Extensions             |
| ---- | ----------------------------------------------------------------------------------------------------------------------------- | ---------------------- |
| jpeg | `FF D8 FF DB` · `FF D8 FF E0 00 10 4A 46 49 46 00 01` · `FF D8 FF EE` · `FF D8 FF E1 ·· ·· 45 78 69 66 00 00` · `FF D8 FF E0` | `.jpg` `.jpeg` `.jfif` |
| png  | `89 50 4E 47 0D 0A 1A 0A`                                                                                                     | `.png`                 |
| gif  | `GIF87a` · `GIF89a`                                                                                                           | `.gif`                 |
| bmp  | `BM`                                                                                                                          | `.bmp`                 |
| qoi  | `qoif`                                                                                                                        | `.qoi`                 |
| webp | `52 49 46 46 ·· ·· ·· ·· 57 45 42 50` (`RIFF····WEBP`)                                                                        | `.webp`                |

`detect` returns the first type whose signature is a prefix-match of `contents`
(with wildcards); a signature longer than `contents` is skipped.
`guessFromExtension` is a case-insensitive extension lookup. Both fall back to
`unknown`.

## Rust mapping (`roastty/src/file_type.rs`)

A direct transcription. The wildcard `?u8` becomes `Option<u8>` (`None` =
wildcard); the table is a `&'static [TypeDetails]`. ASCII signature bytes (e.g.
`'G'`) become `Some(b'G')`.

```rust
//! File-type detection from magic bytes (port of upstream `file_type`).
//!
//! Ref: https://en.wikipedia.org/wiki/List_of_file_signatures

/// A detected file type (upstream `FileType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileType {
    Jpeg,
    Png,
    Gif,
    Bmp,
    Qoi,
    Webp,
    Unknown,
}

struct TypeDetails {
    typ: FileType,
    /// Magic-byte signatures; `None` is a wildcard (matches any byte).
    sigs: &'static [&'static [Option<u8>]],
    exts: &'static [&'static str],
}

// Signature order matters: the first matching type wins.
const TYPE_DETAILS: &[TypeDetails] = &[
    TypeDetails {
        typ: FileType::Jpeg,
        sigs: &[
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xDB)],
            &[
                Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE0), Some(0x00), Some(0x10),
                Some(0x4A), Some(0x46), Some(0x49), Some(0x46), Some(0x00), Some(0x01),
            ],
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xEE)],
            &[
                Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE1), None, None,
                Some(0x45), Some(0x78), Some(0x69), Some(0x66), Some(0x00), Some(0x00),
            ],
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE0)],
        ],
        exts: &[".jpg", ".jpeg", ".jfif"],
    },
    TypeDetails {
        typ: FileType::Png,
        sigs: &[&[
            Some(0x89), Some(0x50), Some(0x4E), Some(0x47),
            Some(0x0D), Some(0x0A), Some(0x1A), Some(0x0A),
        ]],
        exts: &[".png"],
    },
    TypeDetails {
        typ: FileType::Gif,
        sigs: &[
            &[Some(b'G'), Some(b'I'), Some(b'F'), Some(b'8'), Some(b'7'), Some(b'a')],
            &[Some(b'G'), Some(b'I'), Some(b'F'), Some(b'8'), Some(b'9'), Some(b'a')],
        ],
        exts: &[".gif"],
    },
    TypeDetails {
        typ: FileType::Bmp,
        sigs: &[&[Some(b'B'), Some(b'M')]],
        exts: &[".bmp"],
    },
    TypeDetails {
        typ: FileType::Qoi,
        sigs: &[&[Some(b'q'), Some(b'o'), Some(b'i'), Some(b'f')]],
        exts: &[".qoi"],
    },
    TypeDetails {
        typ: FileType::Webp,
        sigs: &[&[
            Some(0x52), Some(0x49), Some(0x46), Some(0x46), None, None, None, None,
            Some(0x57), Some(0x45), Some(0x42), Some(0x50),
        ]],
        exts: &[".webp"],
    },
];

impl FileType {
    /// Detect the file type from the leading magic bytes (upstream `detect`).
    pub(crate) fn detect(contents: &[u8]) -> FileType {
        for td in TYPE_DETAILS {
            for sig in td.sigs {
                if contents.len() >= sig.len()
                    && contents.iter().zip(sig.iter()).all(|(&f, s)| match s {
                        Some(b) => f == *b,
                        None => true, // wildcard
                    })
                {
                    return td.typ;
                }
            }
        }
        FileType::Unknown
    }

    /// Guess the file type from an extension, case-insensitively (upstream `guessFromExtension`).
    ///
    /// Byte-oriented to match upstream's `[]const u8`: a non-UTF-8 extension naturally returns
    /// `Unknown` rather than being unrepresentable.
    pub(crate) fn guess_from_extension(extension: &[u8]) -> FileType {
        for td in TYPE_DETAILS {
            for ext in td.exts {
                if extension.eq_ignore_ascii_case(ext.as_bytes()) {
                    return td.typ;
                }
            }
        }
        FileType::Unknown
    }
}
```

Because `contents.len() >= sig.len()`, `contents.iter().zip(sig.iter())` yields
exactly `sig.len()` pairs — the faithful equivalent of upstream's
`contents[0..signature.len]` slice.

## Scope / faithfulness notes

- **Ported (1:1)**: `file_type.FileType` → `crate::file_type::FileType`
  (`detect`, `guess_from_extension`, the enum, and the verbatim
  signature/extension table).
- **Faithful**: the signature table (bytes, wildcards, and **order**), the
  prefix-match-with- wildcards detection (longer-than-content signatures
  skipped), the case-insensitive extension lookup, and the `Unknown` fallback
  are all reproduced exactly. `guess_from_extension` takes `&[u8]` (mirroring
  upstream's `[]const u8`) and compares with `<[u8]>::eq_ignore_ascii_case`
  against each `ext.as_bytes()`, mirroring `std.ascii.eqlIgnoreCase`; a
  non-UTF-8 extension naturally returns `Unknown`.
- **Faithful adaptation**: the wildcard `?u8` → `Option<u8>` (`None` =
  wildcard); ASCII signature bytes → `Some(b'…')`; the `inline for` comptime
  table walk → a runtime walk over a `&'static [TypeDetails]` (identical
  results).
- **Deferred**: nothing — the upstream file is fully covered.
- No C ABI/header/ABI-inventory change (internal Rust). Adds a crate-root
  `file_type` module.

## Changes

1. `roastty/src/file_type.rs` (new): `FileType`, `TypeDetails`, `TYPE_DETAILS`,
   `detect`, `guess_from_extension`.
2. `roastty/src/lib.rs`: add `#[allow(dead_code)] mod file_type;` (alphabetical,
   after `config`).
3. Tests (in `file_type.rs`):
   - **each format detected** from its magic bytes (jpeg variants, png, gif both
     versions, bmp, qoi, webp).
   - **wildcards honored**: the EXIF jpeg signature
     (`FF D8 FF E1 ?? ?? 45 78 69 66 00 00`) and the webp signature
     (`RIFF ???? WEBP`) detect regardless of the wildcard bytes.
   - **signature ordering**: a bare `FF D8 FF E0` (4 bytes) detects jpeg
     (matching the short trailing signature, not requiring the longer JFIF one).
   - **too-short content** (e.g. `FF D8`) → `Unknown`; **unrelated bytes** →
     `Unknown`.
   - **extension guessing**: `b".png"`, `b".JPG"`/`b".Jpeg"` (case-insensitive)
     → the right type; an unknown extension → `Unknown`.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty file_type
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/file_type.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `FileType::detect` matches the verbatim signature table (bytes, wildcards,
  order) as a prefix-match (skipping signatures longer than the content) and
  `guess_from_extension` does a case-insensitive extension lookup, both falling
  back to `Unknown` — faithful to `file_type.zig`;
- the tests pass (each format / wildcards / ordering / too-short / unknown /
  extensions), and the existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the signature table, the wildcard/prefix matching,
the ordering, or the extension lookup diverges from upstream, an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed the design and found **one Required** finding (fixed):

- **Required (fixed)**: `guess_from_extension` should be byte-oriented
  (`&[u8]`), not `&str` — upstream accepts `[]const u8`, and
  `str::eq_ignore_ascii_case` is equivalent only for already- valid UTF-8.
  Changed to `extension: &[u8]` compared with `<[u8]>::eq_ignore_ascii_case`
  against each `ext.as_bytes()`, so a non-UTF-8 extension naturally returns
  `Unknown` instead of being unrepresentable.

Codex mechanically compared the signature table and confirmed the **type order,
extension order, all signature bytes, the wildcard positions, and the JPEG/WebP
ordering all match upstream**, and that the `detect` prefix/wildcard logic using
`zip` is faithful thanks to the `contents.len() >= sig.len()` guard.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d567-prompt.md`
- Result: `logs/codex-review/20260604-d567-last-message.md`
