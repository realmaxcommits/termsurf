# Experiment 4: Embedded ABI App Bridge Audit

## Description

The copied macOS app depends on Ghostty's embedded C ABI. Roastty has a larger C
ABI than Ghostty because it also exposes reusable terminal/runtime pieces, so a
raw full-header symbol diff is not a meaningful parity check. The meaningful
first source audit slice is the app-facing embedded ABI: every upstream
`ghostty_*` symbol and type used by the pinned macOS Swift app must have the
renamed `roastty_*` equivalent used by the Roastty Swift app, and every required
function must be exported by `roastty/include/roastty.h` and implemented in
`roastty/src/lib.rs`.

This experiment creates a reproducible inventory for that app-facing ABI slice,
records source-audit rows for the outcome, and fixes only concrete gaps found in
that slice.

## Changes

Planned changes:

- `issues/0805-roastty-ghostty-parity/04-embedded-abi-app-bridge-audit.md`
  - Record the plan, review, commands, result, and conclusion.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add Experiment 4 to the issue index with status `Designed`.
  - Add a learning if the audit establishes a reusable ABI-checking rule.
- `issues/0805-roastty-ghostty-parity/source-audit.md`
  - Add rows for app-facing embedded ABI symbol coverage and Swift call-site
    prefix coverage.
- `issues/0805-roastty-ghostty-parity/feature-matrix.md` or
  `walkthrough-matrix.md`
  - Add rows only if the audit proves a user-visible app behavior or app
    walkthrough guard.
- `issues/0805-roastty-ghostty-parity/abi-app-symbols.md`
  - Create a durable inventory artifact listing:
    - upstream `ghostty_*` function symbols declared in
      `vendor/ghostty/include/ghostty.h`;
    - `roastty_*` function symbols declared in `roastty/include/roastty.h`;
    - upstream Swift `ghostty_*` and `GHOSTTY_*` symbol references under
      `vendor/ghostty/macos/Sources`;
    - Roastty Swift `roastty_*` and `ROASTTY_*` symbol references under
      `roastty/macos/Sources`;
    - Swift-used functions, typedefs, structs, enums, and constants classified
      by declaration kind;
    - app-facing struct field-order and enum/constant numeric-value checks where
      the Swift app depends on ABI shape;
    - mapped missing/extra symbols for the app-facing subset;
    - any accepted extra Roastty symbols that are outside the embedded app ABI.

Possible code changes, only if required by the audit:

- `roastty/include/roastty.h`
- `roastty/src/lib.rs`
- `roastty/macos/Sources/**`

Do not change `vendor/ghostty/`. Do not attempt a full terminal/config/source
audit in this experiment.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Initial verdict: **Changes required**.

Required findings and fixes:

- The Swift symbol comparison used path-prefixed `rg -o` output, which would
  make `comm` compare source paths instead of symbol names. Fixed by using
  `rg --no-filename -o` for both Ghostty and Roastty Swift symbol scans.
- The implementation-export check matched arbitrary `roastty_*` text in
  `roastty/src/lib.rs`. Fixed by extracting only
  `#[no_mangle] pub extern "C" fn roastty_*` definitions for the app-facing
  implementation check.
- The inventory plan covered functions but not Swift-used types and constants,
  despite the experiment claiming to audit symbols and types. Fixed by adding a
  Swift-used identifier declaration pass and requiring `abi-app-symbols.md` to
  classify functions, typedefs, structs, enums, enum values, macros, and other
  constants. Swift-used app-facing structs/enums/constants must compare field
  names/order and numeric values or record a classified non-pass row.

Re-review verdict: **Approved**. The reviewer confirmed all three required
findings were resolved and no new required findings were introduced.

## Verification

Run from the repo root. Save useful command transcripts under `logs/` with the
prefix `issue805-exp4-`.

### 1. Extract Header Function Symbols

Commands:

```bash
perl -ne 'while(/\b(ghostty_[A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' \
  vendor/ghostty/include/ghostty.h | sort -u \
  > /tmp/issue805-exp4-ghostty-header-fns.txt

perl -ne 'while(/\b(roastty_[A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' \
  roastty/include/roastty.h | sort -u \
  > /tmp/issue805-exp4-roastty-header-fns.txt

sed 's/^ghostty_/roastty_/' /tmp/issue805-exp4-ghostty-header-fns.txt \
  > /tmp/issue805-exp4-ghostty-header-fns-mapped.txt

comm -23 /tmp/issue805-exp4-ghostty-header-fns-mapped.txt \
  /tmp/issue805-exp4-roastty-header-fns.txt
```

Pass criteria:

- Every upstream header function required by the macOS embedded app has a
  declared Roastty equivalent, or is recorded as a `Gap`,
  `Intentional divergence`, or `Not applicable` row with evidence.
