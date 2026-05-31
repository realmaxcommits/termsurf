# Experiment 2: Implement the Roastty ABI Skeleton

## Status

**Result:** Fail

This experiment was designed and implemented around the wrong ABI naming model.
It exported `ghostty_*` compatibility symbols so a Ghostty-derived Swift app
could link without being renamed first. That is not the desired Roastty
architecture.

Roastty is a faithful Rust adaptation of Ghostty where possible, but it is a
renamed project. The app-facing ABI must use `roastty_*` symbols and the Swift
app must be renamed/adapted to call those symbols. The word `ghostty` should
remain only in references to the upstream Ghostty project, vendored source
paths, and future attribution/credit text.

The implementation produced by this experiment should not be treated as a
successful foundation. A follow-up experiment must replace the compatibility
surface with a Roastty-renamed ABI and tests.

## Description

Create the initial `roastty/` Rust crate and implement a minimal C ABI skeleton
for the Ghostty-style app/config/surface lifecycle. This is the first concrete
step toward replacing Zig `libghostty` with Rust `libroastty`.

This experiment does not implement terminal emulation, PTY IO, rendering, font
handling, config semantics, or Swift app integration. It proves that Roastty can
own opaque C handles, expose stable symbols, manage ownership correctly, and be
tested from outside Rust.

The original design below incorrectly required `ghostty_*` compatibility
symbols. That premise has been rejected. The correct direction is to expose a
Roastty-renamed ABI such as `roastty_app_new`, `roastty_surface_new`, and
`roastty_config_new`, then rename/adapt the Swift app boundary to match.

## ABI Scope

Implement only the minimum ABI group needed to prove lifecycle shape:

- `ghostty_init`
- `ghostty_info`
- `ghostty_string_free`
- `ghostty_config_new`
- `ghostty_config_free`
- `ghostty_config_clone`
- `ghostty_config_load_cli_args`
- `ghostty_config_load_file`
- `ghostty_config_load_default_files`
- `ghostty_config_load_recursive_files`
- `ghostty_config_finalize`
- `ghostty_config_diagnostics_count`
- `ghostty_config_get_diagnostic`
- `ghostty_config_open_path`
- `ghostty_app_new`
- `ghostty_app_free`
- `ghostty_app_tick`
- `ghostty_app_userdata`
- `ghostty_app_set_focus`
- `ghostty_app_update_config`
- `ghostty_app_needs_confirm_quit`
- `ghostty_app_has_global_keybinds`
- `ghostty_app_set_color_scheme`
- `ghostty_surface_config_new`
- `ghostty_surface_new`
- `ghostty_surface_free`
- `ghostty_surface_userdata`
- `ghostty_surface_app`
- `ghostty_surface_update_config`
- `ghostty_surface_needs_confirm_quit`
- `ghostty_surface_process_exited`
- `ghostty_surface_set_content_scale`
- `ghostty_surface_set_focus`
- `ghostty_surface_set_occlusion`
- `ghostty_surface_set_size`
- `ghostty_surface_size`
- `ghostty_surface_foreground_pid`
- `ghostty_surface_tty_name`
- `ghostty_surface_set_color_scheme`
- `ghostty_surface_request_close`

Every function in this list may return inert/default values. The requirement is
that the exported symbol exists, the signature exactly matches
`vendor/ghostty/include/ghostty.h`, ownership is correct, and repeated
create/free cycles are safe.

The ABI harness must compile against `vendor/ghostty/include/ghostty.h`, not a
Roastty-only approximation. If the implementation needs Rust-side copies of
Ghostty C structs, they must match the real C layouts used by that header.

Do not implement broader input, mouse, clipboard, split, inspector, quicklook,
or text-selection APIs in this experiment. Those require behavior semantics and
test parity that belong in later experiments.

## Changes

1. Create a new `roastty/` crate.
   - Add it to the top-level Cargo workspace.
   - Configure it as a Rust library that can build a C-linkable artifact,
     preferably `cdylib` and `staticlib`.
   - Keep the crate macOS-first but do not require AppKit, Swift, Metal, or
     CoreText yet.

