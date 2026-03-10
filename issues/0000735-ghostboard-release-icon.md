# Issue 735: Ghostboard app icons

## Goal

Use `ghostboard-1.png` as the release icon and `ghostboard-1-debug.png` as the
debug icon for Ghostboard.

## Background

The current release icon is generated from `assets/termsurf-2-black-3.png` via
`scripts/generate-icons.sh`. The script runs `sips` to resize the source image
into all required sizes for the `AppIcon.appiconset` and `AppIconImage.imageset`
asset catalogs.

A new icon, `assets/ghostboard-1.png`, has been created for Ghostboard. It
should replace the current icon in release builds. A debug variant
(`ghostboard-1-debug.png`) also exists in `assets/`.

### Icon pipeline

1. `scripts/generate-icons.sh [source]` takes a source PNG and generates:
   - `ghostboard/macos/Assets.xcassets/AppIcon.appiconset/icon-{16,32,64,128,256,512,1024}.png`
   - `ghostboard/macos/Assets.xcassets/AppIconImage.imageset/macOS-AppIcon-{256px-128pt@2x,512px,1024px}.png`
2. The script defaults to `assets/termsurf-2-black-3.png` when no argument is
   given.

### Debug icon

The debug icon is a single PNG at
`ghostboard/macos/Assets.xcassets/TermSurfDebugIcon.imageset/termsurf-debug-icon.png`.
In debug builds, `AppDelegate.swift` sets the dock icon at runtime via
`NSImage(named: "TermSurfDebugIcon")`. The new debug source is
`assets/ghostboard-1-debug.png`.

### What needs to change

- Update `generate-icons.sh` to default to `assets/ghostboard-1.png` instead of
  `assets/termsurf-2-black-3.png`.
- Regenerate all release icon assets from the new source.
- Replace the debug icon PNG in `TermSurfDebugIcon.imageset/` with
  `ghostboard-1-debug.png`.
