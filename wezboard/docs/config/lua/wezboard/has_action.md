---
title: wezboard.has_action
tags:
 - utility
 - version
---

# wezboard.has_action(NAME)

{{since('20230408-112425-69ae8472')}}

Returns true if the string *NAME* is a valid key assignment action variant
that can be used with [wezboard.action](action.md).

This is useful when you want to use a wezboard configuration across multiple
different versions of wezboard.

```lua
if wezboard.has_action 'PromptInputLine' then
  table.insert(config.keys, {
    key = 'p',
    mods = 'LEADER',
    action = wezboard.action.PromptInputLine {
      -- other parameters here
    },
  })
end
```