2. Add an ABI module with C-compatible types.
   - Mirror only the structs/enums needed by the scoped functions from
     `vendor/ghostty/include/ghostty.h`.
   - Use `#[repr(C)]` for all C-facing structs and enums.
   - Match the exact layout of these Ghostty types:
     - `ghostty_string_s`
     - `ghostty_runtime_config_s`
     - `ghostty_surface_config_s`
     - `ghostty_surface_size_s`
     - any enum or nested struct needed by the scoped functions
   - Model opaque handles as heap-owned Rust structs behind raw pointers.
   - Make null inputs safe: functions must not dereference null pointers.
   - Make double-free impossible to guarantee only by contract; tests should
     cover valid ownership, not intentional undefined behavior.

3. Implement app/config/surface lifecycle state.
   - `ghostty_config_t` owns minimal config state and diagnostics.
   - `ghostty_app_t` stores runtime userdata and enough state for focus and
     color-scheme setters.
   - `ghostty_surface_t` stores its parent app pointer, surface userdata,
     content scale, focus, occlusion, and size.
   - `ghostty_surface_app(surface)` must return the parent app handle.
   - `ghostty_surface_size(surface)` must return the most recent size passed to
     `ghostty_surface_set_size` in the width/height pixel fields.
   - `ghostty_surface_size(surface)` may return zero for terminal-derived fields
     such as columns, rows, and cell width/height because no terminal or font
     metrics exist yet. Tests must pin that skeleton behavior.

4. Define ownership rules explicitly in code comments and tests.
   - `ghostty_config_t`, `ghostty_app_t`, and `ghostty_surface_t` are owned by
     Roastty after creation and must be released by their matching free
     function.
   - Config handles passed into `ghostty_app_new` and
     `ghostty_app_update_config` remain caller-owned. Roastty may inspect or
     clone state, but must not free the caller's config handle.
   - `ghostty_app_new` copies the `ghostty_runtime_config_s` function pointers
     and userdata values, but does not own or free callback userdata.
   - `ghostty_surface_new` stores the parent app handle as a borrowed pointer
     and must not free the app.
   - `ghostty_surface_config_s` platform pointers, strings, command arrays, and
     env vars are caller-owned for this skeleton. Roastty may copy scalar values
     such as userdata, content scale, context, and initial size, but must not
     free borrowed fields.
   - `ghostty_string_free` may only be called with string values returned by
     Roastty string-returning functions.

5. Implement string ownership.
   - Implement Ghostty-compatible `ghostty_string_s` behavior for empty strings
     and allocated strings.
   - Empty strings must be `{ ptr = NULL, len = 0, sentinel = false }`.
   - Test allocated sentinel strings and allocated non-sentinel byte strings,
     matching the behavior covered by `vendor/ghostty/src/main_c.zig`.
   - `ghostty_string_free` must free only strings allocated by Roastty.
   - Avoid exposing Rust-owned string slices whose lifetime is shorter than the
     C caller expects.

6. Add an external ABI integration harness.

   The harness must:
   - compile against `vendor/ghostty/include/ghostty.h`;
   - link through the built Roastty `cdylib` or `staticlib`;
   - call exported C symbols instead of Rust internals;
   - call `ghostty_init`;
   - create, clone, finalize, and free config handles;
   - create and free an app handle with runtime userdata;
   - create and free a surface handle;
   - verify `ghostty_app_userdata`;
   - verify `ghostty_surface_app`;
   - set and read surface size;
   - call inert lifecycle setters;
   - allocate and free string-returning values;
   - repeat create/free cycles to catch obvious ownership mistakes.

   A Rust integration test is acceptable only if it calls the C ABI through an
   `extern "C"` boundary and links the produced library artifact. It must not
   import Roastty internals directly.

7. Add a symbol export check.

   Use `nm`, `otool`, or an equivalent platform tool to verify every scoped
   `ghostty_*` symbol is exported unmangled from the built library artifact.

8. Add an ABI inventory file or section.

   Record a table in this experiment's result, or in a generated file under
   `roastty/`, with three categories:
   - implemented in Experiment 2;
   - used by the Ghostty Swift app but deferred;
   - not relevant to the current ABI skeleton.

   The inventory must include at least all `ghostty_*` symbols referenced by
   `vendor/ghostty/macos/Sources/` and all `ghostty_*` declarations in the
   lifecycle portion of `vendor/ghostty/include/ghostty.h`.

