+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 10: Embedded ABI — the config + function tail (tranche 3)

## Description

The last ABI tranche: the **11-symbol tail** after Exp 8 (input) + Exp 9
(action) took the gap 56 → 11. Closing it should take the renamed app from
"types resolve" to **compiles + links** against `libroastty` — the Phase B exit
criterion.

**The remaining 11 (from the `app-uses ∖ roastty.h` diff):**

- **6 value types** (all extractable from `ghostty.h`, byte-faithful):
  `roastty_config_color_s` (`{u8 r,g,b}`), `roastty_config_color_list_s`
  (`{const color_s* colors; size_t len}`), `roastty_config_command_list_s`
  (`{const command_s* commands; size_t len}`), `roastty_command_s`
  (`{const char* action_key, action, title, description}`),
  `roastty_quick_terminal_size_s`
  (`{tag_e tag; union{f32 percentage; u32 pixels} value}`),
  `roastty_config_quick_terminal_size_s`
  (`{quick_terminal_size_s primary, secondary}`) — plus the
  `quick_terminal_size_tag_e` dependency (constants not yet in the header →
  fresh, no collision).
- **4 functions** the app calls, implemented as **documented behavior-preserving
  stubs** (the real behavior is a feature-completion item, not a crash, and not
  needed to build/link/run the core):
  - `roastty_cli_try_action(void)` — upstream parses `argv` for `+subcommand`
    CLI actions; `main_c.zig:119` `return`s normally when there's no `+action`
    and only `exit()`s when one is present. The GUI launch path
    (`main.swift:31`) has no `+action`, so **no-op** reaches `NSApplicationMain`
    exactly as upstream would. _Phase-C gap: `roastty +action` CLI invocations
    now fall through to a GUI launch instead of running the action and exiting._
  - `roastty_set_window_background_blur(app, void*)` — upstream is a cosmetic
    CGS blur that early-returns at `background-opacity >= 1.0`; void return,
    callers ignore it → **no-op**.
  - `roastty_inspector_metal_init(inspector, void*) -> bool` — **return
    `false`**. (Note: the app does **not** gate on the result — `InspectorView`
    discards it and calls `metalRender` every frame while the inspector window
    is open. Launch-safe because the inspector is opt-in debug UI off the core
    path, but with these stubs the app **leaks the retained Metal
    `device`/`commandBuffer`/`descriptor`** per frame the inspector is open —
    recorded as the Phase-C inspector-wiring item.)
  - `roastty_inspector_metal_render(inspector, void*, void*)` — **no-op** (see
    the leak note above). `roastty_inspector_metal_shutdown` is **not
    referenced** by the app/header/Rust (reviewer-confirmed) — not needed.

`roastty_app` in the diff is the **Exp-7 Swift-var false positive**
(`@StateObject var roastty_app`), not a C symbol — no action.

**Inert config after Exp 10 (Phase C):** defining the 6 types makes the app
compile, but `roastty_config_get` (lib.rs:9766) still has **no match arm** for
`macos-icon-ghost-color`, `macos-icon-screen-color`, `command-palette-entry`, or
`quick-terminal-size` (returns `false`), so those four features silently fall
back to nil/empty/default until Rust arms are added. This is **not** "config
color / command palette / quick-terminal-size works" — it's "the app builds;
those accessors are inert pending Phase C."

## Approach

1. **Types:** extract the 6 types + `quick_terminal_size_tag_e` + the **named**
   `quick_terminal_size_value_u` union typedef (standalone, as upstream defines
   it) from `ghostty.h`, rename, **collision-check** (none expected — verified
   `QUICK_TERMINAL_SIZE_*` absent), dependency-order, insert into `roastty.h`.
   Add Rust `#[repr(C)]` mirrors **only if** roastty code/exports need to
   construct them; if the types are only consumed by the app (the existing
   config accessors already return them once defined), no Rust struct is needed
   — confirm by checking which roastty exports reference them.
