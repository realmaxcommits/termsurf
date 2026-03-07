# Issue 717: Remove `cocoa` crate from wezboard

## Goal

Replace all `cocoa` crate usage with `objc2-app-kit` + `objc2-foundation` in the
wezboard macOS code. Remove the `cocoa` and `objc` 0.2 dependencies entirely.

## Background

Issue 716 eliminated all 193 build warnings by migrating `msg_send!`/`class!`/
`sel!` macros from `objc` 0.2 to `objc2`. But the migration kept the `cocoa`
crate for its trait methods (`NSWindow::frame()`, `NSApp()`, etc.) and types
(`id`, `nil`, `NSRect`, `NSSize`, etc.). This created an awkward hybrid where
`objc2::msg_send!` calls must constantly bridge between `cocoa`'s `objc` 0.2
types and `objc2` types.

The bridge workarounds that need cleaning up:

1. **`MsgSendRect` / `MsgSendSize`** — Local types duplicating `CGRect`/`CGSize`
   layout with `objc2::Encode` impls, because cocoa's `NSRect`/`NSSize` can't
   implement the foreign `Encode` trait. Every geometry call site uses
   `std::mem::transmute`.

2. **`{ let __r: *mut AnyObject = ...; __r as id }`** — Boilerplate wrapping
   every `objc2::msg_send!` that returns an object, because `id`
   (`*mut objc::Object`) isn't a valid return type for `objc2::msg_send!`.

3. **`as *const _ as *const AnyObject` casts** — Needed everywhere because
   `objc::Object` ≠ `objc2::AnyObject`, even though they have identical memory
   layout.

4. **`sel2to1()` / `cls1to2()` / `cls2to1()` bridge helpers** — Transmute
   between `objc` and `objc2` selector/class types.

5. **Inline `NSEdgeInsets` struct** — Defined inside a function body with a
   manual `Encode` impl because cocoa doesn't provide it and `objc2-app-kit`
   does.

6. **`NSApplicationPresentationOptions` `from_bits_truncate`** — Roundtrip
   through `usize` because the cocoa bitflags type doesn't impl `Encode`.

All of these disappear once `cocoa` is replaced with `objc2-app-kit` /
`objc2-foundation`, which provide typed wrappers with native `Encode` impls.

## Scope

### Files to migrate

| File                                     | `cocoa::` imports | Trait method calls | Notes                                                                                                    |
| ---------------------------------------- | ----------------- | ------------------ | -------------------------------------------------------------------------------------------------------- |
| `window/src/os/macos/window.rs`          | 9                 | ~50                | Bulk of work. NSWindow, NSView, NSScreen, NSOpenGLContext, NSEvent, NSColor, NSColorSpace, NSAutorelease |
| `window/src/os/macos/connection.rs`      | 3                 | ~10                | NSApp, NSApplication, NSScreen, NSArray                                                                  |
| `window/src/os/macos/menu.rs`            | 3                 | ~8                 | NSApp, NSMenu, NSMenuItem                                                                                |
| `window/src/os/macos/mod.rs`             | 2                 | 2                  | NSString (alloc, init_str, UTF8String, len)                                                              |
| `window/src/os/macos/app.rs`             | 2                 | 0                  | NSApplicationTerminateReply, NSInteger                                                                   |
| `window/src/os/macos/clipboard.rs`       | 3                 | ~6                 | NSPasteboard, NSArray, NSFilenamesPboardType                                                             |
| `wezboard-font/src/locator/core_text.rs` | 1                 | 0                  | `cocoa::base::id` only                                                                                   |

### What gets removed

- `cocoa` dependency from `window/Cargo.toml` and `wezboard-font/Cargo.toml`
- `objc` dependency from `window/Cargo.toml`, `wezboard-font/Cargo.toml`
- `cocoa` and `objc` from workspace `Cargo.toml`
- All bridge helpers: `sel2to1`, `cls1to2`, `cls2to1`, `get_class`
- `MsgSendRect`, `MsgSendSize` types and all `transmute` calls
- All `__r` temporary variables and `as id` casts
- All `as *const _ as *const AnyObject` casts
- Inline `NSEdgeInsets` struct
- `objc::Encode` impls (replaced by `objc2::Encode`)

