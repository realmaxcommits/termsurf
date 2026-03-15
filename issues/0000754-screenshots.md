# Issue 754: Screenshot pipeline and homepage hero

## Goal

Add `assets/screenshot2.png` to the README (for GitHub) and to the website
homepage as a hero image. Create a reproducible pipeline to convert screenshots
to lossless WebP for the website. This pipeline will be reused for all future
screenshots on the website, docs, and blog.

## Background

### The screenshot

`assets/screenshot2.png` is a new screenshot of TermSurf in action. It should
appear in two places:

1. **README.md** — PNG, referenced directly from `assets/`. This is what shows
   up on GitHub.
2. **Website homepage** — Lossless WebP, served from the website's public
   assets.

### Why WebP

PNG screenshots from macOS Retina displays are large (often 2–5 MB). Lossless
WebP produces identical quality at ~30–50% smaller file sizes. Every screenshot
on the website should be WebP for faster page loads.

### Reproducible conversion

We need a script in `scripts/` that converts PNG screenshots to lossless WebP.
This script will be used every time a new screenshot is added. The website
already uses `sharp` for icon processing (`website/scripts/process-icons.ts`),
so we can use the same library — or use `cwebp` from the command line if
available.

### What needs to happen

1. Replace the old screenshot in `README.md` with the new one
2. Create a script to convert PNG screenshots to lossless WebP
3. Convert `assets/screenshot2.png` to WebP and place it in
   `website/public/images/`
4. Add the screenshot to the website homepage (`website/src/routes/index.tsx`)

## Experiments

### Experiment 1: Update README screenshot

#### Description

Replace the existing `assets/screenshot.png` reference in `README.md` with
`assets/screenshot2.png`.

#### Changes

**`README.md`**

Change the image reference on line 12 from `assets/screenshot.png` to
`assets/screenshot2.png`. Update the alt text if needed.

#### Verification

View the README on GitHub (or locally) and confirm the new screenshot appears.
