# Experiment 1: Map the Electron PDF extension layer

## Description

Before porting thousands of lines of Chromium embedder glue, produce a precise
Electron-to-TermSurf port map for the PDF extension layer. This experiment does
not modify Chromium source. It reads the local Electron and Chromium sources,
identifies the exact browser/renderer/resource/process pieces Electron uses for
PDF, and turns that into a dependency-ordered implementation plan for the first
real Chromium patch.

This is not a delay tactic. Issue 789 and Issue 790 proved that shortcutting
around Chromium's PDF architecture wastes time. Issue 792's direction is now
settled: stay on `content_shell`, copy Electron's embedder-owned extensions
layer, and add the pieces required for inline PDF. The first useful experiment
is therefore the map that prevents the implementation from becoming either too
small to matter or too broad to review.

This experiment must be reviewed by Claude before it is considered ready, per
Issue 792's principles. This document receives the **before** review: Claude
reviews the experiment design before the audit runs. After the audit result is
recorded, Claude must perform the **after** review of the result before the next
experiment is designed. No implementation experiment may begin until both
reviews for this experiment are satisfied.

## Changes

1. Audit Electron's PDF-related extension layer from the local vendor checkout.

   Source roots:

   ```text
   vendor/electron/shell/common/extensions/
   vendor/electron/shell/browser/extensions/
   vendor/electron/shell/renderer/extensions/
   vendor/electron/shell/browser/electron_browser_client.*
   vendor/electron/shell/renderer/renderer_client_base.*
   vendor/electron/patches/chromium/
   ```

   Required search terms:

   ```text
   PdfViewerStreamManager
   streams_private
   pdf_viewer_private
   resources_private
   MimeHandlerView
   CreateWebUIURLLoaderFactory
   pdf_extension_util
   IsPdfRenderer
   ```

2. Cross-check Chromium's reference implementations.

   Use `chromium/src/` only; do not use web search. Compare Electron's code to:

   ```text
   chromium/src/extensions/shell/
   chromium/src/chrome/browser/pdf/
   chromium/src/extensions/browser/guest_view/
   chromium/src/extensions/renderer/guest_view/
   chromium/src/pdf/
   ```

   The goal is to distinguish:
   - pieces Electron owns and TermSurf should copy;
   - pieces Chromium already provides once the right client/delegate is wired;
   - Chrome-product pieces TermSurf must avoid;
   - patches Electron carries because Chromium's default path is
     Chrome-specific.

3. Audit TermSurf's current embedder seams.

   Inspect the Issue 784 baseline in `chromium/src/content/libtermsurf_chromium`
   and identify the exact existing classes/hooks that can host the port:

   ```text
   TsMainDelegate
   TsContentClient
   TsBrowserClient
   TsRendererClient
   TsBrowserMainParts
   ShellBrowserContext usage
   WebContents / tab creation
   non-network URL loader factory hooks
   renderer interface binder hooks
   process command-line hooks
   ```

4. Inventory the parked PDF work from Issues 789 and 790.

   For each layer in the port map, identify whether prior work exists in the
   parked Issue 789/790 branches or `chromium/patches/issue-789/`. Classify each
   prior attempt as:

   ```text
   lift as-is
   lift and rework against the 784 baseline
   discard and rewrite
   no prior attempt
   ```

   Include the branch/commit or patch filename when prior work exists. This
   prevents the next implementation experiment from rediscovering code that was
   already written, while also preventing stale failed code from being copied
   blindly.

5. Produce a dependency-ordered port map in the Result section.

   The result must include a table with these columns:

   ```text
   Layer
   Electron source file(s)
   Chromium reference file(s)
   TermSurf target file(s)
   Prior Issue 789/790 attempt + disposition
   Required GN deps
   Narrow Chromium target, if any
   Broad/forbidden target(s) to avoid
   Required runtime init hook
   Chrome-only call sites to stub or replace
   Platform / CALayerHost interaction risk
   Verification signal
   In first implementation slice? yes/no
   ```

   Required layers to classify:
   - common extensions client;
   - extension API provider, schema registration, manifest handlers, and
     permission registration;
   - browser extensions client;
   - extension system and keyed-service factories;
   - component extension loader and PDF manifest/resource registration;
   - extension URL loader factories;
   - WebUI `chrome://resources` serving;
   - `PluginResponseInterceptorURLLoaderThrottle` and Electron's patch/fork path
     for redirecting Chrome's PDF stream interception into embedder-owned code;
   - `pdf::PdfStreamDelegate` from Chromium's PDF interceptor boundary;
   - `resources_private` API;
   - `streams_private` API and Electron's Chromium patch;
   - `pdf_viewer_private` API;
   - guest-view / `MimeHandlerView` browser wiring;
   - renderer guest-view / `MimeHandlerViewContainerManager` wiring;
   - renderer `ContentRendererClient` overrides: `OverrideCreatePlugin()` and
     `IsPluginHandledExternally()`;
   - renderer-side JavaScript/API exposure for `chrome.mimeHandlerPrivate` and
     `chrome.pdfViewerPrivate`;
   - `--pdf-renderer` process-model wiring;
   - `PdfHost` / plugin browser bindings;
   - screenshot/runtime verification harness.

