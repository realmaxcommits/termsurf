# Experiment 5: Apply the macOS-Only GhosttyKit Build Patch

## Description

Experiment 4 proved that the pristine imported Ghostty `v1.3.1` tree does not
currently build the macOS app on this VM with the documented default build flow.
Prior work in Issue 802 found the same family of Ghostty/Zig/Xcode SDK problem
and resolved it without downgrading Xcode by applying a small build-only patch:
build only the macOS `GhosttyKit` slice and do not construct the iOS or
iOS-simulator slices for a native macOS app build.

This experiment applies that known workaround to the fresh `ghostboard/` import,
then verifies that Ghostty `v1.3.1` app/runtime code builds and runs on macOS.
This is no longer a pristine-upstream baseline; it is an upstream Ghostty
app/runtime baseline with one documented build-system deviation.

## Changes

- `ghostboard/src/build/GhosttyXCFramework.zig` — gate construction of the iOS
  and iOS-simulator `GhosttyLib` values so they are only built for the universal
  xcframework target. For the native target, keep only the macOS `GhosttyKit`
  slice.

This should be the only source change under `ghostboard/`. It must not change
branding, config paths, CLI names, icons, protocol code, Swift app behavior,
runtime Zig code, `webtui`, or `roamium`.

The implementation should follow the prior successful patch:

- `scripts/ghostty-app/macos-only-xcframework.patch`
- Issue 802 Experiment 3:
  `issues/0802-libroastty-completion-and-mac-app/03-macos-only-build.md`

## Verification

1. Apply the build-only patch.
2. Run Zig formatting on the patched Zig file.
3. Build the native macOS-only `GhosttyKit` framework:

   ```bash
   cd ghostboard
   zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false
   ```

4. Build the macOS app with Xcode:

   ```bash
   cd ghostboard/macos
   xcodebuild -target Ghostty -configuration Debug -arch arm64 \
     COMPILATION_CACHE_CAS_PATH="$HOME/Library/Developer/Xcode/DerivedData/CompilationCache.noindex" \
     COMPILATION_CACHE_KEEP_CAS_DIRECTORY=YES
   ```

5. If the app builds, launch it by absolute path, confirm a `ghostty` process is
   running from the built app bundle, then terminate only that built app
   process.
6. Confirm the `ghostboard/` diff is limited to
   `src/build/GhosttyXCFramework.zig`.

Pass criteria:

- The only source change under `ghostboard/` is the build-only
  `GhosttyXCFramework.zig` patch.
- `zig fmt` accepts the patched file.
- `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`
  succeeds.
- `xcodebuild -target Ghostty -configuration Debug -arch arm64` succeeds.
- `ghostboard/macos/build/Debug/Ghostty.app/Contents/MacOS/ghostty` exists.
- The built app launches and produces a scoped process that can be terminated.

Fail criteria:

- More `ghostboard/` source files must be changed.
- The app still fails to build or launch.
- The workaround requires Ghostboard branding, config, protocol, Swift runtime,
  `webtui`, or `roamium` changes.
- The result does not clearly document that this is a build-only deviation from
  pristine upstream Ghostty.

## Notes

If this passes, later experiments can begin the actual Ghostboard port on top of
a proven local macOS build, with the build-only deviation explicitly recorded.

## Design Review

Fresh-context adversarial review returned `APPROVED`.

- No required findings were reported.
- Optional finding accepted: make `-Demit-xcframework=true` explicit in the Zig
  build command rather than relying on Ghostty's default emit behavior.
- Nit accepted for implementation: do not copy the prior Roastty-specific
  comment verbatim into `ghostboard/`; use an Issue 808 comment if a comment is
  needed.
