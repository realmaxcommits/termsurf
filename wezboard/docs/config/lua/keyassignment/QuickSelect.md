# `QuickSelect`

{{since('20210502-130208-bff6815d')}}

Activates [Quick Select Mode](../../../quickselect.md).

```lua
local wezboard = require 'wezboard'

config.keys = {
  { key = ' ', mods = 'SHIFT|CTRL', action = wezboard.action.QuickSelect },
}
```

See also [QuickSelectArgs](QuickSelectArgs.md)