6. Define the first buildable implementation slice.

   Based on the map, choose the smallest implementation experiment that is still
   architecturally real. It must be large enough to move toward Chromium's
   canonical PDF flow, but small enough to build, test, and review.

   The slice must have concrete bounds:
   - it produces at least one named runtime verification signal;
   - it adds no more than roughly 10-15 new TermSurf Chromium source files
     unless the result explains why an abstract interface forces a larger atomic
     unit;
   - it has a single explicit GN dependency list;
   - it avoids Chromium patches outside `content/libtermsurf_chromium/` unless
     the map proves a patch is unavoidable and names it;
   - it does not include the `--pdf-renderer` process-model gate or the Issue
     776 teardown crash unless the map proves they are prerequisites for the
     chosen slice.

   The result must explicitly answer:
   - Which source files are copied/adapted first?
   - Which abstract interfaces must be satisfied in the same slice?
   - Which GN deps are expected?
   - Which runtime logs prove the slice initialized?
   - Which PDF behavior is expected to change, if any?
   - Which pieces are deliberately deferred and why?

   The expected recommendation should be either:
   - **foundation first** — the `ExtensionsClient` / `ExtensionsBrowserClient` /
     `ExtensionSystem` layer, with runtime proof that the extension system
     initializes for `ShellBrowserContext`; or
   - **vertical slice first** — only if the map proves a smaller end-to-end
     slice is buildable and more informative.

   The result must explain why the chosen slice is better for Issue 792 than the
   alternative.

7. Define branch discipline for the first implementation experiment.

   This mapping experiment does not create or use a Chromium branch. The first
   implementation experiment must fork from the protected baseline
   `148.0.7778.97-issue-784`, using a per-experiment branch such as
   `148.0.7778.97-issue-792-exp2`, and must update `chromium/README.md` and
   `chromium/patches/issue-792/` only after it builds.

8. Do not modify Chromium source.

   This experiment may edit only this issue document to record the result. If
   code is needed to answer a question, that question belongs in the next
   experiment.

9. Run Claude review on the completed result.

   Ask Claude to review the map and the proposed first implementation slice. Fix
   all real issues Claude identifies. The experiment cannot pass until Claude
   agrees the map is good enough to drive implementation.

## Verification

1. The main repo and Chromium repo are clean before the audit starts.

   ```bash
   git status --short
   git -C chromium/src status --short
   git -C chromium/src branch --show-current
   ```

   Expected Chromium branch: `148.0.7778.97-issue-784`.

2. The Result section contains the required port map table.

3. Every row in the map cites local file paths from Electron, Chromium, and
   TermSurf. No row may rely on memory or web-sourced claims.

4. The first implementation slice is explicitly named and is small enough to be
   a single reviewed Chromium branch.

5. The Result explicitly distinguishes the design review from the result review:
   this experiment's design review happens before the audit runs; the result
   review happens after the map is recorded.

6. Claude reviews the completed result and agrees it is a sound basis for the
   first implementation experiment, or all real issues Claude raises are fixed
   and re-reviewed.

## Pass Criteria

- No Chromium source changes are made.
- The port map covers every required layer listed above.
- The map separates Electron-owned code, Chromium-provided code,
  Chrome-product-only code, and TermSurf-specific glue.
- The map inventories prior Issue 789/790 code and classifies whether each
  attempt should be lifted, reworked, discarded, or ignored.
- The map names Chrome-only call sites that must be stubbed, replaced, or
  avoided.
- The first implementation slice is concrete enough to implement without
  re-litigating architecture.
- Claude agrees the experiment result is good enough to proceed.

## Partial Criteria

Partial if the audit finds a blocker that prevents a credible implementation
slice from being chosen. Valid partial outcomes include:

- an Electron dependency cannot be mapped to Chromium 148 without a source
  spike;
- a required Chromium hook does not exist on the Issue 784 baseline;
- the first implementation slice cannot be made buildable without taking a
  larger layer than expected.

The result must name the blocker and design the smallest follow-up experiment to
resolve it.

## Failure Criteria

- The result is a hand-wavy file list rather than a dependency-ordered map.
- The map omits one of the required PDF layers.
- The experiment silently proposes an app_shell rebase or broad Chrome browser
  stack import.
- The experiment proceeds to Chromium code without Claude-approved design.

## Result

**Result:** Pass

The audit was run from a clean main repo except for this issue-doc work, with
Chromium clean on `148.0.7778.97-issue-784`. No Chromium source was modified.