2. **Functions:** add the 4 `#[no_mangle] extern "C"` stubs in `lib.rs` with the
   documented behavior + the decls in `roastty.h`. Each logs nothing and is
   side-effect-free except the documented one.
3. **Rebuild RoasttyKit + the app.** The goal: the Swift app **compiles and
   links** (no missing symbols). Any remaining error is either (a) a
   still-missing symbol → add it, or (b) a deeper semantic gap → record for
   Phase C.
4. **`cargo test -p roastty --lib`** green.

## Verification

1. **Header parses clean** (clang `-fsyntax-only`), no duplicate constants; C
   `_Static_assert`s for the non-trivial layouts (`config_color_s` = 3 bytes;
   `quick_terminal_size_s` tag+union).
2. **`cargo test -p roastty --lib`** green (the stubs + types don't regress
   anything).
3. **The worklist is empty** (`app-uses ∖ roastty.h` = ∅, modulo the
   `roastty_app` false positive) **and the app build advances to a clean
   compile+link** — captured as the headline result (Phase B exit). If a link
   still fails, the first missing symbol is recorded.

**Pass** = the 6 types + 4 function stubs are in `roastty.h`/`libroastty`
byte-faithful, the worklist is empty, `cargo test` green, and **the renamed app
compiles + links** against `libroastty` (Phase B done) — or, if a residual
non-ABI build issue remains, it is isolated and documented with the link
reaching past all `roastty_*` symbols.

**Partial** = types + stubs resolve and tests pass, but the app still has a
missing symbol or a non-ABI build error that needs a follow-up (documented).

