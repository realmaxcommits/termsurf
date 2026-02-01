# Issue 333: Display profile name in control panel

## Goal

Show the current browser profile name in the webview control panel,
right-aligned.

## Requirements

1. **Position**: Right-aligned in the control panel
2. **Overflow handling**: When the URL is long enough to overlap with the
   profile name:
   - Profile name renders above the tail end of the URL (z-order)
   - URL truncates with ellipsis (`...`) where it would overlap

## Visual Examples

### Normal case (short URL)

```
┌─────────────────────────────────────────────────────┐
│ https://google.com                          default │
└─────────────────────────────────────────────────────┘
```

### Long URL (truncated with ellipsis)

```
┌─────────────────────────────────────────────────────┐
│ https://example.com/very/long/path/to/...   default │
└─────────────────────────────────────────────────────┘
```

## Files Involved

- Control panel rendering code (TBD - need to locate)
