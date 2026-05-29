# Experiment 3: Register the PDF Component Extension

## Description

Build on Experiment 2's accepted extension foundation and register Chromium's
PDF viewer as a real component extension in TermSurf's extension system.

This experiment still does **not** need to render PDFs. Its job is narrower:
prove that the PDF component extension can be loaded into
`ExtensionRegistry`/`ComponentLoader`-equivalent state on the
content_shell-based TermSurf embedder, with viewer resources served by a
TermSurf-owned component resource manager.

This is the next Electron-shaped layer because Electron does not fake the PDF
viewer as arbitrary HTML. It registers the PDF viewer resources and then routes
PDF stream handling into that registered extension. Experiment 3 should make the
registered extension exist; later experiments can wire the navigation throttle,
stream manager, guest-view/MimeHandlerView hosting, and PDF renderer process
model.

Experiment 2's Claude follow-up findings are part of this design:

- verify `TsContentRendererClient::OverrideCreatePlugin()` return-value
  semantics before any plugin path is exercised;
- load Chromium's generated PDF manifest resource directly and perform local
  placeholder replacement instead of calling Chrome's PDF browser helper;
- strip or defer Chrome-only PDF extension permissions that the current TermSurf
  extension foundation does not provide yet;
- decide how dynamic `web --profile ...` contexts get extension registration;
- document the intentional off-the-record keyed-service asymmetry;
- avoid expanding into guest-view or stream-manager work in this slice.

This experiment must receive Claude design review before implementation. After
implementation and result recording, Claude must review the completed output
before any next experiment is designed.

## Changes

1. Create the Chromium implementation branch.

   Start from the accepted Experiment 2 branch:

   ```bash
   git -C chromium/src checkout 148.0.7778.97-issue-792-exp2
   git -C chromium/src checkout -b 148.0.7778.97-issue-792-exp3
   ```

   Add the branch to `chromium/README.md` only after the branch builds.

2. Verify the renderer plugin override contract before coding.

   Inspect Chromium 148's `ContentRendererClient::OverrideCreatePlugin()` and
   `extensions::ExtensionsRendererClient::OverrideCreatePlugin()` call sites.
   Record the conclusion in the result.

   Expected outcome: TermSurf's Experiment 2 code should match Electron's
   contract. If the extensions renderer client handles a plugin request, the
   content renderer client returns `false` so the path can proceed to external
   handling rather than creating a default plugin. This audit is expected to be
   a documented no-op for this slice, but verify it before assuming that.

   If the Experiment 2 wrapper's return value is wrong, fix it before adding any
   PDF registration. The result must state whether this was a real bug or a
   documented no-op in this slice.

3. Add a TermSurf PDF component extension resource manager.

   Add TermSurf-owned files under:

   ```text
   chromium/src/content/libtermsurf_chromium/extensions/
   ```

   Suggested names:

   ```text
   ts_component_extension_resource_manager.{h,cc}
   ts_pdf_component_extension.{h,cc}
   ```

   Model the resource-map portion on the parked Issue 776 Exp 6
   `TsPdfComponentExtensionResourceManager`, but adapt it to Experiment 2's real
   `TsExtensionsBrowserClient::GetComponentExtensionResourceManager()` hook. Do
   not leave it as a standalone probe object owned by `TsBrowserClient`.

   The manager should:
   - register `kPdfResources` paths from Chromium's generated PDF resource map;
   - provide template replacements for the PDF extension;
   - expose the PDF extension id (`extension_misc::kPdfExtensionId`);
   - load `IDR_PDF_MANIFEST` directly and parse it with the `<NAME>` replacement
     applied;
   - answer `IsComponentExtensionResource()` for PDF viewer resource paths;
   - return `nullptr` or false for non-PDF component extension resources.

   Do **not** call `pdf_extension_util::GetManifest()` or import
   `//chrome/browser/pdf:pdf` for this. That helper is Chrome browser glue and
   is outside this experiment's allowed dependency surface.

