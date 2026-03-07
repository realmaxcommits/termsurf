# Issue 719: Wezboard code smells from objc2 migration

## Goal

Fix all code smells introduced during the objc2 migration (Issues 715-718). The
migration was mechanically sound but left behind unwrap-on-fallible-init, magic
numbers, redundant CGRect copies, verbose boilerplate, missing safety comments,
dead code, and inconsistent patterns.

## Background

An audit of the full diff from the rename commit (`eeeefdc`) to HEAD identified
13 code smells. None are showstoppers, but several are real risks (panics on
null ObjC init, UB on null `Box::from_raw`, missing safety docs on raw pointer
arithmetic).

## Smells

### 1. `Retained::from_raw(...).unwrap()` on fallible ObjC inits

`Retained::from_raw` returns `Option`. If `alloc` + `init` returns null (OOM,
init failure), `.unwrap()` panics. The OpenGL cases already use `.ok_or_else()`
— the remaining sites should too.

Sites:

| File                            | Line | Object                   |
| ------------------------------- | ---- | ------------------------ |
| `window/src/os/macos/window.rs` | 539  | `NSWindow`               |
| `window/src/os/macos/window.rs` | 3477 | `NSView` (initWithFrame) |
| `window/src/os/macos/app.rs`    | 206  | AppDelegate              |
| `window/src/os/macos/menu.rs`   | 174  | menu wrapper             |

Fix: Replace `.unwrap()` with `.ok_or_else(|| anyhow!("failed to init X"))` and
propagate with `?`, or use `expect("description")` where the function returns
`()` and panic is acceptable.

### 2. Manual ivar pointer arithmetic without safety comments

`get_view_ivar` and `set_view_ivar` in `window.rs:1982-1991` compute ivar
offsets via raw pointer math. The `.unwrap()` on `instance_variable()` will
panic if the ivar name doesn't match. No `// SAFETY:` comments document
invariants.

Fix: Add `// SAFETY:` comments explaining the invariants (ivar exists because we
registered it in `ClassBuilder`, type is `*mut c_void`, object is a valid
instance of the expected class).

### 3. `Weak::load().unwrap()` double unwrap

`window.rs:1882`: `self.view_id.as_ref().unwrap().load().unwrap()` — panics if
the view has been deallocated. The old code had the same risk (null deref
instead of panic), but this should be proper error handling.

Fix: Return `anyhow::Result` and use `context()`.

### 4. Magic numbers for ObjC constants

Bare integer literals with inline comments replace what were previously named
constants from the `cocoa` crate:

| Value      | Meaning                         | File:Line        |
| ---------- | ------------------------------- | ---------------- |
| `236isize` | `NSOpenGLCPSurfaceOpacity`      | `window.rs:286`  |
| `222isize` | `NSOpenGLCPSwapInterval`        | `window.rs:293`  |
| `2_isize`  | `NSWindowTabbingModeDisallowed` | `window.rs:548`  |
| `1u64`     | `NSWindowMiniaturizeButton`     | `window.rs:1521` |
| `0u64`     | `NSWindowCloseButton`           | `window.rs:1521` |
| `2u64`     | `NSWindowZoomButton`            | `window.rs:1521` |

Fix: Define named constants at the top of the file.

### 5. `Box::from_raw` without null guard in menu dealloc

`menu.rs:317-320`: The `dealloc` callback calls `get_ivar` then
`Box::from_raw(item)` without checking for null. If the ivar is null (e.g., init
failed before setting it), this is instant UB.

Fix: Add a null check before `Box::from_raw`.

### 6. No-op `CGRect` re-wrapping (12 sites)

`CGRect::new(CGPoint::new(frame.origin.x, frame.origin.y), CGSize::new(frame.size.width, frame.size.height))`
copies a `CGRect` into an identical `CGRect`. This exists because the old code
converted between `cocoa::foundation::NSRect` and `CGRect` using a
`cg_to_ns_rect` helper. After the migration both sides are `CGRect`, making the
conversion a no-op.

