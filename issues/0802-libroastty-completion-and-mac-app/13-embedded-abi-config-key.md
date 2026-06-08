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

# Experiment 13: Embedded ABI — `config_key_is_binding` by-value (the last compile error)

## Description

After Exp 12 the app build is **one error from compiling**:
`AppDelegate.swift:579` calls
`roastty_config_key_is_binding(config, roasttyEvent)` with a by-value
`roastty_input_key_s`, but roastty's function still takes the **opaque
`roastty_key_event_t` handle**. Upstream is
`ghostty_config_key_is_binding(ghostty_config_t, ghostty_input_key_s)` (by
value).

This is **exactly the Exp-8 pattern**
(`surface_key`/`app_key`/`surface_key_is_binding`): a function that took the
interim opaque key handle must take the embedded by-value `input_key_s`. The
infrastructure already exists — `RoasttyInputKey` (the `#[repr(C)]` struct) and
`input_key_to_event` (the by-value→`KeyEvent` converter, with the native-keycode
table) were built in Exp 8.

## Approach

Mirror Exp 8's `surface_key` migration exactly:

1. **Rename the opaque function** to `roastty_config_key_is_binding_handle` (a
   retained export for the existing tests), keeping its body.
2. **Add the by-value** `roastty_config_key_is_binding(config, RoasttyInputKey)`
   that builds the `KeyEvent` via `input_key_to_event` and **mirrors the handle
   body byte-for-byte**:
   `config.key_event_is_binding(&ev.event) || default_key_event_is_binding(&ev.event)`.
   (The handle version returns config-match **OR** default-keybind match — the
   second clause `default_key_event_is_binding` gates the built-in default
   bindings and must NOT be dropped; this is where `config` differs from the
   Exp-8 `surface_key_is_binding`, which has no default fallback.)
3. **Change the `roastty.h` decl** from `roastty_key_event_t` to
   `roastty_input_key_s`.
4. **Migrate the 20 test call sites** to `roastty_config_key_is_binding_handle`
   (a scoped `sed` confined to the test module — must NOT touch the prod
   definition, the new by-value export, or the header decl). Then **add a
   by-value test** exercising a default-keybind event (the migrated tests cover
   only the `_handle` path; the by-value function — the actual app-linked
   deliverable, incl. the default fallback — needs its own coverage).
5. **`cargo test`** green, then rebuild the app — it should now **compile +
   link** (Phase B exit), or surface the next error (recorded).

## Verification

1. **Header parses clean**; the decl takes `roastty_input_key_s`.
2. **`cargo test -p roastty --lib`** green (the 21-site migration + the new
   by-value export don't regress; the opaque `_handle` path stays tested).
3. **App rebuild:** the `AppDelegate:579` error is gone. If the app **compiles +
   links**, that is the headline result (Phase B exit). If another error
   surfaces, it is recorded as the next experiment.

**Pass** = `config_key_is_binding` takes `input_key_s` by value (byte-faithful,
reusing the Exp-8 converter), `cargo test` green, and the app **compiles +
links** against `libroastty` (or the link reaches past all `roastty_*` symbols
with only a documented non-ABI residue).

**Partial** = the error resolves + tests green, but the app surfaces a further
compile error (documented as Exp 14).

**Fail** = the by-value conversion can't reuse the Exp-8 path (documented —
unexpected).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: CHANGES REQUIRED → addressed.** It confirmed the
signature change is faithful (upstream
`ghostty_config_key_is_binding(config_t, input_key_s)`), the Exp-8 infra
(`RoasttyInputKey` + `input_key_to_event`, no mutation) is reusable, and that
`AppDelegate:579` is the **only** app caller (every other app key call already
targets an existing by-value roastty function), so this is plausibly the last
compile error. Findings, folded in:

