# Issue 734: Consistent build and install scripts

## Goal

Replace the inconsistent collection of build and install scripts with a uniform
CLI that can build, install, and uninstall each component independently (debug
or release), or all together.

## Background

The current `scripts/` directory has grown organically. Each component got its
own script at the time it was added, with no shared conventions:

| Script                | Builds | Installs | Uninstalls | Debug/Release |
| --------------------- | ------ | -------- | ---------- | ------------- |
| `build-debug.sh`      | All    | —        | —          | Debug only    |
| `build-release.sh`    | All    | —        | —          | Release only  |
| `build-roamium.sh`    | Roam   | —        | —          | Either        |
| `install.sh`          | —      | Ghost    | —          | Release only  |
| `install-roamium.sh`  | —      | Roam     | —          | Release only  |
| `install-wezboard.sh` | —      | Wez      | —          | Release only  |

### Problems

1. **No per-component builds.** `build-debug.sh` and `build-release.sh` build
   everything — Ghostboard, Chromium, webtui, and Roamium — with no way to build
   just one. `build-roamium.sh` exists as a one-off exception.
2. **No uninstall.** There is no way to remove installed components. Install
   scripts overwrite previous installs, but leave symlinks, directories, and
   Launch Services registrations behind.
3. **Duplicate logic.** `build-debug.sh` and `build-release.sh` are nearly
   identical (97 lines each), differing only in optimization flags and output
   paths.
4. **Inconsistent naming.** `install.sh` installs Ghostboard but the name
   doesn't say so. `build-roamium.sh` exists but `build-ghostboard.sh` doesn't.
5. **No Wezboard build script.** `install-wezboard.sh` exists but there's no
   corresponding build script.
6. **Mixed concerns.** The monolithic build scripts handle Chromium, Zig, and
   Rust builds in one file, making it impossible to iterate on one component.

### Desired interface

Individual component scripts with a consistent pattern:

```
scripts/build.sh <component> [--release] [--clean] [--open]
scripts/install.sh <component>
scripts/uninstall.sh <component>
```

Where `<component>` is one of: `ghostboard`, `wezboard`, `roamium`, `webtui`,
`chromium`, or `all`.

- `build.sh ghostboard` — debug build of Ghostboard
- `build.sh ghostboard --release` — release build of Ghostboard
- `build.sh all --release --clean` — clean release build of everything
- `install.sh roamium` — install Roamium to system location
- `install.sh all` — install all components
- `uninstall.sh ghostboard` — remove Ghostboard from system
- `uninstall.sh all` — remove all installed components

### Install locations

| Component  | Install location                        | Symlinks                  |
| ---------- | --------------------------------------- | ------------------------- |
| Ghostboard | `/Applications/TermSurf Ghostboard.app` | `/usr/local/bin/termsurf` |
| Wezboard   | `/Applications/Wezboard.app`            | —                         |
| Roamium    | `/usr/local/roamium/`                   | —                         |
| webtui     | Bundled inside Ghostboard app           | `/usr/local/bin/web`      |
| Chromium   | Not installed separately                | —                         |

### Scripts to keep unchanged

These scripts are unrelated to build/install and should not be touched:

- `clean-zig.sh` — Zig-specific cache cleanup
- `generate-icons.sh` — icon asset generation
- `nerd-font-test.sh` — font verification
- `rename-ghostty.sh` — upstream merge rename
- `rename-wezterm.sh` — upstream merge rename

### Scripts to replace

These scripts will be replaced by the new `build.sh`, `install.sh`, and
`uninstall.sh`:

- `build-debug.sh`
- `build-release.sh`
- `build-roamium.sh`
- `install.sh` (current Ghostboard-only installer)
- `install-roamium.sh`
- `install-wezboard.sh`
