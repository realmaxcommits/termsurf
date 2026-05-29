# Experiment 1: Stand up the extensions browser system (Electron-mirrored)

## Description

Port Electron's browser-side extensions layer (`vendor/electron/shell/browser/extensions/`
+ `shell/common/extensions/`) onto TermSurf's content_shell embedder as a
separable `Ts*` layer, and initialize it so the **extensions browser system runs
inside Roamium** and the **PDF viewer is registered as a component extension that
handles `application/pdf`**. This is the gating prerequisite Issue 790
Experiment 5 identified: the OOPIF PDF flow (`PdfViewerStreamManager` + the
response interceptor + guest-view) presupposes a running `ExtensionsBrowserClient`
+ `ExtensionRegistry` with the PDF mime handler registered.

Per the issue principles: we stay on **content_shell** (principle 1); we mirror
**Electron** class-for-class (principle 2); and this is a real ~1,500–2,000 LOC
port, which we **just do** (principle 3).

This is the foundational layer. It is one buildable unit because
`ExtensionsBrowserClient` / `ExtensionSystem` are abstract — they cannot be
landed in fragments. If, mid-implementation, a sub-piece proves separable and
the experiment balloons past the convention's size bound, it will be split into
`01`/`02` and the result recorded accordingly.

### Scope

In scope (browser side only):

- `TsExtensionsClient` (`//extensions/common` singleton) — mirrors
  `ElectronExtensionsClient`.
- `TsExtensionsBrowserClient` (`//extensions/browser` singleton) — mirrors
  `ElectronExtensionsBrowserClient`, with its helper cluster:
  `TsProcessManagerDelegate`, `TsExtensionHostDelegate`, `TsExtensionsAPIClient`,
  `TsComponentExtensionResourceManager`, `TsKioskDelegate`,
  `TsExtensionWebContentsObserver`, `TsMessagingDelegate` (only as far as the
  client's pure-virtuals require).
- `TsExtensionSystem` + `TsExtensionSystemFactory` + `TsExtensionLoader` —
  mirror Electron's; `InitForRegularProfile` builds the minimal service set and
  `LoadComponentExtensions()` registers the PDF viewer component extension.
- Keyed-service factory registration
  (`EnsureBrowserContextKeyedServiceFactoriesBuilt`).
- Init wiring in `TsBrowserMainParts::PreMainMessageLoopRun` and in the
  `ShellBrowserContext` setup, mirroring Electron's
  `ElectronBrowserMainParts::PreMainMessageLoopRun` and
  `ElectronBrowserContext` init.
- BUILD.gn deps proven linkable by Issue 790 Exp 5 (`//extensions/browser`,
  `//extensions/common`, `//components/guest_view/browser`,
  `//chrome/browser/pdf` for the PDF manifest, + `AVFoundation.framework`).

Deferred to later experiments (named, not silently dropped):

- The renderer-side extensions/guest-view wiring
  (`MimeHandlerViewContainerManager`, the dispatcher).
- The PDF response interceptor + `PdfViewerStreamManager` stream handoff.
- The `--pdf-renderer` process-model pieces.
- Serving the viewer's `chrome-extension://` resources (only needed once the
  viewer frame loads).

### The question this experiment answers

> Does Electron's extensions browser layer, ported as a `Ts*` layer onto
> TermSurf's content_shell embedder and per-tab CALayerHost window model,
> build, link, and initialize cleanly — registering the PDF component extension
> in the `ExtensionRegistry` and recognizing `application/pdf` as externally
> handled — with zero regression to existing browsing?

## Changes

1. **Chromium branch.** From `148.0.7778.97-issue-784` create
   `148.0.7778.97-issue-792`. Add it to the Branches table in
   `chromium/README.md` linked to this issue.

2. **Port the common layer.** Add under
   `content/libtermsurf_chromium/extensions/`:
   - `ts_extensions_client.{h,cc}` — from `electron_extensions_client.*`,
     stripped of Electron-specific webstore/permission specifics not needed by
     the PDF component extension.

3. **Port the browser layer.** Add under
   `content/libtermsurf_chromium/extensions/`, mirroring the Electron files
   one-to-one, renamed `Ts*`, in `namespace termsurf` (or
   `extensions::termsurf` where a factory must live in `namespace extensions`):
   - `ts_extensions_browser_client.{h,cc}`
   - `ts_extension_system.{h,cc}`
   - `ts_extension_system_factory.{h,cc}`
   - `ts_extension_loader.{h,cc}`
   - `ts_process_manager_delegate.{h,cc}`
   - `ts_extension_host_delegate.{h,cc}`
   - `ts_extensions_api_client.{h,cc}`
   - `ts_component_extension_resource_manager.{h,cc}`
   - `ts_kiosk_delegate.{h,cc}`
   - `ts_extension_web_contents_observer.{h,cc}`
   - `ts_browser_context_keyed_service_factories.{h,cc}`
   - any additional helper an `ExtensionsBrowserClient` pure-virtual forces in
     (recorded in the Result).