The conclusion is foundation-first: the first implementation slice should stand
up the common/browser extension client and extension system only. PDF component
extension registration is the next implementation slice, not part of the first
slice. That split keeps the first branch buildable and reviewable while still
moving directly toward Chromium's canonical PDF flow. Nearly every later PDF
layer assumes `ExtensionsClient`, `ExtensionsBrowserClient`, `ExtensionSystem`,
`ExtensionRegistry`, and extension API/schema registration already exist.
Another bespoke vertical slice would repeat the Issue 789/790 failure pattern:
it can fake one symptom, but it cannot host the canonical PDF OOPIF flow.

### Prior Work Inventory

Parked Chromium branches exist for `148.0.7778.97-issue-789-exp2` through
`-exp7` and `148.0.7778.97-issue-790-exp1` through `-exp5`.
`chromium/patches/issue-789/` contains eleven durable patches:

| Patch                                                  | Disposition                                                                                                               |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `0001-Build-TermSurf-PDF-handoff.patch`                | Discard as canonical direction; useful only as a warning about bespoke stream stores.                                     |
| `0002-Wire-PDF-stream-delegate.patch`                  | Lift/rework ideas only; canonical `PdfStreamDelegate` still needed, but not the old custom path.                          |
| `0003-Attach-PDF-viewer-frame.patch`                   | Discard; replaced by guest-view / MimeHandlerView.                                                                        |
| `0004-Add-PDF-viewer-stream-shim.patch`                | Discard once extension APIs are real; useful for expected `getStreamInfo()` semantics.                                    |
| `0005-Serve-PDF-viewer-chrome-resources.patch`         | Discard/rework; correct lesson is two-layer `chrome://resources` access, but canonical extension hosting should own this. |
| `0006-Grant-PDF-viewer-chrome-resources-access.patch`  | Rework only if canonical extension origin still needs explicit renderer access.                                           |
| `0007-Enable-Mojo-JS-on-PDF-viewer-frame.patch`        | Lift/rework later; relevant once viewer frame exists through guest-view.                                                  |
| `0008-Add-OOPIF-PDF-state-diagnostic.patch`            | Lift diagnostic idea only.                                                                                                |
| `0009-Flip-PDF-viewer-to-OOPIF-mode.patch`             | Discard as a bespoke template edit; canonical PDF viewer should choose its mode through real resources/features.          |
| `0010-Probe-external-PDF-plugin-routing.patch`         | Discard as a probe; it confirmed renderer/plugin routing needs canonical infrastructure.                                  |
| `0011-Link-canonical-PDF-stack-into-libtermsurf.patch` | Discard as implementation; keep as dependency warning because broad Chrome deps/linkage are unsafe.                       |

Issue 790 Experiment 5 is the most important prior result: linking canonical PDF
pieces proved feasibility but identified the missing prerequisite as the
extensions/guest-view/`PdfViewerStreamManager` browser system. Issue 790
Experiment 7 restored Chromium to the Issue 784 baseline; Issue 792 should start
from that baseline and port the real foundation deliberately.

### Port Map

#### 1. Common Extensions Client

- **Electron source:**
  `vendor/electron/shell/common/extensions/electron_extensions_client.{h,cc}`.
- **Chromium reference:** `chromium/src/extensions/common/extensions_client.*`,
  `chromium/src/extensions/shell/common/shell_extensions_client.*`.
- **TermSurf target:** new
  `content/libtermsurf_chromium/extensions/ts_extensions_client.{h,cc}`.
- **Prior attempt:** no Issue 789/790 attempt; implement fresh from Electron.
- **GN deps:** `//extensions/common`, `//extensions:extensions_resources`.
- **Narrow target:** `//extensions/common`.
- **Avoid:** Chrome's full common client from `//chrome/common/extensions`.
- **Runtime hook:** `TsMainDelegate` or `TsBrowserMainParts` before extension
  system initialization calls `extensions::ExtensionsClient::Set(...)`.
- **Chrome-only call sites:** Electron's webstore-specific behavior can be
  stubbed/neutralized.
- **Platform risk:** none; common-layer only.
- **Verification:** `[issue-792-exp2] extensions-client-set`.
- **First slice:** yes.

#### 2. Extension API Provider, Schemas, Manifest Handlers, Permissions

- **Electron source:** `electron_extensions_api_provider.{h,cc}`,
  `shell/common/extensions/api/*.json`, `*.idl`.
- **Chromium reference:** `extensions/common/api`, manifest handler registries,
  `extensions/shell/common`.
- **TermSurf target:** `ts_extensions_api_provider.{h,cc}` plus selected copied
  API schema files only when required.
- **Prior attempt:** no complete prior attempt; Issue 789 hand-made JS shims
  should be discarded after real APIs exist.
- **GN deps:** `//extensions/common`, generated extension API resources.
- **Narrow target:** Electron-style local API provider over extensions core.
- **Avoid:** importing broad Chrome API providers wholesale.
- **Runtime hook:** `TsExtensionsClient` adds core + TermSurf API providers.
- **Chrome-only call sites:** Chrome manifest handlers that assume profiles,
  webstore, or Chrome app features must be omitted unless PDF needs them.