**Fail** = a remaining symbol can't be satisfied without real subsystem work
(documented as the Phase C entry point).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED** (no Required findings). It independently
re-derived the worklist (`app-uses ∖ Rust exports` = the same 4 functions;
roastty already exports 243 symbols vs 242 header decls, so once the 4 stubs
land **no `roastty_*` function symbol is unresolved**), confirmed the three
header copies are byte-identical, verified `roastty_app` is a Swift identifier,
confirmed `inspector_metal_shutdown` is unreferenced, verified the 6 type
layouts against upstream (`config_color_s`=3/align 1, `command_s`=32,
`quick_terminal_size_s`=8, trivial 0/1/2 tags), and confirmed the config types
are **header-only** (the existing `roastty_config_get` has no arm for these keys
→ returns false → Rust never constructs them). Optional accuracy corrections,
all folded in above: the inspector stubs **leak** retained Metal objects per
frame (the app doesn't gate on the init result) rather than "gate render"; four
config keys stay **inert** post-Exp-10; `+action` CLI now falls through to GUI
launch; keep the **named** `quick_terminal_size_value_u` typedef.

## Result

**Result:** Partial — the defined 11-symbol scope is fully resolved (gap → only
the `roastty_app` Swift false positive) and three additional ABI divergences
that surfaced as the app compiled were fixed, but the app does **not** yet fully
compile: it now reaches the **`selection_s`/`point_s` layout divergence** (Exp-6
#3), a real subsystem reconciliation deferred to Exp 11.

### What landed (all in `roastty.h` + `libroastty`, `cargo test` green)

- **6 config/misc value types** byte-faithful (header-only —
  `roastty_config_get` has no arm for these keys, so Rust never constructs
  them): `config_color_s` (3 B), `config_color_list_s`, `config_command_list_s`,
  `command_s` (32 B), `quick_terminal_size_s` (tag + named `value_u`, 8 B),
  `config_quick_terminal_size_s`, + `quick_terminal_size_tag_e`. C
  `_Static_assert`s pin the layouts.
- **4 function stubs** (documented behavior): `cli_try_action` (no-op),
  `set_window_background_blur` (no-op), `inspector_metal_init` (→false),
  `inspector_metal_render` (no-op).
- **3 divergences that the compile surfaced, fixed inline** (small, mechanical):
  - **Mouse/action enum types:** `surface_mouse_button` /
    `inspector_mouse_button` / `inspector_key` decls pointed at the embedded
    `input_mouse_state_e`/`input_mouse_button_e`/ `input_action_e` (matching
    upstream) instead of the old `mouse_button_e`/`key_action_e` — header-only
    (the enums share values; the Rust is `c_int`).
  - **`init` success sentinel:** upstream uses `#define GHOSTTY_SUCCESS 0` (an
    int), but roastty had `ROASTTY_SUCCESS` as a `roastty_result_e` enumerator →
    the app's `roastty_init(...) != ROASTTY_SUCCESS` failed (`Int32 != enum`).
    Added `#define ROASTTY_SUCCESS 0` (mirroring upstream) and renamed the
    granular enum's value to `ROASTTY_RESULT_SUCCESS` to avoid the macro
    collision (the Rust keeps its own separate `ROASTTY_SUCCESS` const —
    unaffected).

### Verification

- **`cargo test -p roastty --lib`:** green (the types are header-only + the 4
  stubs add no logic; no regression): **4396 passed, 0 failed**.
- **Worklist empty:** `app-uses ∖ roastty.h` = `{roastty_app}` (the Swift var).
  All real `roastty_*` symbols (types + functions) resolve.
- **App build progression** (each rebuild advanced past the prior blocker):
  missing config symbols → resolved; mouse/action enum mismatches → resolved;
  `init`/`ROASTTY_SUCCESS` → resolved; **now blocked on
  `selection_s`/`point_s`** (14 errors in `SurfaceView_AppKit.swift`:
  `roastty_point_s(tag:coord:x:y:)` + `ROASTTY_POINT_COORD_*`).

## Conclusion

The 11-symbol config/function tail is closed, and the app build now reaches
**past the entire missing-symbol + enum-mismatch + init surface** — a major
milestone. But "compiles + links" is not yet met: the app uses the embedded
`point_s` `{tag, coord, x, y}` + `point_coord_e` and `selection_s`
`{top_left, bottom_right, rectangle}`, whereas roastty has a **completely
different** point/selection ABI (grid-based tagged-union `point_s`;
size-prefixed `selection_s` with gestures — 801's pull-model scaffolding). That
is the Exp-6 divergence #3 and a genuine subsystem reconciliation (the Rust
`read_selection`/ `write_selection`/`point_coordinate`/gesture machinery + the
`read_text`/`read_selection`/ `quicklook_word` functions the app calls), so it
gets its **own experiment**, not a cram into Exp 10.

**Next (Exp 11):** reconcile the `point_s`/`selection_s` embedded ABI — define
`point_coord_e`, fix `point_tag_e` (`SURFACE` not `HISTORY` at index 3), make
`point_s`/`selection_s` byte-faithful, and rewire the surface selection
functions. Then re-attempt the app compile+link (likely revealing the next
divergence, if any).

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED** (no Required/Optional/Nit findings). It
attacked the two highest-risk axes and both held: (1) **no silent input
corruption** — the embedded
`input_mouse_state_e`/`input_mouse_button_e`/`input_action_e` carry **identical
integer values** to the old
`mouse_button_state_e`/`mouse_button_e`/`key_action_e` (RELEASE=0/PRESS=1,
UNKNOWN=0…ELEVEN=11, REPEAT=2), and the Rust maps via `*_from_int` with matching
semantics, so the decl type-name swap is purely cosmetic; (2) **the `#define`
doesn't corrupt the header** — `ROASTTY_SUCCESS` appears only as the macro +
comment, the enumerator is `ROASTTY_RESULT_SUCCESS`, nothing references the old
name, clang exits 0 and the `_Static_assert`s pass. It also confirmed the 6
config types are byte-faithful + header-only (no Rust mirror), the 4 stubs match
their decls, and **"Partial" is honest** — the `point_s`/`selection_s`
divergence is genuinely Exp-6 #3, properly deferred to Exp 11.
