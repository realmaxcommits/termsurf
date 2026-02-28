# Issue 672: Inner Padding for Pane Borders

The pane border from Issue 669 renders as an overlay in the ZStack, covering
content along all four edges. Terminal text and the loading progress bar are
partially obscured by the border. Add inner padding to push content inward by
the border width so nothing is covered.

## Background

Issue 669 added configurable pane borders via `Rectangle().strokeBorder()` in
the ZStack. The border sits on top of the `SurfaceRepresentable` (Metal
renderer) and any other overlays. With `split-border-width = 2`, 2 points of
content are hidden behind the border on each edge.

Ghostty's split divider avoids this problem by **allocating space** in the
layout — it reduces pane rects in `SplitView.swift` so the divider sits in
dedicated space and never covers content. Our border uses an overlay pattern
instead, so we need to inset the content.

## Approach

Reduce the size passed to `SurfaceRepresentable` by the border width on each
side, and offset the representable inward. This avoids adding layout modifiers
(like `.padding()`) directly to the representable — we just adjust the numbers
it already receives.

The progress bar also needs to be inset so it doesn't render behind the border.

No new config options needed — the existing `split-border-width` drives the
padding automatically.

## Experiment 1: Inset content by border width

### Hypothesis

Passing a reduced size to `SurfaceRepresentable` and offsetting it by the border
width will inset the terminal content without breaking resize. The progress bar
can be inset with `.padding()` on its container.

### Changes

#### 1. SurfaceView.swift — inset the SurfaceRepresentable

In the `GeometryReader`, compute the border inset and adjust the size and
position:

```swift
GeometryReader { geo in
    let borderInset = isSplit ? termsurf.config.splitBorderWidth : 0
    let insetSize = CGSize(
        width: max(10, geo.size.width - borderInset * 2),
        height: max(10, geo.size.height - borderInset * 2)
    )

    SurfaceRepresentable(view: surfaceView, size: insetSize)
        .frame(width: insetSize.width, height: insetSize.height)
        .offset(x: borderInset, y: borderInset)
        .focused($surfaceFocus)
        .saturation(...)
        // ... remaining existing modifiers
```

The `max(10, ...)` guard prevents degenerate sizes if the border is wider than
the pane.

#### 2. SurfaceView.swift — inset the progress bar

Add horizontal and vertical padding to the progress bar container so it doesn't
render behind the border:

```swift
if let progressReport = surfaceView.progressReport, progressReport.state != .remove {
    let borderInset = isSplit ? termsurf.config.splitBorderWidth : 0
    VStack(spacing: 0) {
        SurfaceProgressBar(report: progressReport)
        Spacer()
    }
    .padding(borderInset)
    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    .allowsHitTesting(false)
    .transition(.opacity)
}
```

### Test

1. `cd gui && zig build` — compiles without errors.
2. Open TermSurf, create a split, set `split-border-width = 2`.
3. Terminal content is fully visible — no text clipped by the border.
4. The border frames the content with no gap.
5. Navigate to a URL — progress bar renders inside the border, not behind it.
6. **Resize the window** — panes resize correctly.
7. **Open a new split** — existing pane resizes correctly.
8. Set `split-border-width = 0` — no padding, no border (backward compatible).
9. Set `split-border-width = 4` — larger padding, content still fully visible.
10. Verify unfocused dimming and saturation still work alongside padding.
