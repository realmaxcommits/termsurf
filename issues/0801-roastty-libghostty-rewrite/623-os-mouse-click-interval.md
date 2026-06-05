+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 623: os mouse double-click interval

## Description

Port `os/mouse.zig` into `roastty/src/os/mouse.rs`: the macOS system
double-click interval helper. This gives later mouse selection and surface input
slices the same system-configured timing value upstream uses.

The upstream behavior is intentionally tiny: on macOS, ask `NSEvent` for
`doubleClickInterval`, convert seconds to milliseconds with `ceil`, and return a
`u32`; on other platforms return `null`. Roastty is macOS-only in product scope,
but a non-macOS `None` fallback keeps host tests/builds straightforward without
making a cross-platform behavior surface.

## Upstream behavior (`os/mouse.zig`)

```zig
pub fn clickInterval() ?u32 {
    return switch (builtin.os.tag) {
        .macos => macos: {
            const NSEvent = objc.getClass("NSEvent") orelse {
                log.err("NSEvent class not found. Can't get click interval.", .{});
                return null;
            };

            const interval = NSEvent.msgSend(f64, objc.sel("doubleClickInterval"), .{});
            const ms = @as(u32, @intFromFloat(@ceil(interval * 1000)));
            break :macos ms;
        },
        else => null,
    };
}
```

## Rust mapping (`roastty/src/os/mouse.rs`)

```rust
pub(crate) fn click_interval() -> Option<u32> {
    system_double_click_interval_seconds().map(seconds_to_millis_ceil)
}

#[cfg(target_os = "macos")]
fn system_double_click_interval_seconds() -> Option<f64> {
    Some(unsafe { objc2_app_kit::NSEvent::doubleClickInterval() })
}

#[cfg(not(target_os = "macos"))]
fn system_double_click_interval_seconds() -> Option<f64> {
    None
}

fn seconds_to_millis_ceil(seconds: f64) -> u32 {
    (seconds * 1000.0).ceil() as u32
}
```

### Notes / deviations

- The typed `objc2-app-kit` binding exposes `NSEvent::doubleClickInterval()`, so
  the Rust port does not need raw class lookup or selector messaging.
- Upstream returns `null` if the `NSEvent` class is unavailable. The typed
  binding assumes AppKit is linked on macOS; this is acceptable for Roastty's
  macOS product path. The function still returns `Option<u32>` to preserve the
  public shape and non-macOS fallback.
- `seconds_to_millis_ceil` is split out so the exact conversion behavior can be
  tested without depending on the user's system preference.
- This experiment adds the minimal `objc2-app-kit` dependency/features to
  `roastty/Cargo.toml`: `default-features = false`, feature `NSEvent`.

## Changes

- `roastty/Cargo.toml` — add
  `objc2-app-kit = { version = "0.3", default-features = false, features = ["NSEvent"] }`.
- `roastty/src/os/mouse.rs` — add `click_interval`,
  `system_double_click_interval_seconds`, and `seconds_to_millis_ceil`.
- `roastty/src/os/mod.rs` — expose the new `mouse` module.

## Verification

- `cargo build -p roastty` — no warnings.
- `cargo test -p roastty` — new tests cover:
  - second-to-millisecond conversion uses `ceil` (`0.001` → `1`, `0.0011` → `2`,
    `0.5` → `500`);
  - zero edge value does not panic (`0.0` → `0`);
  - `click_interval()` smoke test returns `Some(ms > 0)` on macOS and `None` on
    non-macOS hosts.
- `cargo fmt -p roastty -- --check` — clean.
- no-ghostty grep on touched source — clean.
- `git diff --check` — clean.

Pass = Roastty has the OS mouse click-interval helper needed by later mouse
selection and surface input slices.

## Design Review

**Reviewer:** Codex (gpt-5.5, medium) · resumed session
`019e8f83-9029-7d43-8e82-f4c5754e14ba`

**Verdict:** APPROVED.

Initial review found one Required issue: the dependency declaration needed to
explicitly disable `objc2-app-kit` default features, since the crate's default
surface is broad. The design now specifies
`objc2-app-kit = { version = "0.3", default-features = false, features = ["NSEvent"] }`.

The review also suggested narrowing edge-case wording to zero rather than
defining negative/NaN/overflow float-cast behavior outside upstream's normal
positive system value path. Follow-up review approved the binding/API choice,
macOS/non-macOS behavior, `ceil(seconds * 1000)` conversion, `Option<u32>`
shape, module exposure, and verification plan.

## Result

**Result:** Pass

`roastty/src/os/mouse.rs` now provides `click_interval()` with the upstream
shape: on macOS it calls the typed AppKit `NSEvent::doubleClickInterval()` and
converts seconds to milliseconds with `ceil`; on non-macOS hosts it returns
`None`. The conversion is factored into `seconds_to_millis_ceil` and covered by
unit tests independent of the user's system preference.

`roastty/Cargo.toml` now has a direct minimal AppKit dependency with
`default-features = false` and feature `NSEvent`; `Cargo.lock` records that the
already-present `objc2-app-kit` package is now also a direct `roastty`
dependency. `roastty/src/os/mod.rs` exposes the new module.

Gates (all green):

- `cargo test -p roastty os::mouse::tests` — **2 passed / 0 failed** focused
  tests.
- `cargo build -p roastty` — no warnings.
- `cargo test -p roastty` — **3444 passed / 0 failed** unit tests, plus **1
  passed / 0 failed** ABI harness test.
- `cargo fmt -p roastty -- --check` — clean.
- no-ghostty grep on `roastty/src/os/mouse.rs`, `roastty/src/os/mod.rs`, and
  `roastty/Cargo.toml` — clean.
- `git diff --check` — clean.

## Conclusion

Roastty now has the system double-click interval helper needed by later mouse
selection and surface input slices. The port keeps the upstream `Option<u32>`
shape while using the typed AppKit binding on macOS.

## Completion Review

**Reviewer:** Codex (gpt-5.5, medium) · resumed session
`019e8f83-9029-7d43-8e82-f4c5754e14ba`

**Verdict:** APPROVED — no Required findings.

Codex confirmed the implementation matches the approved slice and upstream
behavior: `objc2-app-kit` is declared with `default-features = false` and only
`NSEvent`; macOS uses typed `NSEvent::doubleClickInterval()` and wraps it in the
upstream-compatible `Option` shape; non-macOS returns `None`; conversion uses
`ceil(seconds * 1000.0) as u32`; `os::mouse` is exposed from `mod.rs`; the
result documentation is accurate; and touched source satisfies the Roastty
naming constraint.
