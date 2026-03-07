#!/bin/bash
set -x
name="$1"

notes=$(cat <<EOT
See https://wezboard.org/changelog.html#$name for the changelog

If you're looking for nightly downloads or more detailed installation instructions:

[Windows](https://wezboard.org/install/windows.html)
[macOS](https://wezboard.org/install/macos.html)
[Linux](https://wezboard.org/install/linux.html)
[FreeBSD](https://wezboard.org/install/freebsd.html)
EOT
)

gh release view "$name" || gh release create --prerelease --notes "$notes" --title "$name" "$name"
