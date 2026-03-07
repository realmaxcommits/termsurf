# `QuitApplication`

Terminate the Wezboard application, killing all tabs.

```lua
local wezboard = require 'wezboard'

config.keys = {
  { key = 'q', mods = 'CMD', action = wezboard.action.QuitApplication },
}
```


