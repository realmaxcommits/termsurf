+++
status = "open"
opened = "2026-05-23"
+++

# Issue 784: Datalist suggestions do not open

## Goal

Make `<input list="...">` datalist suggestions work in TermSurf's Chromium
engine, without regressing the native popup fixes completed in the preceding
native-popup issues.

After the datalist bug is fixed, perform the remaining native-popup diagnostic
log cleanup that is safe to remove.

## Background

This is the final known native popup bug from the current series.

The earlier work established and fixed several distinct native-popup failure
modes:

- [Issue 779](../0779-date-picker-popup-position/README.md) fixed PagePopup
  placement for date/time/color-style controls. The critical invariant from that
  issue is the `WebPagePopupImpl::SetWindowRect` y-axis correction: when Blink
  asks to place a PagePopup at the input's bottom edge, TermSurf corrects the
  popup y back to the input anchor y before passing the rect downstream.
- [Issue 782](../0782-native-popup-followups/README.md) fixed native widgets
  stopping after `<select>` interactions. The root cause was an invisible
  Chromium Shell window overlapping Wezboard while still accepting AppKit mouse
  events. The fix made TermSurf-managed Shell windows consistently
  mouse-transparent with `setIgnoresMouseEvents:YES`.
- [Issue 783](../0783-native-popup-remainders/README.md) fixed PagePopup
  dismissal on Cmd-Tab and the `<select>` x-position bug. Cmd-Tab dismissal now
  flows through `SetGuiActive`, and selects use direct `NSMenu` placement rather
  than the `NSPopUpButtonCell` path that shifted the menu left.

The native popup test page still includes one failing control: the datalist
input. The field accepts text, but the browser suggestions do not appear. This
appears to be a different widget family than the fixed PagePopup and select-menu
paths, so it should be investigated independently.

## Known Good Invariants

Do not regress these while fixing datalist:

- date/time/date-time/color PagePopup y-position remains correct;
- date/time/date-time/color PagePopups dismiss on Cmd-Tab;
- native widgets still open after a select interaction;
- select dropdown x-position remains correct with direct `NSMenu`;
- Shell windows remain mouse-transparent with `setIgnoresMouseEvents:YES`;
- `SetGuiActive` continues to restore page focus on app reactivation.

If any of these regress, stop and fix the regression before continuing.

## Initial Analysis

Datalist suggestions may use a path that differs from both:

- Blink PagePopup controls, such as `DateTimeChooserImpl` and related
  `WebPagePopupImpl` flows; and
- `<select>` menus, which flow through `RenderFrameHostImpl::ShowPopupMenu`,
  `PopupMenuHelper`, `RenderWidgetHostNSViewBridge::DisplayPopupMenu`, and
  `WebMenuRunner`.

The first experiment should identify which Chromium path a datalist suggestion
attempt takes on macOS in TermSurf:

- whether Blink attempts to open a PagePopup;
- whether it uses an Autofill-style popup;
- whether it sends a browser-side popup request that is suppressed;
- whether input/focus state prevents the datalist trigger from reaching the
  suggestion-open path;
- whether a popup opens but is hidden, offscreen, transparent, behind another
  window, or immediately dismissed.

Only after that path is known should the issue attempt a fix.

## Experiments

### Experiment 1: Code analysis of the datalist popup path

#### Description

Before adding more logs, analyze Chromium's datalist implementation to identify
which subsystem owns `<input list>` suggestions and where Roamium/content_shell
is likely missing support.

This experiment is code analysis only. It must not modify Chromium, Roamium,
Wezboard, the protocol, or the test page.

#### Changes

No code changes.

Read the relevant Chromium paths:

- Blink form-control trigger:
  `third_party/blink/renderer/core/html/forms/text_field_input_type.cc`
- Blink chrome client bridge:
  `third_party/blink/renderer/core/page/chrome_client_impl.cc`
- Blink Autofill client interface:
  `third_party/blink/public/web/web_autofill_client.h`
- Renderer Autofill implementation:
  `components/autofill/content/renderer/autofill_agent.cc`
- Browser Autofill suggestion path:
  `components/autofill/core/browser/ui/autofill_external_delegate.cc`
- Chrome renderer setup: `chrome/renderer/chrome_content_renderer_client.cc`
- content_shell renderer setup:
  `content/shell/renderer/shell_content_renderer_client.cc`
- content_shell main delegate: `content/shell/app/shell_main_delegate.cc`

#### Verification

The analysis is complete when the issue records:

- which Chromium subsystem owns datalist suggestions;
- whether datalist uses PagePopup, `<select>` menu plumbing, or another popup
  family;