- **Platform risk:** none.
- **Verification:** schema/provider registration log and PDF component manifest
  parses without unknown required API/manifest failures.
- **First slice:** yes, minimal provider only.

#### 3. Browser Extensions Client

- **Electron source:** `electron_extensions_browser_client.{h,cc}` and helper
  delegates under `vendor/electron/shell/browser/extensions/`.
- **Chromium reference:** `extensions/browser/extensions_browser_client.*`,
  `extensions/shell/browser/shell_extensions_browser_client.*`.
- **TermSurf target:** `ts_extensions_browser_client.{h,cc}` with minimal helper
  delegates.
- **Prior attempt:** no complete prior attempt.
- **GN deps:** `//extensions/browser`, `//extensions/common`.
- **Narrow target:** `//extensions/browser`.
- **Avoid:** `//chrome/browser/extensions:extensions`.
- **Runtime hook:** `TsBrowserMainParts::PreMainMessageLoopRun()` creates and
  sets `ExtensionsBrowserClient`.
- **Chrome-only call sites:** Electron's permission prompts, webstore/update,
  and Chrome profile helpers should be stubbed.
- **Platform risk:** low; mostly browser-context logic.
- **Verification:** `[issue-792-exp2] extensions-browser-client-set`.
- **First slice:** yes.

#### 4. Extension System and Keyed-Service Factories

- **Electron source:** `electron_extension_system.{h,cc}`,
  `electron_extension_system_factory.{h,cc}`,
  `electron_browser_context_keyed_service_factories.{h,cc}`.
- **Chromium reference:** `extensions/browser/extension_system.*`,
  `extensions/shell/browser/shell_extension_system.*`.
- **TermSurf target:** `ts_extension_system.{h,cc}`,
  `ts_extension_system_factory.{h,cc}`,
  `ts_browser_context_keyed_service_factories.{h,cc}`.
- **Prior attempt:** no complete prior attempt.
- **GN deps:** `//extensions/browser`, `//components/keyed_service/core`.
- **Narrow target:** `//extensions/browser`.
- **Avoid:** Chrome extension service graph.
- **Runtime hook:** `ShellBrowserContext` service creation via
  `BrowserContextDependencyManager`, then `InitForRegularProfile()` and
  `FinishInitialization()`.
- **Chrome-only call sites:** profile prefs/state stores may need in-memory or
  ShellBrowserContext-backed substitutes.
- **Platform risk:** none directly; service lifecycle can affect all tabs.
- **Verification:** `[issue-792-exp2] extension-system-ready context=<ptr>`.
- **First slice:** yes.

#### 5. Component Extension Loader and PDF Manifest / Resources

- **Electron source:** `electron_extension_system.cc`,
  `electron_component_extension_resource_manager.{h,cc}`.
- **Chromium reference:** `chrome/browser/pdf/pdf_extension_util.*`.
- **TermSurf target:** `TsExtensionSystem::LoadComponentExtensions()` and
  `TsComponentExtensionResourceManager`.
- **Prior attempt:** Issue 789 served PDF viewer resources manually; discard the
  manual frame attach but reuse knowledge of resource IDs and template flags.
- **GN deps:** unresolved for the PDF-registration slice; `//chrome/browser/pdf`
  and `//chrome/browser/pdf:pdf` are explicitly forbidden in the first
  foundation slice.
- **Narrow target:** no obviously perfect narrow target; Electron uses
  `pdf_extension_util` with `nogncheck`.
- **Avoid:** broad `//chrome/browser/pdf:pdf` if it drags Chrome profile deps.
- **Runtime hook:** extension system startup loads PDF component extension.
- **Chrome-only call sites:** `pdf_extension_util::GetAdditionalData()` may
  assume Chrome profile/browser state; strings/resources are likely safe.
- **Platform risk:** none.
- **Verification:** PDF extension id `mhjfbmdgcfjbbpaeojofohoefgiehjai` appears
  in `ExtensionRegistry` with `application/pdf`.
- **First slice:** no. This belongs in the next implementation slice after the
  extension foundation is initialized. The `//chrome/browser/pdf` dependency
  boundary must be audited there as the central question, not slipped into the
  foundation branch.

#### 6. Extension URL Loader Factories

- **Electron source:** `electron_browser_client.cc` factory hooks.
- **Chromium reference:** `extensions/browser/extension_protocols.*`, content
  browser client non-network URL loader hooks.
- **TermSurf target:** `TsBrowserClient`.
- **Prior attempt:** Issue 789 implemented custom extension/resource factories;
  discard once real component extension resources are served by extension
  infrastructure.
- **GN deps:** `//extensions/browser`.
- **Narrow target:** extension protocol factories.
- **Avoid:** ad hoc viewer HTML navigation from Issue 789.
- **Runtime hook:** `CreateNonNetworkNavigationURLLoaderFactory` and
  `RegisterNonNetworkSubresourceURLLoaderFactories`.
- **Chrome-only call sites:** none expected if using extensions core.
- **Platform risk:** none.
- **Verification:** `chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/...`
  resources load only for the component extension.