4. Load the PDF component extension into the extension registry.

   Extend `TsExtensionSystem` with the minimal loader state required to create
   and register the PDF component extension for the regular browser context.
   Prefer a small TermSurf-owned loader/helper over importing Chrome's full
   `ComponentLoader` service if the full service pulls in Chrome profile/UI
   dependencies.

   The loaded extension must appear in:

   ```text
   extensions::ExtensionRegistry::Get(context)->enabled_extensions()
   ```

   with id:

   ```text
   extension_misc::kPdfExtensionId
   ```

   If Chromium's component extension helpers require a broader service, stop at
   the smallest buildable adapter and record exactly which helper forced the
   dependency.

   The minimal loader sequence is:
   - parse the local manifest dictionary from `IDR_PDF_MANIFEST`;
   - remove permissions/API entries that require Chrome-only providers not
     present in Experiment 2's extension foundation;
   - create the extension with `extensions::Extension::Create()` and
     `extensions::mojom::ManifestLocation::kComponent`;
   - add it to the enabled registry for the context;
   - notify the existing extension system/registry surfaces only as required for
     browser-side resource lookup in this slice. Renderer-side extension
     visibility is deferred to the slice that introduces the PDF viewer frame.

   The permissions stripping is diagnostic and temporary. It is the explicit
   Experiment 3 choice for unknown PDF extension permissions; later experiments
   must add the real providers rather than treating the stripped manifest as
   final PDF support. Record every stripped key in the result.

   Add a structured log of the loaded extension's `id`/`name`/`version`/manifest
   location so the result proves the real component extension was created.

5. Handle dynamic profile contexts deliberately.

   Experiment 2 registered only the startup regular context and inherited
   off-the-record context. This experiment must decide how dynamic profile
   contexts created by `TsBrowserMainParts::CreateBrowserContext()` receive the
   PDF component extension.

   Implement registration for dynamic contexts now, and verify with
   `web --profile issue792exp3`. Do not let dynamic profiles silently produce an
   invalid context with a missing pref service or a missing PDF component
   extension.

6. Keep off-the-record behavior intentionally partial.

   Add a code comment near the `CreateBrowserContextServices()` call explaining
   why Experiment 2 creates the full keyed-service set only for the regular
   context. The PDF component extension does not need to be enabled in the
   inherited off-the-record context in this experiment. Chromium's PDF manifest
   declares split-incognito behavior, but this experiment intentionally proves
   regular and dynamic regular contexts first; off-the-record PDF enablement is
   deferred until the missing PDF providers exist.

7. Update GN narrowly.

   Expected possible deps:

   ```text
   //chrome/browser/resources/pdf:resources
   //components/pdf/common:constants
   //extensions/common:common_constants
   //ui/base
   ```

   These deps are allowed only if the implementation proves they are needed for
   manifest/resource registration.

   Forbidden deps in this experiment:

   ```text
   //chrome/browser/pdf
   //chrome/browser/pdf:pdf
   //chrome/browser/plugins:impl
   //chrome/browser/extensions:extensions
   //chrome/browser:resources
   //chrome/browser/ui
   ```

   If registering the component extension truly requires a forbidden dep, stop
   and record a Partial result instead of importing it.

8. Add structured diagnostics.

   Use Chromium `LOG(INFO)` lines with this exact prefix:

   ```text
   [issue-792-exp3]
   ```

   Required lines:

   ```text
   [issue-792-exp3] pdf-resource-manager-ready count=<n> manifest_parsed=<0|1>
   [issue-792-exp3] pdf-component-extension-created id=<id> name=<name> version=<version> location=<location>
   [issue-792-exp3] pdf-component-extension-registered context=<ptr> enabled=<0|1>
   [issue-792-exp3] pdf-component-resource path=<path> resource_id=<id> found=<0|1>
   ```

   The logs must be low-volume and may remain if useful.

   The `pdf-component-resource` line needs a deterministic trigger in this
   experiment. Add a startup self-test after registration that asks the resource
   manager for a known PDF viewer path from `kPdfResources`; verify the exact
   path string from generated `pdf_resources_map.h` before coding the self-test.
   Do not rely on a future PDF navigation path to trigger the log.

