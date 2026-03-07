# CopyMode `NextMatch`

{{since('20220624-141144-bd1b7c5d')}}

Move the CopyMode/SearchMode selection to the next matching text, if any.

```lua
local wezboard = require 'wezboard'
local act = wezboard.action

return {
  key_tables = {
    search_mode = {
      { key = 'n', mods = 'CTRL', action = act.CopyMode 'NextMatch' },
    },
  },
}
```
