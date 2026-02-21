# Issue 611: Rename Ghostty to TermSurf Ghost

## Goal

Rename the app from "Ghostty" to "TermSurf Ghost". The bundle identifier,
display name, product name, CLI text, config paths, and About view all reflect
the new name. Internal identifiers (`GhosttyKit`, `Ghostty.*` Swift namespaces,
`ghostty_*` C API, `Ghostty.xcodeproj`) stay unchanged for upstream merge
safety.

## Background

Ghost is a Ghostty fork. It currently ships with all of Ghostty's branding —
same name, same bundle identifier (`com.mitchellh.ghostty`), same config paths.
This causes real problems:

1. **Icon collision (Issue 610).** macOS Launch Services caches app icons by
   bundle identifier. With `/Applications/Ghostty.app` installed, our app
   inherits Ghostty's icon regardless of what's in our app bundle.
2. **Config collision.** Both apps read from `~/.config/ghostty/`. Changes to
   one affect the other.
3. **Identity.** Users and developers need to distinguish Ghost from upstream
   Ghostty at a glance — in the dock, menu bar, Finder, and CLI.

### Prior art

This rename was done twice before:

- **ts1** created a parallel `termsurf-macos/` directory alongside the upstream
  `macos/`, keeping the originals untouched for clean merges. See
  `docs/issues/500-rename.md` for the full inventory.
- **ts5 (Issue 500)** modified `ts5/macos/` directly (no parallel directory).
  Renamed to "TermSurf" with bundle identifier `com.termsurf`. Internal
  identifiers stayed unchanged. The icon was left as Ghostty's — the same
  problem we're solving now in Issue 610.

Ghost follows ts5's approach: modify `ghost/macos/` directly. The name is
"TermSurf Ghost" (not just "TermSurf") to give this generation its own identity
within the TermSurf family.

### Naming convention

| Context                   | Value                                               |
| ------------------------- | --------------------------------------------------- |
| App name                  | TermSurf Ghost                                      |
| Bundle identifier         | com.termsurf.ghost                                  |
| Bundle identifier (debug) | com.termsurf.ghost.debug                            |
| Config directory          | `~/.config/termsurf-ghost/`                         |
| Config fallback (macOS)   | `~/Library/Application Support/com.termsurf.ghost/` |
| CLI binary name           | `ghostty` (unchanged)                               |
| CLI usage text            | `termsurf-ghost`                                    |
| CLI version output        | `TermSurf Ghost {version}`                          |
| Custom icon path          | `~/.config/termsurf-ghost/Ghost.icns`               |

The CLI binary stays `ghostty` because renaming it requires changes to the Zig
build system, shell completions, and the `ghostty` symlink in the app bundle. A
future issue can tackle that.

## What to change

### 1. Xcode project configuration

In `ghost/macos/Ghostty.xcodeproj/project.pbxproj`:

| Setting                                     | Old                                      | New                        |
| ------------------------------------------- | ---------------------------------------- | -------------------------- |
| `PRODUCT_BUNDLE_IDENTIFIER`                 | `com.mitchellh.ghostty`                  | `com.termsurf.ghost`       |
| `PRODUCT_BUNDLE_IDENTIFIER` (debug)         | `com.mitchellh.ghostty.debug`            | `com.termsurf.ghost.debug` |
| `INFOPLIST_KEY_CFBundleDisplayName`         | `Ghostty`                                | `TermSurf Ghost`           |
| `INFOPLIST_KEY_CFBundleDisplayName` (debug) | `Ghostty[DEBUG]`                         | `TermSurf Ghost[DEBUG]`    |
| `PRODUCT_NAME`                              | `$(TARGET_NAME)` → resolves to `Ghostty` | `TermSurf Ghost`           |
| Permission dialog strings                   | `within Ghostty`                         | `within TermSurf Ghost`    |

Rename files in `ghost/macos/`:

- `Ghostty-Info.plist` → `Ghost-Info.plist`
- `Ghostty.entitlements` → `Ghost.entitlements`
- `GhosttyDebug.entitlements` → `GhosttyDebug.entitlements` (unchanged —
  internal)
- `GhosttyReleaseLocal.entitlements` → `GhosttyReleaseLocal.entitlements`
  (unchanged — internal)

Update file references in `project.pbxproj` for renamed files.

