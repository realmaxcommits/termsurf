---
title: wezboard.strftime_utc
tags:
 - utility
 - time
 - string
---
# `wezboard.strftime_utc(format)`

{{since('20220624-141144-bd1b7c5d')}}

Formats the current UTC date/time into a string using [the Rust chrono
strftime syntax](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html).

```lua
local wezboard = require 'wezboard'

local date_and_time = wezboard.strftime_utc '%Y-%m-%d %H:%M:%S'
wezboard.log_info(date_and_time)
```

See also [strftime](strftime.md) and [wezboard.time](../wezboard.time/index.md).
