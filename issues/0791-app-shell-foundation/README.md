+++
status = "open"
opened = "2026-05-29"
+++

# Issue 791: Evaluate re-basing TermSurf's Chromium embedder on app_shell

## Goal

Determine whether TermSurf's Chromium embedder (`libtermsurf_chromium` /
Roamium) should be re-based — or rewritten — on Chromium's **app_shell**
(`extensions/shell`) instead of **content_shell** (`content/shell`). The
decision hinges on two questions:

1. **Preservation** — can a move to app_shell keep all current Roamium
   functionality working (the Issue 715–789 feature set: CALayerHost
   compositing, the Unix-socket/protobuf protocol, input forwarding, DevTools,
   dark mode, popups, the badge stub, multi-profile, etc.)?
2. **PDF (and beyond)** — would app_shell make inline PDF support (and other
   extension-dependent features) substantially easier, by inheriting the
   extensions + guest-view infrastructure that Issue 790 showed is the gating
   prerequisite?

This issue is **investigation only**. It produces evidence and a recommendation
(re-base vs rewrite vs stay on content_shell); it does not commit to the
migration.

## Background

### How we got here

Inline PDF support was pursued across three now-closed issues:

- [Issue 776: PDF files show blank white screen](../0776-pdf-not-loading/README.md)
  — investigation; proved no single toggle fixes it; TermSurf needs an
  Electron-style embedder layer.
- [Issue 789: Electron-Style PDF Viewer Infrastructure](../0789-electron-style-pdf-viewer/README.md)
  — built the stream handoff, viewer shell, `chrome://resources` loading, and
  the `mimeHandlerPrivate` shim; the viewer reached `getStreamInfo()`.
- [Issue 790: Mojo JS / OOPIF PDF](../0790-pdf-viewer-mojo-bindings/README.md) —
  got Mojo JS bindings, OOPIF viewer mode, and the internal PDF plugin to
  instantiate; stopped at the `IsPdfRenderer()` process-model layer. **Decisive
  finding:** completing inline PDF requires adopting Chromium's canonical
  extensions + guest-view + `PdfViewerStreamManager` stack — which is
  effectively re-implementing app_shell piece-by-piece on top of content_shell
  (a ~2k LOC port). Issue 790 then restored the app to the pre-PDF baseline
  (`148.0.7778.97-issue-784`) and deferred PDF pending this foundation decision.

That last finding is what motivates this issue: if TermSurf is going to depend
on the extensions/guest-view system anyway, basing on app_shell (which already
maintains that integration) may be cleaner than content_shell plus an
ever-growing hand-ported extension layer.

### content_shell vs app_shell (verified facts)

- **content_shell** (`content/shell`) — Chromium's minimal embedder for testing
  the content layer. No extensions system. TermSurf is built on it today:
  `TsBrowserClient : content::ShellContentBrowserClient`, plus
  `TsBrowserMainParts`, `TsContentClient`, `TsRendererClient`,
  `ShellBrowserContext`, etc.
- **app_shell** (`extensions/shell`) — Chromium's minimal embedder **with the
  extensions system**. It is _not_ "content_shell plus a flag": its
  `ShellContentBrowserClient` subclasses `content::ContentBrowserClient`
  **directly** (a parallel base, in `namespace extensions`), and it already
  wires the extension URL-loader factories
  (`CreateExtensionNavigationURLLoaderFactory`, worker/service-worker variants),
  `guest_view`/`web_view`, `MimeHandlerView`, `LocalFrameHost`, and
  `GuestViewHost`. It ships its own `ShellBrowserMainParts`,
  `ShellBrowserContext`, `ShellExtensionSystem`, `ShellExtensionsBrowserClient`,
  and keyed-service factories.
- Origin/caveat: app_shell was built as the **Chrome Apps** runtime (Chrome Apps
  are deprecated); the `extensions/shell` harness persists as the reference
  "extensions system on a minimal content embedder." Long-term maintenance and
  apps-oriented assumptions must be assessed.

## Analysis

### The central hypothesis

