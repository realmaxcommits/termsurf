---
title: wezboard.config_file
tags:
 - filesystem
---

# `wezboard.config_file`

{{since('20210502-130208-bff6815d')}}

This constant is set to the path to the `wezboard.lua` that is in use.

```lua
local wezboard = require 'wezboard'
wezboard.log_info('Config file ' .. wezboard.config_file)
```