- **First slice:** no; after registry is alive.

#### 7. WebUI `chrome://resources` Serving

- **Electron source:** `electron_browser_client.cc` calls
  `content::CreateWebUIURLLoaderFactory(...)`.
- **Chromium reference:** `content/public/browser/web_ui_url_loader_factory.h`.
- **TermSurf target:** `TsBrowserClient`.
- **Prior attempt:** Issue 789 Exp 6/7 proved this is a two-layer problem:
  browser factory plus renderer origin access. Lift/rework only if canonical
  extension flow still needs it.
- **GN deps:** content public browser, already present or cheap.
- **Narrow target:** content helper.
- **Avoid:** custom per-path `chrome://resources` server.
- **Runtime hook:** subresource factory for PDF extension frame.
- **Chrome-only call sites:** none.
- **Platform risk:** none.
- **Verification:** no
  `Not allowed to load local resource: chrome://resources/...` for PDF viewer.
- **First slice:** no.

#### 8. Plugin Response Interceptor / Electron Patch Path

- **Electron source:**
  `vendor/electron/patches/chromium/hack_plugin_response_interceptor_to_point_to_electron.patch`.
- **Chromium reference:** Chrome plugin response interceptor under
  `chrome/browser/plugins`.
- **TermSurf target:** a TermSurf-owned fork or patch to route the interceptor
  to TermSurf/Electron-style `streams_private`.
- **Prior attempt:** Issue 789 custom throttle/handler patches are discardable;
  Issue 790 Exp 5 proves canonical stack wants this layer.
- **GN deps:** must be determined carefully.
- **Narrow target:** none obvious; fork required unless a later implementation
  audit finds a narrow non-Chrome target.
- **Avoid:** `//chrome/browser/plugins:impl`.
- **Runtime hook:** PDF response interception on navigation.
- **Chrome-only call sites:** Chrome profile/plugin utility calls must be
  replaced with TermSurf registry lookups.
- **Platform risk:** none.
- **Verification:** `application/pdf` response creates stream info instead of
  download/blank.
- **First slice:** no.

#### 9. PDF Interceptor Boundary: `PdfStreamDelegate`, `PdfNavigationThrottle`,

`PdfURLLoaderRequestInterceptor`

- **Electron source:** `electron_browser_client.cc` creates
  `pdf::PdfNavigationThrottle` and `pdf::PdfURLLoaderRequestInterceptor` with a
  Chrome/Electron stream delegate.
- **Chromium reference:** `components/pdf/browser/*`,
  `chrome/browser/pdf/chrome_pdf_stream_delegate.*`.
- **TermSurf target:** `TsPdfStreamDelegate` or equivalent.
- **Prior attempt:** Issue 789 `0002` and Issue 790 Exp 5 contain prior delegate
  experiments; lift concepts, rework against canonical extension registry and
  stream manager.
- **GN deps:** `//components/pdf/browser:interceptors`.
- **Narrow target:** `//components/pdf/browser:interceptors`.
- **Avoid:** `//chrome/browser/plugins:impl`.
- **Runtime hook:** browser client throttle/interceptor hooks.
- **Chrome-only call sites:** `ChromePdfStreamDelegate` profile/plugin queries.
- **Platform risk:** none.
- **Verification:** throttle/interceptor logs fire on PDF navigation.
- **First slice:** no.

#### 10. `resources_private` API

- **Electron source:** `api/resources_private/resources_private_api.{h,cc}`,
  `shell/common/extensions/api/resources_private.idl`.
- **Chromium reference:** PDF viewer resource string flow in
  `pdf_extension_util`.
- **TermSurf target:** TermSurf extensions API implementation.
- **Prior attempt:** Issue 789 supplied template strings manually; discard after
  real API.
- **GN deps:** extension API generated code, PDF resources.
- **Narrow target:** Electron-style local API implementation.
- **Avoid:** Chrome resources private API.
- **Runtime hook:** extension API function registry.
- **Chrome-only call sites:** `pdf_extension_util::GetAdditionalData` may need
  TermSurf substitutions.
- **Platform risk:** none.
- **Verification:** viewer can request localized/resource strings.
- **First slice:** no.

#### 11. `streams_private` API

- **Electron source:** `api/streams_private/streams_private_api.{h,cc}`.
- **Chromium reference:** `extensions/browser/api/streams_private` concepts and
  `extensions/browser/mime_handler/stream_info.h`.
- **TermSurf target:** TermSurf extensions API implementation.
- **Prior attempt:** Issue 789 stream shim and stream store are discardable
  except for expected metadata semantics.
- **GN deps:** extension API generated code, mime handler stream info.
- **Narrow target:** `//extensions/browser/mime_handler:stream_container` or
  nearby narrow targets; verify.
- **Avoid:** Chrome streams_private API.
- **Runtime hook:** extension API function registry.
- **Chrome-only call sites:** Chrome stream manager references must route to
  `PdfViewerStreamManager`.
