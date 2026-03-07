---
title: wezboard.home_dir
tags:
 - utility
 - filesystem
---

# `wezboard.home_dir`

This constant is set to the home directory of the user running `wezboard`.

```lua
local wezboard = require 'wezboard'
wezboard.log_error('Home ' .. wezboard.home_dir)
```


