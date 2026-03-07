# `ToggleFullScreen`

Toggles full screen mode for the current window.

```lua
local wezboard = require 'wezboard'

config.keys = {
  {
    key = 'n',
    mods = 'SHIFT|CTRL',
    action = wezboard.action.ToggleFullScreen,
  },
}
```

See also: [native_macos_fullscreen_mode](../config/native_macos_fullscreen_mode.md).