app_shell would make PDF (and future extension-dependent features) **much
easier**, because the extensions + guest-view + extension-URL-loader +
MimeHandlerView infrastructure the PDF OOPIF flow needs is already wired and
Chromium-maintained — exactly the ~2k LOC that Issue 790 would have hand-ported.
That part of the hypothesis is well-supported by the Issue 789/790 findings.

The **open** part is preservation: content_shell's and app_shell's shell-level
base classes are parallel, not compatible. app_shell's
`ShellContentBrowserClient` extends `content::ContentBrowserClient` directly,
while TermSurf's `TsBrowserClient` currently extends content/shell's
`ShellContentBrowserClient`. Re-basing means re-pointing every TermSurf
customization at app_shell's (different) base classes — or at
`content::ContentBrowserClient` directly. The cost depends entirely on how much
of the Issue 715–789 work is:

- **portable** — overrides of `content::ContentBrowserClient` /
  `ContentRendererClient` / `ContentMainDelegate` virtuals that move over
  mechanically; vs
- **content/shell-coupled** — reliant on content/shell specifics
  (`ShellBrowserContext`, `ShellBrowserMainParts` internals, content_shell's web
  test plumbing, its window/`Shell` object, etc.).

If it's mostly the former, re-basing is likely the right long-term call and PDF
becomes dramatically simpler. If the compositing/IPC/window work is deeply
entangled with content/shell internals, the migration risk may outweigh the
benefit and cherry-picking (or a third path) wins.

### Re-base vs rewrite

Two shapes to evaluate:

- **Re-base** — keep TermSurf's `Ts*` classes but change their base from
  content/shell's classes to app_shell's (or to `content::*` directly + the
  app_shell extensions wiring). Smaller conceptual change; migrates existing
  code in place.
- **Rewrite** — stand up Roamium's embedder fresh on app_shell, porting the
  715–789 features deliberately. Larger, but a chance to shed accumulated
  content_shell-isms.

### What the experiments should determine (not designed yet)

- An **audit** of `libtermsurf_chromium` (and the Roamium binary) measuring how
  much depends on `content::*` virtuals vs content/shell specifics — the key
  unknown for migration cost.
- What app_shell pulls in (binary size, dependency footprint, apps-era cruft)
  and its maintenance/deprecation status.
- A scoped **prototype**: a minimal Roamium embedder on app_shell that boots and
  shows one existing feature (e.g. CALayerHost compositing of a page) working —
  to validate preservation cheaply before committing.
- How this interacts with the **multi-engine vision**: app_shell is
  Chromium-specific, so it only affects the Roamium/Chromium engine, not the
  planned WebKit (Surfari) / Gecko / Ladybird engines. The protocol and GUI
  layers are unaffected either way.

### Constraints / non-goals

- Investigation only — no migration is performed in this issue without a
  recorded decision.
- Preserve the parked PDF work (Issue 789/790 branches + `patches/issue-789/`);
  this issue does not touch it.
- The current buildable baseline remains `148.0.7778.97-issue-784` until/unless
  a migration experiment is approved.

## Experiments

### Experiment 1: Audit content/shell coupling in libtermsurf_chromium

#### Description

Measure exactly how coupled the current (Issue 784 baseline) TermSurf Chromium
embedder is to content/shell specifics versus portable `content::*` virtuals.
This is the key unknown that decides whether re-basing on app_shell is a
low-risk mechanical move or a heavy migration — and therefore whether to re-base,
rewrite, or stay. **Read-only audit: no code changes, no Chromium branch, no
build.**

A quick survey of the 784 baseline already shows the surface is small — the
embedder is just `TsMainDelegate`, `TsBrowserClient`, `TsBrowserMainParts`,
`TsTabObserver`, the FFI entry (`libtermsurf_chromium.cc`), and the macOS
window/compositor bridges (`ts_shell_window_mac`, `ts_ca_layer_bridge_mac`,
`ts_compositor_bridge_mac`). Their content/shell ties seen so far:

