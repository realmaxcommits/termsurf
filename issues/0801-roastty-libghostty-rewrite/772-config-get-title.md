+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 772: Config Get Title

## Description

Wire the optional `title` config key into Roastty's aggregate config and the
public `roastty_config_get` C ABI.

Upstream stores `title` as `?[:0]const u8` with a default of `null`. Its C ABI
getter returns `true` for the key even when the value is unset, writing a null
pointer into the caller's output slot. Roastty currently mirrors only that
default by writing a hard-coded null pointer from `roastty_config_get`, and
`config::Config` has no aggregate title field.

This experiment only ports config parsing, formatting, storage, stable C string
lookup, and the ABI getter. It does not wire runtime window-title behavior or
surface title update policy.

## Changes

- `roastty/src/config/mod.rs`
  - Add `title: Option<String>` to `config::Config`.
  - Default it to `None`, matching upstream `null`.
  - Include `title` in `format_config` after `fullscreen`, preserving the
    currently implemented upstream order among available fields.
  - Update the full key-order test to assert `fullscreen`, `title`, then
    `window-padding-color`.
  - Route `Config::set("title", ...)` through the existing optional value helper
    and `parse_string_field`: missing values are `ValueRequired`, empty values
    reset to `None`, and non-empty values become `Some(String)`.
  - Reject interior NUL bytes in parsed title values as `InvalidValue`, because
    the public ABI exposes title as a C string pointer.
  - Add aggregate tests for defaults, formatting, set routing, quoted-space
    values, empty reset, missing values, interior NUL rejection, and
    clone/partial-eq behavior.
- `roastty/src/lib.rs`
  - Store a cached `CString` for the parsed config title in `ConfigHandle`, so
    `roastty_config_get("title")` can return a stable pointer with the same
    lifetime model as the config handle.
  - Rebuild that cached C string from `parsed.title` in one central helper used
    by `Config::sync_from_parsed_config`, and initialize it for
    `roastty_config_new` and `roastty_config_clone`, so all current and future
    parsed-config sync paths keep the pointer fresh.
  - Make `roastty_config_get("title")` write a null pointer when the parsed
    title is `None`, or the cached C string pointer when it is `Some`.
  - Add C ABI tests proving the key returns `true` with null by default,
    reflects file-loaded, CLI-loaded, cloned, and reset-to-default values, and
    reports missing CLI values as diagnostics without changing the default.
  - Add C ABI tests proving `title = ""` and `title =` reset to null, repeated
    getter calls return a pointer valid while the handle lives, updating from
    `Some(title)` back to `None` clears the cached pointer, clones with
    `Some(title)` return independent valid title pointers, and interior NUL file
    values report `InvalidValue` without caching a partial title.

## Verification

- `cargo test -p roastty config_title -- --nocapture --test-threads=1`
- `cargo test -p roastty config_get_title -- --nocapture --test-threads=1`
- `cargo test -p roastty config_ -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

The experiment passes if `title` is stored in aggregate config, can be set
through file and CLI loading, formats in full config output in upstream order
among implemented fields, resets to `None` on an empty value, reports missing
values consistently with upstream optional strings, and is returned by
`roastty_config_get` from parsed state with a pointer that remains stable until
the config handle is freed or parsed config state is mutated and synced.

## Design Review

Codex reviewed the design and found one blocking issue: the original plan did
not specify how to handle interior NUL bytes even though the ABI cache uses
`CString`. The plan now rejects interior NUL title values as `InvalidValue`
during config parsing and includes file-loaded interior-NUL coverage.

The review also asked for central cache refresh semantics and exact formatter
order coverage. The plan now requires a single cache rebuild path tied to
`Config::sync_from_parsed_config`, initialization for new and cloned handles,
and a full key-order assertion for `fullscreen`, `title`,
`window-padding-color`.

## Result

**Result:** Pass

Implemented aggregate config storage for `title`. `config::Config` now stores
`title: Option<String>`, defaults it to `None`, formats it between `fullscreen`
and `window-padding-color`, routes `Config::set("title", ...)` through optional
string parsing/reset behavior, and rejects interior NUL values as `InvalidValue`
because the public ABI exposes title as a C string.

`ConfigHandle` now caches a `CString` derived from parsed title state. The cache
is rebuilt by `Config::sync_from_parsed_config`, initialized on clone, and read
by `roastty_config_get("title")`, which now writes a null pointer for `None` or
a stable pointer for `Some(title)` until the config handle is freed or parsed
config state is mutated and synced.

Verification passed:

- `cargo test -p roastty config_title -- --nocapture --test-threads=1`
- `cargo test -p roastty config_get_title -- --nocapture --test-threads=1`
- `cargo test -p roastty config_ -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

`title` now reports parsed optional string config state through the C ABI while
preserving upstream's `true` plus null-pointer behavior for the unset default.
Runtime window-title behavior and surface title update policy remain follow-up
work.

## Completion Review

Codex reviewed the completed implementation and found no blocking code
correctness issues. The review confirmed that title storage, formatting, setter
routing, interior-NUL rejection, cached parsed-state lookup, and focused tests
cover the experiment scope.

The review found two process/documentation issues before commit: add
`[review.design]` and `[review.result]` provenance frontmatter to match the
README status, and clarify that returned title pointers are stable only until
the config handle is freed or parsed config state is mutated and synced. Both
documentation fixes were applied before the result commit.