Do NOT rename:

- `Ghostty.xcodeproj/` — internal
- `GhosttyKit.xcframework` — internal
- `Ghostty.icon` — internal (references in `icon.json` stay as-is)
- Any `Ghostty.*` Swift namespaces
- Entitlements files that are only referenced by build settings (internal)

### 2. Info.plist

In `ghost/macos/Ghost-Info.plist` (after rename):

- Change UTType description: `"Ghostty Surface Identifier"` →
  `"TermSurf Ghost Surface Identifier"`
- Menu items already use `$(INFOPLIST_KEY_CFBundleDisplayName)` — they'll
  automatically read "New TermSurf Ghost Tab Here" etc.
- Keep `GHOSTTY_MAC_LAUNCH_SOURCE` and `com.mitchellh.ghosttySurfaceId` as-is
  (internal compatibility)

### 3. CLI text

In `ghost/src/`:

- `src/cli/help.zig` — `"ghostty"` → `"termsurf-ghost"`, `"Ghostty"` →
  `"TermSurf Ghost"`, example commands updated
- `src/cli/version.zig` — `"Ghostty {version}"` → `"TermSurf Ghost {version}"`
- `src/cli/list_themes.zig` — `"👻 Ghostty Theme Preview 👻"` →
  `"🏄 TermSurf Ghost Theme Preview 🏄"`

### 4. Config paths

In `ghost/macos/Sources/`:

- `Ghostty/Ghostty.Config.swift` — Use
  `ghostty_config_load_files(cfg, "termsurf-ghost", "com.termsurf.ghost")`
  instead of `ghostty_config_load_default_files(cfg)`
- `Ghostty/Ghostty.Config.swift` — Custom icon path →
  `~/.config/termsurf-ghost/Ghost.icns`
- `Features/Settings/SettingsView.swift` — Config path and app name in
  instructions: `$HOME/.config/termsurf-ghost/config` and
  `restart TermSurf Ghost`

This requires porting the `ghostty_config_load_files` C API function from ts1,
as upstream Ghostty only has `ghostty_config_load_default_files`. ts5 already
ported this — the implementation can be copied from there:

- `ghost/src/os/macos.zig` — `appSupportDirWithBundleId` function
- `ghost/src/config/Config.zig` — `loadFiles` method
- `ghost/src/config/CApi.zig` — `ghostty_config_load_files` export
- `ghost/include/ghostty.h` — C header declaration

### 5. About view

In `ghost/macos/Sources/Features/About/AboutView.swift`:

- Title: `"Ghostty"` → `"TermSurf Ghost"`
- Subtitle: `"Terminal emulator with integrated browser,\nbuilt on Ghostty."`
- GitHub URL → `https://github.com/termsurf/termsurf`

### 6. Build system

In `ghost/src/build/GhosttyXcodebuild.zig`:

- App path: `Ghostty.app` → `TermSurf Ghost.app`

### 7. Icon (from Issue 610)

The `Ghostty.icon` modification from Issue 610 is already in place
(uncommitted). Once the bundle identifier changes to `com.termsurf.ghost`, macOS
Launch Services will treat this as a new app with no cached icon, and the
surfing ghost should display correctly.

No additional icon work needed beyond what Issue 610 already did.

## What NOT to change

These internal identifiers stay as-is to minimize upstream merge conflicts:

- `Ghostty.xcodeproj/` directory name
- `GhosttyKit.xcframework` framework name
- `Ghostty.*` Swift namespaces (`Ghostty.Config`, `Ghostty.App`, etc.)
- `ghostty_*` C API function names
- `GHOSTTY_MAC_LAUNCH_SOURCE` environment variable
- `com.mitchellh.ghostty.*` notification names in `Package.swift`
- `com.mitchellh.ghosttySurfaceId` UTType identifier
- Swift file names (`AppDelegate+Ghostty.swift`, `Ghostty.Config.swift`, etc.)
- `ghostty` CLI binary name
- `GhosttyDebug.entitlements`, `GhosttyReleaseLocal.entitlements` filenames

## Merge conflict expectations

All changes are in files that upstream Ghostty also modifies. Future
`git subtree pull` may produce conflicts. The conflicts will be small and
predictable — keep our version of the renamed strings, resolve Xcode project
changes manually if upstream restructures build settings.

## Experiments