### What replaces it

| `cocoa`                                  | `objc2` equivalent                                  |
| ---------------------------------------- | --------------------------------------------------- |
| `cocoa::base::id`                        | `*mut AnyObject` or typed `&NSFoo`                  |
| `cocoa::base::nil`                       | `std::ptr::null_mut()` or `None`                    |
| `cocoa::base::BOOL` / `YES` / `NO`       | `bool`                                              |
| `cocoa::foundation::NSRect`              | `objc2_foundation::CGRect`                          |
| `cocoa::foundation::NSSize`              | `objc2_foundation::CGSize`                          |
| `cocoa::foundation::NSPoint`             | `objc2_foundation::CGPoint`                         |
| `cocoa::foundation::NSInteger`           | `objc2_foundation::NSInteger`                       |
| `cocoa::foundation::NSString`            | `objc2_foundation::NSString`                        |
| `cocoa::foundation::NSArray`             | `objc2_foundation::NSArray`                         |
| `cocoa::appkit::NSApp()`                 | `objc2_app_kit::NSApplication::sharedApplication()` |
| `cocoa::appkit::NSWindow` trait          | `objc2_app_kit::NSWindow` methods                   |
| `cocoa::appkit::NSView` trait            | `objc2_app_kit::NSView` methods                     |
| `cocoa::appkit::NSScreen` trait          | `objc2_app_kit::NSScreen` methods                   |
| `cocoa::appkit::NSEvent` trait           | `objc2_app_kit::NSEvent` methods                    |
| `cocoa::appkit::NSMenu` trait            | `objc2_app_kit::NSMenu` methods                     |
| `cocoa::appkit::NSMenuItem` trait        | `objc2_app_kit::NSMenuItem` methods                 |
| `cocoa::appkit::NSPasteboard` trait      | `objc2_app_kit::NSPasteboard` methods               |
| `cocoa::appkit::NSCursor` (via msg_send) | `objc2_app_kit::NSCursor` methods                   |
| `objc::rc::StrongPtr`                    | `objc2::rc::Retained<T>`                            |
| `objc::rc::WeakPtr`                      | `objc2::rc::Weak<T>`                                |
| `objc::declare::ClassDecl`               | `objc2::declare::ClassBuilder`                      |
| `objc::runtime::Object`                  | `objc2::runtime::AnyObject`                         |
| `objc::runtime::Class`                   | `objc2::runtime::AnyClass`                          |
| `objc::runtime::Sel`                     | `objc2::runtime::Sel`                               |
| `objc::runtime::Protocol`                | `objc2::runtime::AnyProtocol`                       |

## Ideas for experiments

1. **Start with `mod.rs` + `app.rs` + `connection.rs`** — Smallest files.
   Establish the pattern for replacing `id`/`nil`, `NSApp()`, `NSScreen`,
   `StrongPtr`. Remove bridge helpers once no file uses them.

2. **`menu.rs` + `clipboard.rs`** — Medium files. Replace `NSMenu`,
   `NSMenuItem`, `NSPasteboard` trait calls with typed `objc2-app-kit` methods.
   Replace `StrongPtr` fields with `Retained<T>`.

3. **`window.rs`** — The bulk. Replace all `NSWindow`/`NSView`/`NSEvent` trait
   calls, `StrongPtr`/`WeakPtr` fields, `ClassDecl` → `ClassBuilder`, remove
   `MsgSendRect`/`MsgSendSize`/`NSEdgeInsets`, `objc::Encode` impls.

4. **`core_text.rs` + final cleanup** — Remove last `cocoa::base::id` usage,
   remove `cocoa` and `objc` from workspace deps.