- the first likely missing link in Roamium/content_shell;
- whether existing native-popup logs are expected to fire for datalist;
- the smallest useful logging plan for the next experiment.

**Result:** Pass

Datalist suggestions use Chromium's Autofill suggestion infrastructure, not the
PagePopup path used by date/time/color controls and not the AppKit menu path
used by `<select>`.

The normal Blink trigger is:

1. `DataListIndicatorElement::DefaultEventHandler(...)` or
   `TextFieldInputType::OpenPopupView()` decides the datalist suggestions should
   open.
2. Blink calls
   `ChromeClientImpl::OpenTextDataListChooser(HTMLInputElement& input)`.
3. `ChromeClientImpl` calls `AutofillClientFromFrame(...)`.
4. If a `WebAutofillClient` exists, Blink calls
   `fill_client->OpenTextDataListChooser(WebInputElement(&input))`.
5. Chromium's `AutofillAgent::OpenTextDataListChooser(...)` calls
   `ShowSuggestions(...)` with trigger source `kOpenTextDataListChooser`.
6. The Autofill browser side eventually reaches
   `AutofillExternalDelegate::OnQuery(...)` and
   `AutofillExternalDelegate::ShowSuggestions(...)`, where datalist options are
   inserted into the suggestion list and shown through the Autofill popup UI.

The important difference is the renderer setup. Chrome installs Autofill support
in `ChromeContentRendererClient::RenderFrameCreated(...)`:

- it creates a `PasswordAutofillAgent`;
- it creates a `PasswordGenerationAgent`;
- it constructs `new AutofillAgent(...)`;
- the `AutofillAgent` constructor calls
  `render_frame->GetWebFrame()->SetAutofillClient(this)`.

Roamium is based on content_shell, not Chrome. content_shell's
`ShellMainDelegate::CreateContentRendererClient()` creates a
`ShellContentRendererClient`, and
`ShellContentRendererClient::RenderFrameCreated(...)` only installs
`ShellRenderFrameObserver`. It does not install `AutofillAgent`, and therefore
does not appear to install a `WebAutofillClient` on the Blink frame.

That makes the most likely failure boundary:

```text
ChromeClientImpl::OpenTextDataListChooser(...)
  -> AutofillClientFromFrame(frame) returns null
  -> no AutofillAgent::OpenTextDataListChooser(...)
  -> no browser-side Autofill query
  -> no visible datalist suggestions
```

This also explains why the existing native-popup logs did not settle the issue.
The PagePopup logs from Issue 779 and the `<select>` menu logs from Issues 782
and 783 are on the wrong widget families. There is an existing
`[issue-779-trace] AutofillExternalDelegate::ShowSuggestions` log in the browser
Autofill path, but if the renderer has no `WebAutofillClient`, execution never
reaches it.

#### Conclusion

The datalist bug is most likely missing Autofill plumbing in the content_shell
embedding used by Roamium. This is not a geometry bug, an AppKit popup placement
bug, or a PagePopup lifecycle bug.

The next experiment should be a narrow logging pass that proves or disproves the
missing-client boundary:

- log `ChromeClientImpl::OpenTextDataListChooser(...)` with whether
  `AutofillClientFromFrame(...)` is null;
- log `ChromeClientImpl::TextFieldDataListChanged(...)` with the same client
  presence check;
- log `ShellContentRendererClient::RenderFrameCreated(...)` so the trace proves
  the content_shell renderer client is the active renderer client;
- log `AutofillAgent::OpenTextDataListChooser(...)` and
  `AutofillAgent::ShowSuggestions(...)`, if reached;
- log whether a browser-side `ContentAutofillClient` exists for the Shell
  `WebContents`, if a renderer Autofill query reaches the browser.

Expected result: the Blink datalist trigger fires, but
`AutofillClientFromFrame(...)` is null. If confirmed, the fix should be designed
around adding the minimal Autofill/datalist support required by the
content_shell/Roamium embedding, without importing the full Chrome browser UI.

## Cleanup Requirement

Do not perform broad log cleanup before the datalist fix. Some remaining
native-popup traces may still be useful for identifying the datalist path.

After datalist suggestions work and the known-good invariants above are
verified, perform a dedicated cleanup pass:

- remove obsolete diagnostic logs from Issues 779, 782, and 783 that no longer
  serve the datalist investigation;
- preserve low-volume logs only if they are still useful for future popup
  regression diagnosis;
- do not remove the behavioral fixes that were introduced in the same commits as
  trace code;
- regenerate the Chromium patch archive after any Chromium cleanup commit.

The cleanup must be done by reviewed hunks, not by blanket-reverting historical
trace commits.
