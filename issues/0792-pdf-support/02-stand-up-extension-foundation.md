# Experiment 2: Stand Up the Extension Foundation

## Description

Implement the first buildable Electron-shaped foundation slice from Experiment

1. This experiment does **not** try to render PDFs and does **not** register the
   PDF component extension. It only proves that TermSurf can install Chromium's
   extension common/browser foundation on the existing `content_shell` base
   without regressing normal browsing.

The key architectural move is to add TermSurf-owned equivalents of Electron's
and app_shell's extension foundation classes:

- common `ExtensionsClient`;
- extension API provider;
- browser `ExtensionsBrowserClient`;
- keyed-service factory registration;
- `ExtensionSystem` for the `ShellBrowserContext`.

This is the correct first slice because every later PDF layer assumes this
foundation exists. A PDF-specific shortcut here would repeat the Issue 789/790
failure mode.

This experiment must receive Claude design review before implementation. After
implementation and result recording, Claude must review the completed output
before any next experiment is designed.

## Changes

1. Create a Chromium implementation branch.

   From the clean protected baseline:

   ```bash
   git -C chromium/src checkout 148.0.7778.97-issue-784
   git -C chromium/src checkout -b 148.0.7778.97-issue-792-exp2
   ```

   Add the branch to `chromium/README.md` only after the branch builds.

2. Add the TermSurf common extensions client.

   Add:

   ```text
   chromium/src/content/libtermsurf_chromium/extensions/ts_extensions_client.{h,cc}
   chromium/src/content/libtermsurf_chromium/extensions/ts_extensions_api_provider.{h,cc}
   ```

   Model this on:

   ```text
   vendor/electron/shell/common/extensions/electron_extensions_client.{h,cc}
   vendor/electron/shell/common/extensions/electron_extensions_api_provider.{h,cc}
   chromium/src/extensions/shell/common/shell_extensions_client.*
   ```

   The first implementation should register Chromium's core extension API
   provider and a minimal TermSurf API provider. It should not add PDF API
   schemas yet.

3. Add the TermSurf browser extensions client.

   Add:

   ```text
   chromium/src/content/libtermsurf_chromium/extensions/ts_extensions_browser_client.{h,cc}
   ```

   Model this on:

   ```text
   vendor/electron/shell/browser/extensions/electron_extensions_browser_client.{h,cc}
   chromium/src/extensions/shell/browser/shell_extensions_browser_client.*
   ```

   Implement the Chromium 148 `ExtensionsBrowserClient` interface with minimal
   Electron-style multi-context behavior. The client must not assume a single
   browser context. Track every `ShellBrowserContext` TermSurf creates,
   including the inherited off-the-record context, and map each context to its
   corresponding pref service. The first slice may keep off-the-record extension
   behavior disabled, but context-routing methods must still accept the context
   and must not break multi-profile or off-the-record browsing.

   Planned helper/delegate decisions:
   - browser-context routing: real, multi-context;
   - off-the-record context handling: real routing, extension enablement false
     unless a later slice needs it;
   - pref lookup: real, backed by the TermSurf pref service from step 4;
   - resource loading: placeholder/no bundled component resources in this slice;
   - process manager delegate: `nullptr` unless Chromium requires a no-op
     object;
   - extension host delegate: narrow app_shell-style delegate;
   - runtime API delegate: narrow app_shell-style delegate;
   - component extension resource manager: placeholder `nullptr`;
   - event broadcasting: real `EventRouter` broadcast for the target context;
   - extension `WebContents` observers: app_shell-style observer creation;
   - safe-browsing and kiosk delegates: no-op delegates only if a non-null
     object is required;
   - application locale: fixed `"en-US"` for this slice.

   Other behavior:
   - no webstore/update flow;
   - no extension permission UI;
   - `PopulateExtensionFrameBinders()` for extension frame binders, matching
     app_shell;
   - no Chrome UI product behavior.