- **Required — don't drop the default fallback.** The handle body is
  `config.key_event_is_binding(&e) || default_key_event_is_binding(&e)`
  (lib.rs:10057-10060) — the by-value version must mirror **both** clauses, else
  app-global default keybinds silently break. (Unlike Exp-8's
  `surface_key_is_binding`, which has no default fallback — `config` is not a
  pure mirror of `surface`.)
- **Optional — add a by-value test** (the migrated tests only cover `_handle`).
- **Nit — it's 20 sites, not 21**, all in the test module; the sed must not
  touch the prod def / header decl.

## Result

**Result:** Pass — `config_key_is_binding` takes `roastty_input_key_s` by value
(byte-faithful, reusing the Exp-8 converter + mirroring the `config || default`
body), `cargo test` is green, and **the renamed Roastty app now compiles AND
links against `libroastty`** — `** BUILD SUCCEEDED **`, `Roastty.app` bundle
produced, **0 errors**. This is the **Phase B exit**.

### What landed

- **`config_key_is_binding` by-value:** the opaque function renamed to
  `roastty_config_key_is_binding_handle` (retained for tests); the new
  `roastty_config_key_is_binding(config, RoasttyInputKey)` builds the `KeyEvent`
  via `input_key_to_event` and returns
  `config.key_event_is_binding(&ev.event) || default_key_event_is_binding(&ev.event)`
  — **both** clauses (the default fallback the review caught). Header decl →
  `roastty_input_key_s`. The 20 test sites migrated to `_handle`.
- **A by-value regression test**
  (`config_key_is_binding_by_value_uses_default_fallback`): Escape (native
  `0x35`, no mods) is a default binding → `true`; native 'A' (`0x00`) → not a
  binding → `false`. It would fail if the default fallback were dropped.

### Verification

- **`cargo test -p roastty --lib`: 4401 passed, 0 failed** (4400 + the new
  by-value test).
- **App build: `** BUILD SUCCEEDED
  **`, 0 errors, `roastty/macos/build/Debug/Roastty.app` present.** The
  copied-and-renamed Ghostty macOS app builds end-to-end on `libroastty`.

## Conclusion

**Phase B is done.** Across Exp 6–13 the entire embedded ABI was reconciled
byte-faithfully — input (Exp 8), the action tagged-union (Exp 9),
config/function tail + mouse/action/init fixes (Exp 10), selection/point (Exp
11), the target union + action-tag completion (Exp 12), and this final by-value
config key — so an **unmodified-except-for-rename Ghostty app compiles and links
against the Rust port**. This is a strong structural proof: every struct layout,
enum, and signature the app touches now matches `libroastty`'s.

It is **not yet a runtime/behavioral proof** — building is not running. **Phase
C** is next: **run** `Roastty.app` and bring up the live `surface_draw` render
path (the app supplies the `NSView`; libroastty must render into it — the "crux"
deferred from 801), then drive the app under macOS automation and verify
features one by one (typing, rendering, selection, clipboard, scrollback,
search, splits/tabs, config, keybindings, resize, colors), fixing each gap in
`libroastty`. Exp 14 should be the first Phase-C step: launch the built app and
capture what it does (likely a blank/again-divergent render that pins the first
live-path work item).

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED** (no findings). It re-verified every static
claim: the by-value body has **both** clauses
(`config.key_event_is_binding(&ev) || default_key_event_is_binding(&ev)`,
lib.rs:10058) — the Required design fix landed and is behaviorally equivalent to
`_handle`; the header decl matches upstream (`config_t, input_key_s`) and parses
clean, `_handle` also declared; the migration is correct and complete (the only
bare by-value calls are the prod def + the 2 legit by-value test calls; 20 sites
on `_handle`; no `_handle_handle`, no prod/ header touched); the by-value test
is a **real regression guard** (fresh config → Escape `0x35` passes only via the
`|| default` clause; native 'A' `0x00` → not a binding); and the roastty
user/default split is **net-equivalent to upstream's combined keybind set**.
"Pass / Phase B exit" judged honest — fully implemented (not stubbed), with
building≠running correctly scoped to Phase C.
