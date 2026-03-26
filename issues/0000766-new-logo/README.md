+++
status = "open"
opened = "2026-03-26"
+++

# Issue 766: Update icons with termsurf-12 logo

## Goal

Replace the dock icon (Wezboard) and website icon with the new termsurf-12 logo.

## Background

Two new logo files are available in `assets/`:

- `assets/termsurf-12.png` — New dock icon source (with background)
- `assets/termsurf-12-transparent.png` — New website icon source (transparent)

### Dock icon (Wezboard)

The current dock icon is at
`wezboard/assets/macos/TermSurf Wezboard.app/Contents/Resources/wezboard.icns`.
The icon generation script at `wezboard/assets/icon/update.sh` converts a source
image to multiple sizes and generates `.icns` and `.ico` files using ImageMagick
and `png2icns`.

### Website icon

The current website icon source is
`website/raw-icons/termsurf-11-transparent.png`. The processing script at
`website/scripts/process-icons.ts` uses Sharp to generate:

- `website/public/images/termsurf-11-transparent-192.png` (192px PNG)
- `website/public/images/termsurf-11-transparent-32.ico` (32px ICO)
- `website/public/favicon.ico`

## Experiments

### Experiment 1: Update dock icon and website icon

#### Description

Replace both icons with the new termsurf-12 logo. Use the existing generation
scripts to produce the correct sizes and formats.

#### Changes

**1. Dock icon**

Use `assets/termsurf-12.png` as the new source. Run the icon generation script
to produce the `.icns` file:

```bash
cd wezboard/assets/icon
# Update script to point at new source, or copy source in
cp ../../../assets/termsurf-12.png ./termsurf-12.png
./update.sh termsurf-12.png
```

Verify `wezboard.icns` is updated, then copy it into the app bundle:

```bash
cp wezboard.icns ../macos/TermSurf\ Wezboard.app/Contents/Resources/wezboard.icns
```

**2. Website icon**

Copy the transparent version into the website raw-icons directory:

```bash
cp assets/termsurf-12-transparent.png website/raw-icons/
```

Run the icon processing script to generate all sizes:

```bash
cd website
npx tsx scripts/process-icons.ts
```

Update any references from `termsurf-11-transparent` to
`termsurf-12-transparent` in the website source code (check `src/util/icons.ts`
and any templates that reference the old filename).

#### Verification

| #   | Test                    | Steps                                          | Expected                                 |
| --- | ----------------------- | ---------------------------------------------- | ---------------------------------------- |
| 1   | Dock icon updated       | Build and launch Wezboard, check dock          | New termsurf-12 logo appears in dock     |
| 2   | Website favicon updated | Run website dev server, check browser tab icon | New termsurf-12 transparent logo appears |
| 3   | Website 192px icon      | Check website manifest/meta tags               | 192px icon uses new logo                 |