12 sites in `window.rs`: lines 346, 348, 435, 437, 667, 669, 974, 976, 1129,
2318, 2322, 3475.

Fix: Delete the re-wrapping. Use the `CGRect` directly.

### 7. Verbose `__r` boilerplate pattern (44 sites)

```rust
let __r: *mut AnyObject = objc2::msg_send![...];
__r as id
```

This exists because `msg_send!` infers return type from context, and the `id`
alias doesn't trigger inference. 44 occurrences in `window.rs`.

Fix: Define an inline helper:

```rust
unsafe fn msg_send_id(args...) -> id {
    objc2::msg_send![...]
}
```

Or use a macro. Alternatively, annotate the return type directly where possible:
`let x: id = objc2::msg_send![...] as id;` — though this may not always work
with `msg_send!` inference.

### 8. `std::env::set_var` thread safety

`listener.rs:21`: `std::env::set_var("TERMSURF_SOCKET", &sock_path)` is unsafe
in Rust 2024 edition and not thread-safe. It's called during startup before
other threads read env vars, but this is fragile.

Fix: Use `unsafe { std::env::set_var(...) }` with a `// SAFETY:` comment, or
pass the socket path through a different channel (e.g., store in a global or
pass as an argument to child process spawning).

### 9. `type id = *mut AnyObject` defined in two files

`window.rs:52` and `core_text.rs:6` both define the same type alias. The alias
is used inconsistently — some code uses `id`, some uses `*mut AnyObject`
directly, and new code uses `Retained<AnyObject>`. The coexistence of raw `id`
and owned `Retained<AnyObject>` in the same functions is confusing.

Fix: Keep the alias for now (removing it touches ~100 sites), but ensure
consistency — new code should not mix `id` and `*mut AnyObject` in the same
function.

### 10. Inconsistent `#[allow(deprecated)]` vs manual ivar helpers

`app.rs` and `menu.rs` use `#[allow(deprecated)]` with `get_ivar`/`set_ivar`.
`window.rs` replaced them with manual `get_view_ivar`/`set_view_ivar` pointer
arithmetic. Two different approaches for the same problem.

Fix: Use `#[allow(deprecated)]` with `get_ivar`/`set_ivar` everywhere — it's
simpler and less error-prone than manual pointer arithmetic. Remove
`get_view_ivar`/`set_view_ivar` and use the deprecated API with suppression
instead.

### 11. Dead `TermSurfState`

`state.rs`: Empty struct behind `lazy_static` mutex, never used. Added as
scaffolding in Issue 715 Experiment 5.

Fix: Delete the file and the `mod state` declaration. Re-add when actually
needed.

### 12. Dead `yes_no!` macro

The audit flagged this but grep shows it no longer exists — already cleaned up.
No action needed.

### 13. Missing `// SAFETY:` comments on unsafe blocks

No `unsafe` blocks in the migration have safety comments. This is consistent
with the upstream WezTerm style but is a Rust anti-pattern.

Fix: Add `// SAFETY:` comments to the new unsafe blocks we introduced (ivar
helpers, `Retained::from_raw`, `Box::from_raw`, `msg_send!` blocks that do
non-obvious things). Don't retrofit comments to inherited upstream code.

## Ideas for experiments

1. **Quick mechanical fixes** — No-op CGRect removal (12 sites), magic number
   constants, dead code deletion, null guard on `Box::from_raw`, safety
   comments. Pure cleanup, no behavior change.

2. **`__r` boilerplate reduction** — Either a helper macro/function or direct
   type annotation refactor for the 44 `__r as id` sites.

3. **Error handling** — Replace `Retained::from_raw().unwrap()` with proper
   error propagation. Fix `Weak::load().unwrap()` double unwrap.

4. **Ivar access consistency** — Pick one approach (deprecated API with
   suppression vs manual helpers) and apply everywhere.

5. **`set_var` fix** — Wrap in `unsafe` with safety comment, or switch to a
   different mechanism for passing the socket path.