4. **Init wiring.** In `ts_browser_main_parts.cc::PreMainMessageLoopRun()`
   (mirroring `ElectronBrowserMainParts`): create `TsExtensionsClient` and
   `ExtensionsClient::Set`; create `TsExtensionsBrowserClient`, `Init()`,
   `ExtensionsBrowserClient::Set`; call
   `extensions::EnsureBrowserContextKeyedServiceFactoriesBuilt()` and the
   TermSurf one. Where TermSurf creates/sets up its `ShellBrowserContext`
   (mirroring `ElectronBrowserContext`):
   `BrowserContextDependencyManager::CreateBrowserContextServices(context)`,
   then `TsExtensionSystem::InitForRegularProfile(true)` and
   `FinishInitialization()`.

5. **PDF component extension load.** `TsExtensionSystem::LoadComponentExtensions()`
   mirrors Electron: `pdf_extension_util::GetManifest()` → `ParseManifest` →
   `Extension::Create(DIR_RESOURCES/"pdf", kComponent, manifest, REQUIRE_KEY)` →
   `extension_loader_->registrar()->AddExtension(...)`. Log the registered
   extension id and the parsed `mime_types`.

6. **BUILD.gn.** Add the new sources and the deps. Mirror the Exp-5-proven set;
   add `AVFoundation.framework` for the lone media-capture symbol. Do **not**
   add `//chrome/browser/plugins:impl` or other broad Chrome product targets.

7. **Diagnostics.** Add `[issue-792-exp1]` logs: client/browser-client set,
   factories built, system init, per-context init, PDF extension registered
   (id + mime types), and an `ExtensionRegistry` membership count. Temporary;
   removed once the layer is proven.

8. **Format** touched C++ with Chromium's formatter; do not edit Rust.

## Verification

1. **Build (primary gate).**
   `autoninja -C out/Default libtermsurf_chromium` links without
   `//chrome/browser/plugins:impl` or the Exp-8 broad-Chrome symbol explosion.
   `gn desc out/Default //content/libtermsurf_chromium deps` records the added
   deps.

2. **Init proof (runtime logs).** Launch debug Wezboard + `web` with the
   repo-built Roamium, load `index.html`, and confirm the `[issue-792-exp1]`
   logs show: `ExtensionsBrowserClient` set, factories built, system
   initialized for the context, PDF component extension registered with
   `mime_types: ["application/pdf"]`, registry count ≥ 1.

3. **External-handling proof (if reachable).** Load `bitcoin.pdf`; check whether
   `application/pdf` is now treated as externally handled (an extension claims
   the mime type) rather than the pre-PDF blank/download. The PDF need not
   render — that needs the deferred stream/guest-view layers. Record the exact
   observed behavior either way.

4. **No regression (required).** With the extensions system live but the PDF
   stream path not yet wired: `index.html` renders; link click, scroll,
   keyboard input, DevTools, dark mode, second webview, and a non-PDF download
   all behave exactly as on the 784 baseline. No new startup crash from the
   extensions/keyed-service static initializers.

5. **Commit + archive.** On Pass/Partial, commit the Chromium branch with
   git-poet and regenerate `chromium/patches/issue-792/` via `git format-patch
   148.0.7778.97..HEAD`. Update `chromium/README.md`.

## Pass Criteria

- `libtermsurf_chromium` builds and links with only the bounded extensions/PDF
  deps (no `//chrome/browser/plugins:impl`, no broad Chrome product graph).
- The extensions browser system initializes for the `ShellBrowserContext`
  without crashing.
- The PDF viewer is registered as a component extension in the
  `ExtensionRegistry`, declaring `application/pdf` (logged).
- Zero regression to the Issue 715–789 feature set.

## Partial Criteria

Partial if the system builds and initializes but a bounded piece is unresolved —
e.g. the PDF manifest cannot be loaded without an additional resource dep, or
`application/pdf` is registered but not yet observably "externally handled"
because that signal needs the renderer-side wiring. Name the exact boundary;
it defines Experiment 2.

## Failure Criteria

- The build pulls in `//chrome/browser/plugins:impl` or the broad Chrome graph.
- The extensions system crashes at startup or corrupts the browser context.
- Any regression to existing browsing (HTML, input, DevTools, overlay, popups,
  downloads).
- The port deviates from Electron's structure without a recorded reason.
