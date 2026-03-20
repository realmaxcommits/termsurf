+++
status = "open"
opened = "2026-03-20"
+++

# Issue 764: Move profile label to bottom-right with engine name

## Goal

The viewport bottom-right should show `[avatar] profile/engine` (e.g.,
`👤 default/roamium`). Remove the profile from the top-right.

## Background

### Current layout

- **Top-left:** Page title (or "Viewport")
- **Top-right:** `👤 default` (profile icon + name)
- **Bottom-left:** Hover URL (when hovering a link)
- **Bottom-right:** `roamium` (engine name, dimmed)

### Target layout

- **Top-left:** Page title (or "Viewport")
- **Top-right:** _(empty)_
- **Bottom-left:** Hover URL (when hovering a link)
- **Bottom-right:** `👤 default/roamium` (profile icon + name + slash + engine)

### Scope

TUI-only change. One section in the `ui()` function in `webtui/src/main.rs`.
