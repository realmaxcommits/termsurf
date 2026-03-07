---
title: wezboard.config_dir
tags:
 - filesystem
---

# `wezboard.config_dir`

This constant is set to the path to the directory in which your `wezboard.lua`
configuration file was found.

```lua
local wezboard = require 'wezboard'
wezboard.log_error('Config Dir ' .. wezboard.config_dir)
```