4. Add TermSurf extension prefs, extension system, and keyed-service factories.

   Add:

   ```text
   chromium/src/content/libtermsurf_chromium/extensions/ts_extension_prefs.{h,cc}
   chromium/src/content/libtermsurf_chromium/extensions/ts_extension_system.{h,cc}
   chromium/src/content/libtermsurf_chromium/extensions/ts_extension_system_factory.{h,cc}
   chromium/src/content/libtermsurf_chromium/extensions/ts_browser_context_keyed_service_factories.{h,cc}
   ```

   Model this on:

   ```text
   vendor/electron/shell/browser/extensions/electron_extension_system.{h,cc}
   vendor/electron/shell/browser/extensions/electron_extension_system_factory.{h,cc}
   vendor/electron/shell/browser/extensions/electron_browser_context_keyed_service_factories.{h,cc}
   chromium/src/extensions/shell/browser/shell_extension_system.*
   chromium/src/extensions/shell/browser/shell_extension_system_factory.*
   chromium/src/extensions/shell/browser/shell_browser_context_keyed_service_factories.*
   chromium/src/extensions/shell/browser/shell_prefs.*
   ```

   The system should initialize enough extension services to make
   `ExtensionSystem::Get(browser_context)` return a ready TermSurf system with
   an `ExtensionRegistry`, `ExtensionPrefs`, `QuotaService`, `NullAppSorting`,
   `ServiceWorkerManager`, and `UserScriptManager` as required by the linked
   extension core. It should not load component extensions yet.

   `ts_extension_prefs` must create a minimal `PrefService` for every
   `ShellBrowserContext` that extension services attach to. Model this on
   app_shell's `shell_prefs::CreateUserPrefService()`:
   - create a JSON or in-memory pref store for the context;
   - register `ExtensionPrefs::RegisterProfilePrefs()`;
   - register `PermissionsManager::RegisterProfilePrefs()`;
   - call `user_prefs::UserPrefs::Set(browser_context, pref_service.get())`;
   - keep the `PrefService` alive for at least as long as the browser context.

   If the implementation chooses in-memory prefs for the off-the-record context,
   the result must state that explicitly.

5. Wire initialization order through TermSurf's existing browser main parts.

   Modify:

   ```text
   chromium/src/content/libtermsurf_chromium/ts_browser_main_parts.{h,cc}
   ```

   Add owned members for the common client, browser client, and per-context pref
   services. In `TsBrowserMainParts::PreMainMessageLoopRun()`, before the
   `ShellBrowserContext` is created by the inherited content_shell path, perform
   the same ordering used by Electron and app_shell:

   ```text
   ExtensionsClient::Set(...)
   ExtensionsBrowserClient::Set(...)
   EnsureTsBrowserContextKeyedServiceFactoriesBuilt()
   InitializeBrowserContexts()
   create/register PrefService for each context
   ExtensionSystem::Get(context)
   InitForRegularProfile(/*extensions_enabled=*/true)
   FinishInitialization()
   ```

   The Issue 784 baseline creates contexts inside
   `ShellBrowserMainParts::PreMainMessageLoopRun()` by calling
   `InitializeBrowserContexts()`. Because extension factory registration must
   happen before context creation, this experiment should override
   `TsBrowserMainParts::PreMainMessageLoopRun()` and reproduce the content_shell
   ordering with the extension setup inserted before
   `InitializeBrowserContexts()`. Do not call the base `PreMainMessageLoopRun()`
   if doing so would create the context before extension factories are
   registered.

   Call `InitForRegularProfile(/*extensions_enabled=*/true)` even though no
   extensions are loaded in this slice. Passing false would disable the
   foundation this issue is trying to build.

6. Wire extension frame binders through `TsBrowserClient`.

   Modify:

   ```text
   chromium/src/content/libtermsurf_chromium/ts_browser_client.{h,cc}
   ```

   Keep the existing badge stub and content_shell binders. Add the extension
   binder hook only when an extension is associated with the frame, matching
   app_shell's `PopulateExtensionFrameBinders()` usage. This binder path is not
   expected to be exercised in slice 2 because no extensions are registered yet;
   it will become testable when the PDF component extension is registered in the
   next slice.

