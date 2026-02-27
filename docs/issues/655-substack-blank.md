# Issue 655: Substack pages go blank after initial render

## Goal

Substack blog posts should render and remain visible, just like they do in
regular Chrome. Currently, the page renders correctly for about one second, then
switches to a blank white screen.

## Background

Pages that reproduce the issue:

- `https://themasonic.substack.com/p/the-investigation`
- `https://kirschsubstack.com/`

These pages load and render correctly in regular Chrome. In TermSurf's Chromium
browser, the page appears for roughly one second — text, images, layout all look
correct — then the entire viewport goes white. The page content is gone.

Confirmed across multiple Substack blogs, so this is a Substack-wide issue —
their shared frontend codebase triggers the problem.

### What we know

- The initial HTML/CSS renders correctly — the page looks fine for ~1 second.
- JavaScript executes after the initial paint and triggers the blank state.
- Regular Chrome renders the same page without issues.
- The issue is specific to TermSurf's Chromium setup (Content API fork, out-of-
  process via XPC, CALayerHost compositing).

### Substack's behavior in regular Chrome

Substack shows a subscription overlay/modal on blog posts for non-subscribers.
This overlay has a semi-transparent backdrop and a signup form. In regular
Chrome, the overlay appears on top of the content and can be dismissed. The
content remains visible behind the overlay.

### Possible causes

1. **Overlay renders as opaque white.** Substack's subscription modal uses a
   backdrop that covers the page. If our Chromium build is missing something the
   overlay needs (a web component, CSS feature, font), the backdrop could render
   as solid white without the dismiss UI, hiding all content underneath.

2. **`<dialog>` element or `backdrop-filter` CSS.** Substack may use the HTML
   `<dialog>` element or CSS `backdrop-filter` for the overlay. If the Content
   API build doesn't fully support these, the overlay could render as an opaque
   white layer.

3. **Service Worker failure.** Substack uses service workers. If registration
   fails in our environment, the page might redirect or blank itself as a
   fallback.

4. **JavaScript feature detection.** Substack's JS might detect a missing
   browser API (e.g., Notification, Push, Payment) and enter an error code path
   that blanks the page.

5. **Navigation or redirect.** The JavaScript might trigger a client-side
   navigation (e.g., to a login page or error page) that our Chromium setup
   doesn't handle correctly, resulting in a blank page. This would be related to
   the navigation issues investigated in Issues 628–632.

### Diagnostic ideas

- Check the Chromium server logs for errors or warnings when the page goes
  blank.
- If DevTools are available, inspect the DOM after the page goes white — is the
  content still in the DOM but hidden by CSS, or has the DOM changed entirely?
- Try loading the page with JavaScript disabled to confirm JS is the trigger.
- Try other Substack blogs to confirm the issue is Substack-wide.
- Compare the user agent string — Substack might serve different content based
  on the UA.