- Extra Roastty functions are not failures unless the copied app incorrectly
  depends on behavior that should be Ghostty-equivalent.

### 2. Extract Swift App Symbol References

Commands:

```bash
rg --no-filename -o '(ghostty|GHOSTTY)_[A-Za-z0-9_]+' \
  vendor/ghostty/macos/Sources | sort -u \
  > /tmp/issue805-exp4-ghostty-swift-symbols.txt

rg --no-filename -o '(roastty|ROASTTY)_[A-Za-z0-9_]+' \
  roastty/macos/Sources | sort -u \
  > /tmp/issue805-exp4-roastty-swift-symbols.txt

sed -e 's/^ghostty_/roastty_/' -e 's/^GHOSTTY_/ROASTTY_/' \
  /tmp/issue805-exp4-ghostty-swift-symbols.txt \
  > /tmp/issue805-exp4-ghostty-swift-symbols-mapped.txt

comm -23 /tmp/issue805-exp4-ghostty-swift-symbols-mapped.txt \
  /tmp/issue805-exp4-roastty-swift-symbols.txt

comm -13 /tmp/issue805-exp4-ghostty-swift-symbols-mapped.txt \
  /tmp/issue805-exp4-roastty-swift-symbols.txt
```

Pass criteria:

- Missing mapped Roastty Swift references are explained by deliberate file
  removal, platform scope, or a recorded source-audit gap.
- Extra Roastty Swift references are explained by app renaming or Roastty-only
  support code, or are recorded as gaps.
- No copied app-facing Swift file still calls or types against `ghostty_*` or
  `GHOSTTY_*` symbols.

### 3. Confirm Swift-Used Type and Constant Declarations

Commands:

```bash
rg --no-filename -o '(ghostty|GHOSTTY)_[A-Za-z0-9_]+' \
  vendor/ghostty/macos/Sources | sort -u |
  sed -e 's/^ghostty_/roastty_/' -e 's/^GHOSTTY_/ROASTTY_/' \
  > /tmp/issue805-exp4-ghostty-swift-symbols-mapped.txt

rg --no-filename -o '(roastty|ROASTTY)_[A-Za-z0-9_]+' \
  roastty/include/roastty.h | sort -u \
  > /tmp/issue805-exp4-roastty-header-identifiers.txt

comm -23 /tmp/issue805-exp4-ghostty-swift-symbols-mapped.txt \
  /tmp/issue805-exp4-roastty-header-identifiers.txt
```

Pass criteria:

- Every Swift-used upstream ABI identifier has a mapped Roastty declaration in
  `roastty/include/roastty.h`, or is recorded as a `Gap`,
  `Intentional divergence`, or `Not applicable` row with evidence.
- `abi-app-symbols.md` classifies each Swift-used identifier as a function,
  typedef, struct, enum, enum value, macro, or other constant.
- For Swift-used app-facing structs, enums, and constants, the inventory
  compares the upstream declaration to the Roastty declaration after
  `ghostty`/`GHOSTTY` prefix normalization. Field names/order and numeric values
  must match, or the difference must be recorded as a `Gap`,
  `Intentional divergence`, or `Not applicable` row.

### 4. Confirm Implementation Exports

Commands:

```bash
perl -ne 'while(/\b(roastty_[A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' \
  roastty/include/roastty.h | sort -u \
  > /tmp/issue805-exp4-roastty-header-fns.txt

perl -0ne 'while(/#\[no_mangle\]\s*pub\s+extern\s+"C"\s+fn\s+(roastty_[A-Za-z0-9_]+)/g){print "$1\n"}' \
  roastty/src/lib.rs | sort -u \
  > /tmp/issue805-exp4-roastty-exported-fns.txt

comm -23 /tmp/issue805-exp4-roastty-header-fns.txt \
  /tmp/issue805-exp4-roastty-exported-fns.txt
```

Pass criteria:

- Every app-facing Roastty header function has a
  `#[no_mangle] pub extern "C" fn` implementation in `roastty/src/lib.rs`, or a
  documented reason why it is intentionally declared elsewhere.

### 5. Build and Test

Commands:

```bash
cargo test -p roastty
scripts/roastty-app/build-macos-app.sh Debug
git diff --check
git status --short
```

Pass criteria:

- Rust tests pass if Roastty source changes are made.
- The debug Roastty app builds.
- Markdown is formatted.
- `git diff --check` passes.
- Only Experiment 4's planned docs and any concrete ABI/app-bridge fixes are
  changed.

Overall result:

- **Pass** if the app-facing embedded ABI has a complete mapped inventory, every
  required mapped symbol is present and implemented or documented with an
  accepted status, and any concrete gaps found in this slice are fixed and
  verified.
- **Partial** if the inventory is complete but unresolved `Gap` rows remain.
- **Fail** if the inventory cannot be reproduced or the audit leaves missing
  symbols unclassified.