7. Update GN narrowly.

   Modify:

   ```text
   chromium/src/content/libtermsurf_chromium/BUILD.gn
   ```

   Expected deps:

   ```text
   //components/keyed_service/core
   //components/prefs
   //components/value_store
   //extensions:extensions_resources
   //extensions/browser
   //extensions/common
   ```

   Before editing GN, verify the exact Chromium 148 value-store target name:

   ```bash
   gn ls out/Default '//components/value_store:*'
   ```

   Additional narrow deps may be added only if the compiler proves they are
   required by the copied Electron/app_shell foundation classes. The experiment
   result must list every additional dep and the source file that forced it.

   Forbidden deps in this experiment:

   ```text
   //chrome/browser/pdf
   //chrome/browser/pdf:pdf
   //chrome/browser/plugins:impl
   //chrome/browser/extensions:extensions
   //chrome/browser/ui
   ```

8. Add structured temporary diagnostics.

   Use Chromium `LOG(INFO)` lines with this exact prefix:

   ```text
   [issue-792-exp2]
   ```

   Required lines, in order:

   ```text
   [issue-792-exp2] extensions-client-set
   [issue-792-exp2] extensions-browser-client-set
   [issue-792-exp2] keyed-service-factories-built
   [issue-792-exp2] extension-prefs-ready context=<ptr> prefs=<ptr>
   [issue-792-exp2] extension-system-init-for-regular-profile context=<ptr>
   [issue-792-exp2] extension-system-finish-initialization context=<ptr>
   [issue-792-exp2] extension-system-ready context=<ptr> registry=<ptr> is_ready=true
   ```

   If the ordering differs, the result must explain why and cite the actual
   lifecycle path. These logs may remain after the experiment if they are gated
   or low-volume; otherwise remove them before archiving.

9. Build the Chromium target.

   Use the Chromium skill build rule:

   ```bash
   autoninja -C out/Default libtermsurf_chromium
   ```

   Do not use `ninja` directly.

10. Archive only after the branch builds.

    After the branch builds and verification passes, commit the Chromium branch
    and archive it under:

    ```text
    chromium/patches/issue-792/
    ```

    Do not archive an incoherent or non-building branch. If the experiment is
    Partial but the branch builds and contains useful foundation work, archive
    it with the result explaining the missing layer.

## Verification

1. Confirm the starting state.

   ```bash
   git status --short
   git -C chromium/src status --short
   git -C chromium/src branch --show-current
   ```

   Chromium must start clean on `148.0.7778.97-issue-784`.

2. Build the baseline before changing code.

   ```bash
   autoninja -C chromium/src/out/Default libtermsurf_chromium
   ```

   This distinguishes an inherited build failure from an experiment-introduced
   failure.

3. Build the library after implementation.

   ```bash
   autoninja -C chromium/src/out/Default libtermsurf_chromium
   ```

4. Run the debug Roamium/Wezboard stack with a normal web page.

   Use the existing debug testing flow from `AGENTS.md`: run the repo-built
   `wezboard-gui`, run the debug `web` binary inside it, and pass the repo-built
   Roamium binary with `--browser`.

5. Verify normal browsing still works.

   Test:
   - `https://example.com` loads;
   - clicking a link still navigates;
   - typing in a text field still works;
   - scrolling still works;
   - DevTools still opens if the current debug workflow supports it.
   - opening a second profile or profile-backed tab still loads a basic page
     without crashing on extension-system calls.

6. Verify the required logs.

   The Chromium log must contain all required `[issue-792-exp2]` lines and no
   crash or DCHECK around extension client setup, keyed-service factory
   creation, `ShellBrowserContext`, `ExtensionRegistry`, or prefs.

7. Verify the extension system exists for the active browser context.

   Add a temporary or retained diagnostic that logs:

   ```text
   [issue-792-exp2] extension-system-ready context=<ptr> registry=<ptr> is_ready=true
   ```

   Pass requires `is_ready=true`.

8. Verify PDFs are unchanged.

   Loading the local Bitcoin PDF fixture is allowed to remain blank, download,
   or behave exactly as it did before this experiment. A visible PDF render is
   not expected and is not required. If PDF behavior changes, the result must
   explain the unexpected change.

9. Run Claude review after recording the result.

   Provide Claude with the experiment file, the Chromium diff, build output
   summary, runtime logs, and the recorded result. Fix all real findings before
   proceeding.

## Pass Criteria

- Chromium branch `148.0.7778.97-issue-792-exp2` builds `libtermsurf_chromium`.
- The extension foundation initializes for the active `ShellBrowserContext`.
- The required `[issue-792-exp2]` logs appear in a coherent lifecycle order.
- `ExtensionSystem::Get(context)` returns a TermSurf extension system with
  `is_ready=true`.
