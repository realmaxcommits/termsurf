---
title: wezboard.running_under_wsl
tags:
 - utility
---
# `wezboard.running_under_wsl()`

This function returns a boolean indicating whether we believe that we are
running in a Windows Services for Linux (WSL) container.  In such an
environment the `wezboard.target_triple` will indicate that we are running in
Linux but there will be some slight differences in system behavior (such as
filesystem capabilities) that you may wish to probe for in the configuration.

```lua
local wezboard = require 'wezboard'
wezboard.log_error(
  'System '
    .. wezboard.target_triple
    .. ' '
    .. tostring(wezboard.running_under_wsl())
)
```