9. Build and archive only after verification.

   Build:

   ```bash
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the branch builds and verification passes or produces a useful Partial, do
   the full bookkeeping:
   - commit the Chromium branch;
   - regenerate:

     ```text
     chromium/patches/issue-792/
     ```

   - add the new branch row to `chromium/README.md`;
   - after Claude's after-review agrees with the result, update Experiment 3's
     line in `issues/0792-pdf-support/README.md` from `Designed` to the final
     status (`Pass`, `Partial`, or `Fail`).

## Verification

1. Confirm starting state.

   ```bash
   git status --short
   git -C chromium/src status --short
   git -C chromium/src branch --show-current
   ```

   Chromium should start clean on `148.0.7778.97-issue-792-exp2`.

2. Build the branch.

   ```bash
   autoninja -C chromium/src/out/Default libtermsurf_chromium
   ```

3. Run the automated debug-path HTML smoke.

   Reuse the existing screenshot harness against:

   ```text
   http://localhost:9616/index.html
   ```

   Pass requires the page to visibly render and no extension IPC crash.

4. Verify the PDF component extension registration logs.

   The runtime log must show:
   - PDF resources registered;
   - PDF manifest parsed;
   - PDF extension id equals `extension_misc::kPdfExtensionId`;
   - the extension appears in the enabled registry for the regular context.

5. Run the PDF unchanged smoke.

   Load:

   ```text
   http://localhost:9616/bitcoin.pdf
   ```

   The PDF may still be blank, download, show a controlled non-rendering error,
   or otherwise fail to render. That is acceptable because
   navigation/stream/guest-view work is out of scope. The required signal is
   that the PDF component extension is registered and normal browsing is not
   regressed. A browser-process crash, renderer IPC crash, or repeated restart
   loop is not acceptable.

   This slice does not install `PdfNavigationThrottle` or
   `PdfViewerStreamManager`, so `application/pdf` navigations are expected to
   remain on the default path, likely blank/download as before. If the behavior
   changes, record it because that is useful input for the next experiment.

6. Run the dynamic-profile check selected in Change 5.

   Verify:

   ```text
   web --profile issue792exp3 --browser .../roamium http://localhost:9616/index.html
   ```

   and confirm the profile context is registered with extension prefs and has
   the PDF component extension enabled.

7. Run Claude review after recording the result.

   Provide Claude with the experiment file, Chromium diff, build output summary,
   runtime logs, screenshot artifact paths, and the recorded result. Fix all
   real findings before proceeding.

## Pass Criteria

- Chromium branch `148.0.7778.97-issue-792-exp3` builds `libtermsurf_chromium`.
- The PDF component extension is created from Chromium's real PDF manifest and
  resources.
- The PDF component extension is present in the enabled extension registry for
  the regular browser context.
- PDF viewer resources can be resolved through the TermSurf component extension
  resource manager.
- Normal HTML browsing still renders through the full debug TermSurf path.
- Loading `bitcoin.pdf` does not crash; rendering is not required in this
  experiment.
- Dynamic regular profiles receive the same PDF component extension registration
  as the startup regular profile.
- No forbidden Chrome deps are added.
- Claude reviews the completed result and agrees it is good enough to proceed.

## Partial Criteria

Partial if:

- the branch builds and resources/manifest parse, but the extension cannot yet
  be inserted into the registry without a larger loader service;
- a forbidden dep is proven necessary for registration, and the result records
  the exact dependency chain;
- the PDF extension registers for the startup context but dynamic profiles need
  a follow-up slice;
- normal browsing works, but PDF navigation remains unchanged;
- the implementation identifies a precise next missing layer, such as
  `PdfNavigationThrottle`, `PdfViewerStreamManager`, guest-view/MimeHandlerView,
  or PDF renderer process routing.

## Failure Criteria

- The experiment imports Chrome's full browser UI/profile stack to get the PDF
  extension registered.
- The experiment adds any forbidden dep without stopping and recording Partial.
- The experiment tries to make PDFs render by reintroducing the custom Issue
  789/790 wrapper stack instead of registering the component extension.
- The experiment changes Wezboard, Roamium Rust, webtui, or the TermSurf
  protobuf protocol.
- The experiment modifies guest-view/MimeHandlerView or stream-manager behavior
  before proving the PDF component extension is registered.
- The experiment proceeds without Claude design review or ignores real Claude
  findings.

## Result

**Result:** Pass

Chromium branch `148.0.7778.97-issue-792-exp3` builds `libtermsurf_chromium` and
registers the PDF viewer as a browser-side component extension without pulling
in Chrome's full browser UI/profile stack.

Implementation notes:

- Added `TsComponentExtensionResourceManager`, wired through
  `TsExtensionsBrowserClient::GetComponentExtensionResourceManager()`.
- Added `TsPdfComponentExtension` creation/registration and enabled it for each
  regular browser context.
- The PDF resource map resolves through the TermSurf resource manager. The
  self-test found `index.html` with resource id `21596`.
- The `OverrideCreatePlugin()` audit confirmed the Experiment 2 renderer wrapper
  matches Electron's contract: when
  `ExtensionsRendererClient::OverrideCreatePlugin()` handles a plugin request,
  `TsContentRendererClient::OverrideCreatePlugin()` returns `false` so the path
  can fall through to external handling rather than constructing the default
  plugin. No code change was needed for this slice.
- The PDF manifest is a static snapshot of Chromium 148's real PDF manifest.
  This avoids the forbidden `//chrome/browser:resources` dependency; TermSurf
  does not currently load Chrome's broad `browser_resources.pak`, and the PDF
  manifest does not live in the narrow PDF resource target.