9. Add the new crate to build scripts only if needed.

   This experiment does not need to change `scripts/build.sh` unless the
   workspace or verification flow requires a first-class `roastty` component. If
   scripts are not changed, document why in the result.

10. Do not touch Wezboard.

11. Do not wire the Swift macOS app to Roastty yet.

## Test Parity

This experiment's test-parity target is the ABI/lifecycle subset, not terminal
behavior.

Relevant Ghostty reference tests and behavior:

- `vendor/ghostty/src/main_c.zig` contains `ghostty_string_s` tests for empty,
  C-string, and Zig-string ownership behavior.
- `vendor/ghostty/include/ghostty.h` defines the C ABI shape used by the macOS
  Swift app.
- `vendor/ghostty/macos/Sources/Ghostty/Ghostty.App.swift` and
  `vendor/ghostty/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  show the lifecycle calls the Swift app makes.

Roastty must add equivalent tests for:

- empty string result;
- allocated sentinel string result;
- allocated non-sentinel byte string result;
- freeing string results;
- config create/clone/free;
- app create/free and userdata round trip;
- surface create/free, parent app round trip, and size round trip.
- `ghostty_surface_size_s` skeleton behavior: width/height round trip from
  `ghostty_surface_set_size`, terminal-derived fields zeroed.

Ghostty terminal/parser/render tests are explicitly out of scope for this
experiment because this experiment does not claim any terminal behavior.

## Verification

Run:

```bash
cargo fmt
cargo test -p roastty
cargo build -p roastty
cargo metadata --format-version 1 --no-deps
nm -gU target/debug/<library-artifact> | rg 'ghostty_'
```

Expected results:

- `cargo test -p roastty` passes.
- The ABI harness compiles against `vendor/ghostty/include/ghostty.h` and links
  against Roastty's exported symbols rather than calling Rust internals
  directly.
- `cargo build -p roastty` produces the Rust library artifact.
- The symbol export check finds every scoped `ghostty_*` symbol unmangled.
- `cargo metadata` lists `webtui`, `roamium`, and `roastty` as top-level
  workspace members.
- `cargo metadata` still does not list any Wezboard crate as a top-level
  workspace member.

Also run existing workspace checks to ensure the new crate did not regress the
existing members:

```bash
cargo check -p webtui
cargo check -p roamium
./scripts/build.sh webtui
./scripts/build.sh roamium
```

Run:

```bash
git status --short
```

Expected changes are limited to:

- `roastty/`;
- top-level `Cargo.toml`;
- top-level `Cargo.lock`;
- issue documentation updates;
- build script/documentation updates only if required and justified.

## Failure Criteria

This experiment fails if:

- exported ABI symbols are missing from the scoped list;
- any scoped function uses a signature that does not match
  `vendor/ghostty/include/ghostty.h`;
- required C-facing structs use Roastty-only layouts instead of Ghostty's header
  layouts;
- the harness can only test Rust internals rather than the C ABI boundary;
- ownership of config/app/surface/string handles is ambiguous or unsafe for
  valid create/free usage;
- the implementation does not produce a documented implemented/deferred symbol
  inventory;
- the implementation silently claims terminal, PTY, renderer, font, input, or
  Swift integration behavior;
- Wezboard files are modified;
- `webtui` or `roamium` checks/build scripts regress;
- the experiment proceeds without a passing Codex design review and a passing
  Codex completion review.

## Codex Design Review

Initial review:

- `logs/codex-review/20260531-072335-661036-last-message.md`
- Result: **Needs changes**

Valid findings addressed in this design:

- Require exact signatures and C layouts from
  `vendor/ghostty/include/ghostty.h`.
- Require the external ABI harness to compile against Ghostty's real header.
- Specify ownership rules for config, app, surface, runtime callbacks, borrowed
  surface config fields, and strings.
- Require a symbol export check for every scoped `ghostty_*` symbol.
- Require an implemented/deferred/not-relevant ABI inventory.
- Pin `ghostty_string_s` and `ghostty_surface_size_s` skeleton behavior in
  tests.

Follow-up review:

- `logs/codex-review/20260531-072750-172641-last-message.md`
- Result: **Pass under the now-rejected design**

Codex found no blocking findings. The review confirmed that the prior ABI
looseness, ownership, external harness, symbol-check, Swift inventory, string
semantics, and surface-size findings were resolved. Implementation may proceed.

This design review is superseded by the later naming correction. It reviewed a
`ghostty_*` compatibility design, which is not the desired Roastty architecture.

## Result

**Result:** Fail

Implemented a Rust ABI skeleton as a new crate under `roastty/`, but it used the
wrong public ABI names.

Changes made:

- Added `roastty` as a top-level Cargo workspace member.
- Added `roastty/Cargo.toml` with `rlib`, `cdylib`, and `staticlib` outputs.
- Implemented scoped `ghostty_*` compatibility symbols. This is the reason the
  experiment failed.
- Mirrored the C-facing structs needed by the scoped ABI from
  `vendor/ghostty/include/ghostty.h`.
- Implemented inert config, app, and surface handle lifecycle.
- Implemented Ghostty-compatible empty, sentinel, and non-sentinel
  `ghostty_string_s` ownership behavior.
- Added a C ABI integration harness that compiles against the real
  `vendor/ghostty/include/ghostty.h` and links against the built Roastty dynamic
  library.
- Added `roastty/ABI_INVENTORY.md` with implemented, Swift-used-but-deferred,
  and not-currently-relevant symbol categories.
- Did not change Wezboard.
- Did not wire the Swift macOS app to Roastty.
- Did not claim terminal, PTY, rendering, font, input, or Swift integration
  behavior.

Verification run:

```bash
cargo fmt -- roastty/src/lib.rs roastty/tests/abi_harness.rs
cargo test -p roastty
cargo build -p roastty
cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | "\(.name) \(.manifest_path)"'
nm -gU target/debug/libroastty.dylib | rg 'ghostty_'
cargo check -p webtui
cargo check -p roamium
./scripts/build.sh webtui
./scripts/build.sh roamium
```

All verification commands passed.

Passing verification was not enough because the experiment verified the wrong
name surface. The correct ABI must be Roastty-renamed.

The metadata check listed exactly the TermSurf-owned workspace members:

```text
webtui /Users/ryan/dev/termsurf/webtui/Cargo.toml
roamium /Users/ryan/dev/termsurf/roamium/Cargo.toml
roastty /Users/ryan/dev/termsurf/roastty/Cargo.toml
```

The symbol export check found the scoped unmangled `ghostty_*` exports in
`target/debug/libroastty.dylib`, including config, app, surface, info, init, and
string-free symbols.

No build-script changes were needed in this experiment. `scripts/build.sh`
already remains focused on existing deliverable components; Roastty is not yet a
standalone component that needs a first-class script target.

## Codex Completion Review

Initial completion review:

- `logs/codex-review/20260531-073657-896457-last-message.md`
- Result: **Needs changes**

Valid findings addressed:

- Completed the ABI inventory with missing lifecycle-adjacent symbols.
- Added broader C harness null-input coverage for config, app, and surface
  functions.
- Added a code-level ABI ownership comment near the opaque handle definitions.

Follow-up completion review:

- `logs/codex-review/20260531-074020-064176-last-message.md`
- Result: **Needs one inventory fix**

Codex confirmed the null-safety harness and ownership-comment findings were
resolved, but found one remaining missing inventory symbol:
`ghostty_inspector_metal_shutdown`.

Final completion review:

- `logs/codex-review/20260531-074149-178418-last-message.md`
- Result: **Pass under the now-rejected design**

Codex confirmed `ghostty_inspector_metal_shutdown` is now listed in the deferred
inspector group and found no remaining blockers under the original
compatibility-symbol premise.

This completion review is superseded by the later architecture correction: the
review evaluated the implementation against the then-written `ghostty_*`
compatibility design, but that design was wrong for Roastty.

## Conclusion

Roastty does not yet have the correct C ABI skeleton. This experiment proved
that Rust can expose a tested C ABI and preserve the existing `webtui` and
`roamium` workspace builds, but it used `ghostty_*` compatibility names instead
of the required `roastty_*` names.

The next experiment must correct the naming model before adding behavior:
replace the compatibility ABI with a Roastty-renamed ABI, update the harness and
inventory around `roastty_*` symbols, and ensure any remaining `ghostty`
references are only citations to the upstream project or vendored reference
paths. Only after that should the issue proceed to config and lifecycle
behavior.