- Normal web browsing smoke tests pass.
- No PDF-specific deps or component-extension registration are added.
- No forbidden Chrome deps are added.
- Claude reviews the completed result and agrees it is good enough to proceed.

## Partial Criteria

Partial if:

- the branch builds but a required foundation service is missing or not ready;
- Chromium 148's `ExtensionsBrowserClient` interface forces a larger atomic
  slice than planned;
- `ShellBrowserContext` is created too early for the extension clients to be
  installed without owning more of the main-parts lifecycle;
- the foundation initializes but a normal browsing regression appears;
- the branch does not build, but the failure precisely identifies the next
  dependency or lifecycle seam to solve.

## Failure Criteria

- The experiment registers the PDF component extension.
- The experiment adds `//chrome/browser/pdf`, `//chrome/browser/pdf:pdf`,
  `//chrome/browser/plugins:impl`, `//chrome/browser/extensions:extensions`, or
  `//chrome/browser/ui`.
- The experiment tries to render PDFs before the extension foundation is ready.
- The experiment copies app_shell's window/app model instead of only its
  extension foundation.
- The experiment modifies Wezboard, Roamium Rust, webtui, or the TermSurf
  protobuf protocol.
- The experiment proceeds without Claude design review or ignores real Claude
  findings.

## Result

**Result:** Pass.

The branch `148.0.7778.97-issue-792-exp2` builds the target
`libtermsurf_chromium`.

Verification performed:

- Baseline branch `148.0.7778.97-issue-784` built `libtermsurf_chromium` before
  changes.
- The Chromium 148 value-store target was verified with
  `gn ls out/Default '//components/value_store:*'`; the usable target is
  `//components/value_store:value_store`.
- After implementation, `autoninja -C out/Default libtermsurf_chromium`
  succeeded.
- A standalone Roamium smoke run with
  `./out/Default/roamium --no-sandbox --disable-gpu --single-process https://example.com`
  emitted the required `[issue-792-exp2]` lifecycle logs for both the regular
  and off-the-record `ShellBrowserContext` objects.
- Both contexts logged `extension-system-ready ... is_ready=1`.
- The first Claude post-review found that the full debug TermSurf path was not
  verified and that the renderer-side extension foundation was missing. The full
  path crashed with
  `Receiver for unknown Channel-associated interface: extensions.mojom.Renderer`.
- The implementation was updated to add `TsContentRendererClient` and
  `TsExtensionsRendererClient`, wiring the extension renderer client from
  `TsMainDelegate::CreateContentRendererClient()`.
- After that fix, the full debug Wezboard + debug `web --browser .../roamium`
  path visibly rendered the local HTML fixture. Artifact:
  `logs/issue-792-exp2-html-20260529-082204/pdf-smoke.png`.
- A non-default profile run using
  `web --profile issue792exp2 --browser .../roamium http://localhost:9616/index.html`
  reached `TabReady`, `UrlChanged`, `TitleChanged`, and `LoadingState` without
  the extension IPC crash. Artifact:
  `logs/issue-792-exp2-profile-20260529-082047/profile-smoke.png`.
- The vendored Bitcoin PDF fixture was loaded through the same debug path.
  Behavior remained the expected pre-PDF-support blank/download path rather than
  a rendered PDF, with no extension IPC crash. Artifact:
  `logs/issue-792-exp2-pdf-20260529-081954/pdf-smoke.png`.

Representative log sequence:

```text
[issue-792-exp2] extensions-client-set
[issue-792-exp2] extensions-browser-client-set
[issue-792-exp2] keyed-service-factories-built
[issue-792-exp2] extension-prefs-ready context=0x8232fc280 prefs=0x822968b60
[issue-792-exp2] extension-system-init-for-regular-profile context=0x8232fc280
[issue-792-exp2] extension-system-finish-initialization context=0x8232fc280
[issue-792-exp2] extension-system-ready context=0x8232fc280 registry=0x8232a9500 is_ready=1
[issue-792-exp2] extension-prefs-ready context=0x8232fca00 prefs=0x8229690a0
[issue-792-exp2] extension-system-init-for-regular-profile context=0x8232fca00
[issue-792-exp2] extension-system-finish-initialization context=0x8232fca00
[issue-792-exp2] extension-system-ready context=0x8232fca00 registry=0x8232a9c80 is_ready=1
[issue-792-exp2] extensions-renderer-client-set
```

