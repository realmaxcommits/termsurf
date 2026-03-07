---
title: wezboard.truncate_right
tags:
 - utility
 - string
---
# wezboard.truncate_right(string, max_width)

{{since('20210502-130208-bff6815d')}}

Returns a copy of `string` that is no longer than `max_width` columns
(as measured by [wezboard.column_width](column_width.md)).

Truncation occurs by reemoving excess characters from the right end
of the string.

For example, `wezboard.truncate_right("hello", 3)` returns `"hel"`,

See also: [wezboard.truncate_left](truncate_left.md), [wezboard.pad_left](pad_left.md).
