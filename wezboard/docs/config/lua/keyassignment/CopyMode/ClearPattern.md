# CopyMode `ClearPattern`

{{since('20220624-141144-bd1b7c5d')}}

Clear the CopyMode/SearchMode search pattern.

```lua
local wezboard = require 'wezboard'
local act = wezboard.action

return {
  key_tables = {
    search_mode = {
      { key = 'u', mods = 'CTRL', action = act.CopyMode 'ClearPattern' },
    },
  },
}
```

