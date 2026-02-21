# Issue 612: App icon

## Goal

The app icon in the dock, Finder, and app switcher shows the TermSurf surfing
ghost icon for both release and debug builds. Release shows a cyan wave, debug
shows a green wave.

## Background

Issue 610 attempted to replace the Ghostty icon but failed because macOS Launch
Services caches icons by bundle identifier. Our app shared
`com.mitchellh.ghostty` with the official Ghostty installation, so macOS always
served Ghostty's cached icon regardless of what was in our app bundle.

Issue 611 resolved the blocker by changing the bundle identifier to
`com.termsurf`. macOS now treats TermSurf as a distinct app with no cached icon.

### Issue 610's experiments

Five experiments explored icon replacement:

1. **Replaced `AppIconImage.imageset` PNGs** — Partial. Debug override worked
   (dock), but `AppIconImage.imageset` is not the bundle icon source.
2. **Replaced `Ghostty.icon` with minimal Icon Composer doc** — Failed. Minimal
   `icon.json` produced a degraded 256x256 `.icns`.
3. **Swapped ghost layer in original Icon Composer doc** — The `.icns` was
   correct (verified with `iconutil`), but macOS served a cached icon.
4. **Clean build with cache clearing** — Same result. Bundle ID collision.
5. **Release build** — Same result. `Assets.car` also cached.

All experiments failed due to the bundle ID collision, not the icon changes
themselves. Experiment 3's approach produced a correct `.icns`, but the Icon
Composer format is complex and fragile. None of those changes were committed.

### ts1's approach

ts1 solved this problem simply. Instead of using Ghostty's Icon Composer
(`.icon`) format, ts1 uses a traditional `AppIcon.appiconset/` with pre-rendered
PNGs at standard macOS sizes:

| File            | Pixels    | Used for             |
| --------------- | --------- | -------------------- |
| `icon-16.png`   | 16x16     | 16pt @1x             |
| `icon-32.png`   | 32x32     | 16pt @2x, 32pt @1x   |
| `icon-64.png`   | 64x64     | 32pt @2x             |
| `icon-128.png`  | 128x128   | 128pt @1x            |
| `icon-256.png`  | 256x256   | 128pt @2x, 256pt @1x |
| `icon-512.png`  | 512x512   | 256pt @2x, 512pt @1x |
| `icon-1024.png` | 1024x1024 | 512pt @2x            |

The Xcode project references this appiconset via
`ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon`. This bypasses Icon Composer
entirely — `actool` compiles the PNGs directly into `Assets.car`.

For debug builds, ts1 uses a runtime override in `AppDelegate.swift`:

```swift
#if DEBUG
  if appIcon == nil {
    NSApplication.shared.applicationIconImage = NSImage(named: "TermSurfDebugIcon")
  }
#endif
```

This sets the dock icon at runtime without modifying the app bundle (preserving
code signing). The `TermSurfDebugIcon` is a regular imageset containing only the
debug icon PNG.

### What needs to change in Ghost

Ghost currently uses `ASSETCATALOG_COMPILER_APPICON_NAME = Ghostty`, which
points to `ghost/images/Ghostty.icon/` — an Icon Composer bundle. This needs to
change to a traditional `AppIcon.appiconset/` using the ts1 icon files.

**Source images** (already in ts1):

- `ts1/termsurf-macos/icon-source/termsurf-icon.png` — Release icon (cyan wave)
- `ts1/termsurf-macos/icon-source/termsurf-debug-icon.png` — Debug icon (green
  wave)

**Pre-rendered sizes** (already in ts1):

- `ts1/termsurf-macos/Assets.xcassets/AppIcon.appiconset/icon-*.png` — 7 sizes
  from 16px to 1024px

These exact files will be copied into Ghost's asset catalog. No image generation
or resizing needed.

### Key files to change

1. **`ghost/macos/Assets.xcassets/AppIcon.appiconset/`** — New directory. Copy
   all PNGs and `Contents.json` from ts1.
2. **`ghost/macos/Assets.xcassets/TermSurfDebugIcon.imageset/`** — New
   directory. Copy from ts1.
3. **`ghost/macos/Ghostty.xcodeproj/project.pbxproj`** — Change
   `ASSETCATALOG_COMPILER_APPICON_NAME` from `Ghostty` to `AppIcon`.
4. **`ghost/macos/Sources/App/macOS/AppDelegate.swift`** — Change debug icon
   from `"BlueprintImage"` to `"TermSurfDebugIcon"`.
5. **`ghost/macos/Assets.xcassets/AppIconImage.imageset/`** — Replace PNGs with
   TermSurf icon (used by runtime icon-switching system, not the bundle icon).
