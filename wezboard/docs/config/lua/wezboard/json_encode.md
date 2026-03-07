---
title: wezboard.json_encode
tags:
 - utility
 - json
---

# `wezboard.json_encode(value)`

{{since('20220807-113146-c2fee766')}}

Encodes the supplied lua value as json:

```
> wezboard.json_encode({foo = "bar"})
"{\"foo\":\"bar\"}"
```