| Touchpoint | content/shell dependency |
| --- | --- |
| `TsMainDelegate` | extends `content::ShellMainDelegate` |
| `TsBrowserClient` | extends `content::ShellContentBrowserClient`; calls `ConfigureNetworkContextParamsForShell`; `ShellDevTools*` |
| `TsBrowserMainParts` | extends `content::ShellBrowserMainParts`; `content::Shell` window; `ShellBrowserContext` |
| `TsTabObserver` | extends `content::WebContentsObserver` (content/public — portable) |
| FFI / window / compositor | `content::Shell` (`shell.h`, `Shell::Shutdown()`), `ShellBrowserContext*` casts, `shell_paths`, `shell_switches` |

#### Method (read-only)

1. **Enumerate every content/shell touchpoint** in
   `content/libtermsurf_chromium/` (and confirm the `roamium` Rust binary only
   touches the `ts_*` FFI surface, not content/shell — i.e. it is insulated):
   includes (`content/shell/...`), base classes, and API calls
   (`content::Shell`, `ShellBrowserContext`, `ShellMainDelegate`,
   `ShellBrowserMainParts`, `ShellContentBrowserClient`, `ConfigureNetworkContextParamsForShell`,
   `ShellDevTools*`, `shell_paths`, `shell_switches`, etc.).
2. **Classify each touchpoint** as:
   - **Portable** — an override of a `content::ContentBrowserClient` /
     `ContentMainDelegate` / `ContentBrowserMainParts` virtual, or a
     content/public type, that moves to app_shell (or `content::*` directly)
     mechanically; or
   - **Coupled** — relies on content/shell-specific behavior/types that
     app_shell does not provide or implements differently.
3. **Map each content/shell base/type to its app_shell counterpart** and note
   the API delta: `content::ShellMainDelegate` →
   `extensions::ShellMainDelegate`; `content::ShellBrowserMainParts` →
   `extensions::ShellBrowserMainParts`; `content::ShellContentBrowserClient` →
   `extensions::ShellContentBrowserClient` (note: extends
   `content::ContentBrowserClient` directly, not content/shell's client);
   `content::ShellBrowserContext` → `extensions::ShellBrowserContext`; and
   crucially the **window model** — content_shell's `content::Shell` vs
   app_shell's `AppWindow`/window handling.
4. **Deep-dive the riskiest item: the window + CALayerHost compositing path**
   (`ts_shell_window_mac` / `ts_*_bridge_mac` ↔ `content::Shell`). Determine what
   `content::Shell` provides that TermSurf relies on (the `NSView`/`NSWindow`,
   the `WebContents` host, lifecycle) and whether app_shell's window model
   exposes an equivalent, or whether this work would need to bind to
   `content::ContentBrowserClient`/`WebContents` directly (decoupling from any
   shell's window object). This is the make-or-break for "preserve all
   functionality."
5. **Synthesize**: a coupling inventory with a portability verdict per item, an
   overall re-base cost estimate (low / medium / high), the riskiest items
   called out, and a recommendation (re-base / rewrite / stay on content_shell,
   or "prototype needed to decide").

#### Verification / Deliverable

The audit's output is recorded as the experiment Result: the complete coupling
inventory (every touchpoint classified portable/coupled with its app_shell
counterpart), the window/compositing deep-dive finding, the cost estimate, and a
recommendation. Cross-check completeness with `rg` over
`content/libtermsurf_chromium/` for any `content/shell` reference not in the
inventory.

#### Pass Criteria

- Every content/shell touchpoint in the baseline embedder is enumerated and
  classified portable vs coupled, with its app_shell counterpart mapped.
- The window/CALayerHost path is assessed concretely (the riskiest item), with a
  clear statement of whether it ports, decouples, or needs a prototype.
- `roamium` is confirmed insulated (touches only the `ts_*` FFI, not
  content/shell).
- The Result states an overall re-base cost (low/medium/high) and a
  recommendation, naming any item that needs a follow-up prototype experiment.

#### Partial Criteria

Partial if the inventory is complete but one or more touchpoints (likely the
window/compositing path) cannot be judged from source alone and require a
boot-on-app_shell prototype to resolve. Name those items precisely; they define
the next experiment.

#### Failure Criteria

- The audit misses content/shell touchpoints (incomplete inventory), or
- It cannot reach any cost/recommendation conclusion (no decision value).
