# CopyMode `MoveForwardSemanticZone`

{{since('20220903-194523-3bb1ed61')}}

Moves the CopyMode cursor position one semantic zone to the right.

See [Shell Integration](../../../../shell-integration.md) for more information
about semantic zones.

```lua
local wezboard = require 'wezboard'
local act = wezboard.action

return {
  key_tables = {
    copy_mode = {
      {
        key = 'Z',
        mods = 'NONE',
        action = act.CopyMode 'MoveForwardSemanticZone',
      },
    },
  },
}
```