- **Platform risk:** none.
- **Verification:** PDF viewer gets stream metadata through real extension API.
- **First slice:** no.

#### 12. `pdf_viewer_private` API

- **Electron source:** `api/pdf_viewer_private/pdf_viewer_private_api.{h,cc}`,
  `shell/common/extensions/api/pdf_viewer_private.idl`.
- **Chromium reference:** `chrome/common/extensions/api/pdf_viewer_private.*`,
  `chrome/browser/pdf/pdf_extension_util.*`.
- **TermSurf target:** TermSurf extension API implementation.
- **Prior attempt:** Issue 790 Exp 3 proved OOPIF viewer uses
  `pdfViewerPrivate.getStreamInfo`; discard bespoke shim once real API exists.
- **GN deps:** generated extension API, `//chrome/common/extensions/api` may be
  unavoidable for schema types.
- **Narrow target:** Electron-style local implementation.
- **Avoid:** Chrome browser PDF API implementation if it assumes `Profile`.
- **Runtime hook:** extension API function registry and event router.
- **Chrome-only call sites:** save/drive/viewport helpers may need no-op
  implementations.
- **Platform risk:** none.
- **Verification:** `chrome.pdfViewerPrivate.getStreamInfo()` reaches browser
  API and returns canonical stream info.
- **First slice:** no.

#### 13. Guest-View / MimeHandlerView Browser Wiring

- **Electron source:** `electron_extensions_api_client.{h,cc}`,
  `electron_browser_client.cc`, `electron_pdf_document_helper_client.cc`.
- **Chromium reference:** `extensions/browser/guest_view`,
  `extensions/browser/guest_view/mime_handler_view`.
- **TermSurf target:** TermSurf extensions API client and browser client hooks.
- **Prior attempt:** Issue 789 manual frame attach is discardable; canonical
  guest-view replaces it.
- **GN deps:** `//components/guest_view/browser`, `//extensions/browser`.
- **Narrow target:** guest_view/browser and mime_handler pieces.
- **Avoid:** app_shell `AppWindow` / desktop controller assumptions.
- **Runtime hook:** guest view manager delegate registration and
  `MimeHandlerViewGuestDelegate`.
- **Chrome-only call sites:** BrowserWindow/WebContents creation overrides need
  TermSurf per-tab CALayerHost compatibility review.
- **Platform risk:** medium. This is the first layer that can affect the per-tab
  overlay/window model.
- **Verification:** MimeHandlerViewGuest exists for PDF without stealing or
  breaking the outer WebContents.
- **First slice:** no.

#### 14. Renderer Guest-View / `MimeHandlerViewContainerManager`

- **Electron source:** `vendor/electron/shell/renderer/renderer_client_base.cc`.
- **Chromium reference:** `extensions/renderer/guest_view`,
  `extensions/renderer/guest_view/mime_handler_view`.
- **TermSurf target:** introduce `TsRendererClient` on the Issue 784 baseline
  and wire it from `TsMainDelegate::CreateContentRendererClient()`. The Issue
  784 baseline has no `TsRendererClient` and no existing
  `CreateContentRendererClient()` override.
- **Prior attempt:** Issue 776/790 had renderer plugin handling probes; lift
  knowledge, not code.
- **GN deps:** `//extensions/renderer`.
- **Narrow target:** `//extensions/renderer`.
- **Avoid:** Chrome renderer client.
- **Runtime hook:** `RenderThreadStarted` / associated interface binder for
  `MimeHandlerViewContainerManager`.
- **Chrome-only call sites:** none obvious in Electron path.
- **Platform risk:** low/medium; it coordinates with browser guest-view state.
- **Verification:** renderer exposes
  `extensions::mojom::MimeHandlerViewContainerManager`.
- **First slice:** no.

#### 15. Renderer Plugin Overrides

- **Electron source:** `renderer_client_base.cc` implements
  `OverrideCreatePlugin()` and `IsPluginHandledExternally()`.
- **Chromium reference:** content renderer client plugin hooks,
  `pdf::CreateInternalPlugin()`.
- **TermSurf target:** `TsRendererClient`.
- **Prior attempt:** Issue 776 and 790 probes exist; rework after canonical
  guest-view is present.
- **GN deps:** `//pdf`, `//extensions/renderer`.
- **Narrow target:** `//pdf`.
- **Avoid:** Chrome renderer client.
- **Runtime hook:** renderer client creation in `TsMainDelegate`.
- **Chrome-only call sites:** parent-frame origin checks and PDF renderer
  designation must be canonical, not bypassed.
- **Platform risk:** none.
- **Verification:** plugin is treated externally for PDF viewer and internal
  plugin creation occurs only in PDF renderer.
- **First slice:** no.

#### 16. Renderer-Side JS/API Exposure

- **Electron source:** `electron_extensions_renderer_client.*`,
  `electron_extensions_renderer_api_provider.*`.