Implementation notes:

- The extension clients, API provider, browser client, keyed-service factory,
  renderer client, extension system, and pref service are TermSurf-owned files
  under `content/libtermsurf_chromium/extensions/`.
- The browser-process initialization is wired through
  `TsBrowserMainParts::InitializeBrowserContexts()`, not by overriding
  `PreMainMessageLoopRun()`. This is a deliberate deviation from the original
  design because `InitializeBrowserContexts()` is the narrower content_shell
  lifecycle seam: it lets the base main-parts implementation keep owning the
  rest of startup while inserting extension setup before browser-context service
  creation.
- The browser binder path now delegates extension frame binders only when the
  frame is associated with an extension.
- The renderer-side client is wired through
  `TsMainDelegate::CreateContentRendererClient()`. This registers the extension
  dispatcher's associated Mojo interfaces so browser-to-renderer extension
  messages do not crash ordinary pages.
- The implementation follows app_shell's lifecycle by creating browser-context
  keyed services for the regular context only. The off-the-record context gets
  prefs and an extension system for routing, but it does not create the full
  keyed API service set in this foundation slice.
- The implementation did not register the PDF component extension and did not
  add the forbidden Chrome deps.
- The off-the-record context uses a separate JSON pref file instead of a truly
  in-memory pref store. This is acceptable for this foundation slice, but later
  work should revisit whether off-the-record extension prefs need stricter
  ephemerality.

Additional deps beyond the initially expected list:

- `//components/custom_handlers` — needed because extension services construct
  `ProtocolHandlersManager`, which requires a protocol handler registry.
- `//components/keyed_service/content` — needed for browser-context keyed
  service factories.
- `//components/pref_registry` and `//components/user_prefs` — needed for the
  TermSurf extension pref service.
- `//extensions/browser/api:api_provider` — needed for the browser API provider
  symbols.
- `//extensions/common:core_api_provider` — needed for the common core API
  provider.
- `//extensions/renderer` — needed for the renderer-side extension dispatcher,
  frame helper, and core renderer API provider.

Known caveats:

- The full debug-path smoke was automated by loading pages and inspecting
  screenshots/logs. It proves page load, overlay rendering, title/url/loading
  messages, non-default profile startup, and no extension IPC crash. It did not
  manually exercise link clicks, text entry, scrolling, or DevTools UI
  interaction.
- `libtermsurf_chromium_test` was not used as a gate. It currently fails to link
  on an unrelated existing missing symbol,
  `content::TsNotifyTargetUrlChanged(void*, char const*)`.
- Standalone `roamium --help` without
  `--no-sandbox --disable-gpu --single-process` can still trigger child-process
  sandbox/GPU noise outside the normal TermSurf launch path. The
  extension-foundation logs appear before that noise.
- Chromium logs
  `ERROR:components/leveldb_proto/public/proto_database_provider.h:101] In memory database cannot use the given database directory`.
  This did not block extension-system readiness, but it should be watched in
  later slices.
- The longstanding teardown crash in compositor cleanup can still occur after
  the screenshot/log evidence is captured. The extension-specific shutdown crash
  from the first full-path run
  (`unknown Channel-associated interface: extensions.mojom.Renderer`) is gone.
- Claude reviewed the revised implementation and result, agreed Experiment 2 is
  good enough to mark Pass, and identified only follow-up work for later slices.
- The Chromium branch was committed as `eba06286b8700` and archived under
  `chromium/patches/issue-792/`.

## Conclusion

Experiment 2 proves that the Electron-shaped extension foundation can be added
to TermSurf's content_shell-based embedder without immediately pulling in the
PDF-specific stack. The extension system initializes for both regular and
off-the-record contexts and reaches `is_ready=1`, and the renderer-side
extension dispatcher is now present so ordinary pages survive the browser's
extension renderer IPC.

If Claude post-review agrees the result is sound, the next experiment should
build on this branch and add the next minimal PDF prerequisite on top of the now
initialized extension foundation, rather than revisiting the foundation shape.
