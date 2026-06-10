# Roastty App Automation Helpers

Helpers in this directory drive or inspect the copied, renamed Roastty macOS app
for Issue 802 experiments.

## Screenshot Policy

Screenshots are never committed. The screenshot wrapper writes captures outside
the repo to `${TERMSURF_SHOT_DIR:-$HOME/.cache/termsurf/shots}` and prints the
PNG path. Keep retained images outside the working tree.

## PNG Diff

`pngdiff.swift` compares two PNG captures and writes one JSON object to stdout.
Diagnostics and usage errors go to stderr.

```bash
swift scripts/roastty-app/pngdiff.swift expected.png actual.png
swift scripts/roastty-app/pngdiff.swift expected.png actual.png \
  --max-mismatch-ratio 0.01 \
  --max-mean-channel-delta 2.0
```

The helper exits `0` when the metrics are within the supplied thresholds and
nonzero on threshold failure, dimension mismatch, invalid input, or unreadable
images.