- **Chromium reference:** `extensions/renderer`.
- **TermSurf target:** TermSurf renderer extensions client/API provider.
- **Prior attempt:** Issue 789/790 shims should be discarded after real renderer
  extension APIs exist.
- **GN deps:** `//extensions/renderer`, generated extension APIs.
- **Narrow target:** `//extensions/renderer`.
- **Avoid:** ad hoc global JS injection.
- **Runtime hook:** renderer extension system/API provider initialization.
- **Chrome-only call sites:** none expected.
- **Platform risk:** none.
- **Verification:** viewer sees real `chrome.mimeHandlerPrivate` /
  `chrome.pdfViewerPrivate`.
- **First slice:** no.

#### 17. `--pdf-renderer` Process-Model Wiring

- **Electron source:** process/renderer setup around PDF viewer routing; no
  single file owns it.
- **Chromium reference:** `pdf::IsPdfRenderer()`, process command-line switches,
  PDF extension tests.
- **TermSurf target:** process launch / command-line hooks in browser client or
  content client.
- **Prior attempt:** Issue 790 Exp 1-5 proved this is the crash gate; do not
  hack it before guest-view/process identity is canonical.
- **GN deps:** likely `//pdf`.
- **Narrow target:** PDF buildflags/switches.
- **Avoid:** setting `--pdf-renderer` globally.
- **Runtime hook:** renderer process launch for PDF content frame only.
- **Chrome-only call sites:** SiteInstance/process selection may assume Chrome
  extension hosting.
- **Platform risk:** none.
- **Verification:** `pdf::IsPdfRenderer()` true only in PDF renderer process.
- **First slice:** no.

#### 18. `PdfHost` / Plugin Browser Bindings

- **Electron source:** `electron_browser_client.cc` binds `pdf::mojom::PdfHost`;
  `electron_pdf_document_helper_client.cc`.
- **Chromium reference:** `pdf::PDFDocumentHelper`,
  `chrome/browser/pdf/pdf_document_helper_client.*`.
- **TermSurf target:** `TsBrowserClient` binder and TermSurf PDF helper client.
- **Prior attempt:** no complete prior attempt; Issue 790 stopped before this.
- **GN deps:** `//pdf`, maybe Chrome PDF helper pieces if separable.
- **Narrow target:** `//pdf`.
- **Avoid:** Chrome browser PDF helper if it assumes `Profile`/Chrome UI.
- **Runtime hook:** `RegisterBrowserInterfaceBindersForFrame`.
- **Chrome-only call sites:** save/download/permission integrations.
- **Platform risk:** low.
- **Verification:** PDF plugin binds PdfHost without renderer kill.
- **First slice:** no.

#### 19. Internal PDF Plugin Registration

- **Electron source:** renderer client plugin creation path.
- **Chromium reference:** `ContentClient::AddPlugins()`,
  `pdf::CreateInternalPlugin()`.
- **TermSurf target:** `TsContentClient` / `TsRendererClient`.
- **Prior attempt:** Issue 776 Exp 1 added this; lift/rework when renderer side
  is ready.
- **GN deps:** `//pdf`.
- **Narrow target:** `//pdf`.
- **Avoid:** Chrome plugin service implementation.
- **Runtime hook:** content client plugin enumeration and renderer override.
- **Chrome-only call sites:** none if using PDF component helpers.
- **Platform risk:** none.
- **Verification:** PDF mime plugin info exists and internal plugin creation is
  possible in correct renderer process.
- **First slice:** no.

#### 20. `pdf_embedder.html` / MimeHandlerView Template

- **Electron source:** extension/MimeHandlerView flow, not bespoke HTML.
- **Chromium reference:**
  `extensions/browser/guest_view/mime_handler_view/mime_handler_view_attach_helper.cc`.
- **TermSurf target:** none if using attach helper; otherwise TermSurf-owned
  template only if helper cannot link.
- **Prior attempt:** Issue 789 inlined viewer/embedder HTML; discard unless
  canonical helper is impossible.
- **GN deps:** guest-view mime handler target.
- **Narrow target:** guest-view mime handler pieces.
- **Avoid:** hand-written viewer shell.
- **Runtime hook:** response interception creates full-page MimeHandlerView
  page.
- **Chrome-only call sites:** template itself is neutral.
- **Platform risk:** medium due guest creation.
- **Verification:** generated embedder page contains expected MIME handler
  frame/embed path.
- **First slice:** no.

#### 21. OOPIF PDF Feature Flag

- **Electron source:** PDF viewer configuration follows Chromium features.
- **Chromium reference:** `chrome_pdf::features::IsOopifPdfEnabled()`,
  `pdf/pdf_features.*`.
- **TermSurf target:** diagnostic only unless feature state differs.
- **Prior attempt:** Issue 790 Exp 2 proved OOPIF was already enabled.
- **GN deps:** PDF features target.
- **Narrow target:** `//pdf:features` or equivalent.
- **Avoid:** hard-coding viewer mode through template edits.
- **Runtime hook:** startup/runtime diagnostic near PDF navigation.
- **Chrome-only call sites:** none.
- **Platform risk:** none.
- **Verification:** log feature state during PDF test.
- **First slice:** no.