- The manifest is temporarily stripped of `permissions` and
  `web_accessible_resources`. Both stripped keys are logged. Later experiments
  must add the real PDF API/resource providers before treating the manifest as
  final PDF support. `web_accessible_resources` was stripped because the MV2 PDF
  manifest's bare `pdf_embedder.css` entry tripped
  `web_accessible_resources_info.cc` in this stripped-provider slice.
- `Extension::Create()` requires an absolute root path. TermSurf uses an
  absolute profile-local component root for the extension object while serving
  actual resources through the component resource manager.
- Registering the extension in the enabled registry requires notifying
  `RendererStartupHelper::OnExtensionLoaded()`; otherwise the next renderer
  startup DCHECKs because the extension has no process-map entry. This is still
  browser-side extension lifecycle plumbing, not the PDF viewer-frame work.

Verification:

- Build: `autoninja -C out/Default libtermsurf_chromium` — succeeded.
- HTML smoke: `logs/issue-792-exp3-html-20260529-085559/pdf-smoke.png` —
  rendered the TermSurf test page.
- HTML smoke logs: `logs/issue-792-exp3-html-20260529-085559/wezboard-gui.log` —
  show `pdf-resource-manager-ready count=12 manifest_parsed=1`,
  `pdf-component-extension-created id=mhjfbmdgcfjbbpaeojofohoefgiehjai`, and
  `pdf-component-extension-registered ... enabled=1 inserted=1`.
- PDF unchanged smoke: `logs/issue-792-exp3-pdf-20260529-085626/pdf-smoke.png` —
  still shows the blank/default PDF path. The log shows content_shell's
  download-path `Not implemented` message, which is expected because this
  experiment does not install `PdfNavigationThrottle`, `PdfViewerStreamManager`,
  or MimeHandlerView.
- Dynamic profile/process smoke:
  `logs/issue-792-exp3-profile-20260529-085718/profile-smoke.png` — rendered the
  TermSurf test page with `web --profile issue792exp3`; logs show
  `ServerRegister: profile=issue792exp3` and the same PDF component-extension
  registration lines.
- Dynamic profile clarification: `TsBrowserMainParts::CreateBrowserContext()`
  currently routes profile requests back to the default `ShellBrowserContext`.
  The profile smoke proves the profile-named Roamium process still gets the
  startup context's PDF extension registration; it does not prove separate
  per-profile `ShellBrowserContext` registration because TermSurf does not have
  real per-process multi-context profiles today.
- No `unknown Channel-associated interface: extensions.mojom.Renderer` crash
  occurred.
- The known compositor teardown crash (`SEGV_ACCERR` after screenshot capture)
  still occurs and remains out of scope for this experiment.

Bookkeeping status:

- Claude after-review accepted the result with no blockers.
- Chromium branch committed as
  `0474ee55d5996 Register TermSurf PDF component extension`.
- `chromium/patches/issue-792/` was regenerated with the cumulative Issue 792
  patch stack; patch `0024-Register-TermSurf-PDF-component-extension.patch`
  contains this experiment.
- `chromium/README.md` now points to `148.0.7778.97-issue-792-exp3` and includes
  the branch row.
- The Issue 792 README experiment index marks Experiment 3 as `Pass`.

## Conclusion

Experiment 3 proves the browser-side PDF component extension can exist in the
TermSurf extension foundation: the real PDF extension id is enabled in the
registry, the generated PDF resource map is reachable, regular profile processes
get the registration, and normal browsing survives.

The resource self-test proves path-to-resource-id mapping only. It does not
prove `ResourceBundle::LoadDataResourceBytes()` can return viewer bytes. Before
MimeHandlerView or a viewer frame can fetch assets, the next slices must load or
package the PDF resource pak the way Electron does. The static manifest snapshot
should be retired once a narrow resource-pack loading path exists.

The next missing layer is not the component extension registry. The next
experiment should wire the PDF navigation/stream handoff layer that turns an
`application/pdf` response into the PDF viewer flow:
`PdfNavigationThrottle`/`PdfViewerStreamManager` and the TermSurf equivalent of
Electron's streams-private bridge.