#### 22. Screenshot / Runtime Verification Harness

- **Electron source:** not applicable.
- **Chromium reference:** Chrome PDF extension tests use frame/process
  assertions; TermSurf uses existing screenshot harness from Issues 776/789/790.
- **TermSurf target:** existing `scripts/test-issue-776-pdf.sh` and
  `logs/issue-*` workflow.
- **Prior attempt:** lift the automated Bitcoin PDF fixture and screenshot
  harness; keep using local server assets.
- **GN deps:** none.
- **Narrow target:** none.
- **Avoid:** manual-only verification.
- **Runtime hook:** test script launches debug stack with repo-built Roamium.
- **Chrome-only call sites:** none.
- **Platform risk:** macOS screenshot permission already known.
- **Verification:** screenshots plus structured logs.
- **First slice:** no, but foundation slice should still use smoke tests.

### First Implementation Slice

The first implementation experiment should be **foundation first**.

#### Scope

Create `148.0.7778.97-issue-792-exp2` from `148.0.7778.97-issue-784`. Add a
bounded TermSurf extensions foundation:

- `ts_extensions_client.{h,cc}`;
- `ts_extensions_api_provider.{h,cc}`;
- `ts_extensions_browser_client.{h,cc}`;
- minimal helper delegates forced by `ExtensionsBrowserClient` pure virtuals;
- `ts_extension_system.{h,cc}`;
- `ts_extension_system_factory.{h,cc}`;
- `ts_browser_context_keyed_service_factories.{h,cc}`.

Target file count: roughly 10-15 new files. If Chromium 148 pure virtuals force
more files, the result must name the forced interfaces and why the slice remains
atomic.

This slice is pure foundation: Layers 1-4 only. It does not register the PDF
component extension, does not add a component extension loader for PDF, and must
not depend on `//chrome/browser/pdf`.

The `ExtensionsBrowserClient` interface is broad enough that this slice will
force several minimal helper/delegate decisions in the same atomic branch:
browser-context routing, off-the-record context handling, extension prefs and
resource loading, process manager delegate, extension host delegate, runtime API
delegate, component extension resource manager placeholder, event broadcasting,
extension `WebContents` observers, safe-browsing/kiosk no-op delegates, and
application-locale plumbing. Those helpers should be no-op or Shell-compatible
unless the interface requires real state.

#### Expected GN deps

Initial expected deps:

```text
//extensions/browser
//extensions/common
//components/keyed_service/core
//components/prefs
```

Forbidden in this slice:

```text
//chrome/browser/pdf
//chrome/browser/pdf:pdf
//chrome/browser/plugins:impl
//chrome/browser/extensions:extensions
//chrome/browser/ui
```

#### Runtime Verification Signal

Required logs:

```text
[issue-792-exp2] extensions-client-set
[issue-792-exp2] extensions-browser-client-set
[issue-792-exp2] keyed-service-factories-built
[issue-792-exp2] extension-system-ready context=<ptr>
```

Expected PDF behavior change: none required. The slice passes if the extension
system initializes without regressing normal browsing. `application/pdf` is not
expected to change in this slice.

#### Why Foundation First

Foundation-first is better than another vertical slice because the vertical
experiments in Issues 789/790 already proved the cheap path: custom stream
stores, custom viewer attachment, and renderer shims can reach `getStreamInfo()`
but cannot satisfy the canonical OOPIF process/guest-view model. The real
bottleneck is the extension system foundation that all later layers depend on.

### Deferred Work

The following are deliberately not in the first implementation slice:

- `PdfNavigationThrottle` / `PdfURLLoaderRequestInterceptor`;
- `streams_private`, `resources_private`, `pdf_viewer_private`;
- guest-view / MimeHandlerView browser and renderer wiring;
- renderer plugin overrides;
- `--pdf-renderer` process-model work;
- `PdfHost` bindings;
- screenshot proof of rendered PDF.

Each deferred item depends on the extension foundation or on guest-view wiring,
so implementing it first would recreate the known bespoke-stack failure mode.

### Verification Against Experiment Requirements

- No Chromium source changes were made.
- The map covers every required layer plus the optional refinements Claude
  suggested: internal plugin registration, MimeHandlerView template, and OOPIF
  feature state.
- The map inventories Issue 789/790 parked work and classifies it.
- Chrome-only call sites and forbidden broad targets are named where known.
- The first implementation slice is concrete and bounded.

### Conclusion

Experiment 1 establishes that Issue 792 should proceed with an Electron-shaped
extensions foundation slice, not with another PDF-specific shim. The next
experiment should implement that foundation and prove that the extension system
initializes for `ShellBrowserContext`. The following implementation slice should
register the PDF component extension in `ExtensionRegistry`, with the
`//chrome/browser/pdf` dependency boundary treated as that slice's central
question.
