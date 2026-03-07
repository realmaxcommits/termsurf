---
tags:
  - multiplexing
---
# `daemon_options`

Allows configuring the multiplexer (mux) server and how it places itself into
the background to run as a daemon process.

You should not normally need to configure this setting; the defaults should
be sufficient in most cases.

There are three fields supported:
* `pid_file` - specify the location of the pid and lock file.  The default location is `$XDG_RUNTIME_DIR/wezboard/pid` on X11/Wayland systems, or `$HOME/.local/share/termsurf/wezboard/pid`
* `stdout` - specifies where a log of the stdout stream from the daemon will be placed.  The default is `$XDG_RUNTIME_DIR/wezboard/stdout` on X11/Wayland systems, or `$HOME/.local/share/termsurf/wezboard/stdout`.
* `stderr` - specifies where a log of the stderr stream from the daemon will be placed.  The default is `$XDG_RUNTIME_DIR/wezboard/stderr` on X11/Wayland systems, or `$HOME/.local/share/termsurf/wezboard/stderr`.

```lua
config.daemon_options = {
  stdout = '/some/where/stdout',
  stderr = '/some/where/stderr',
  pid_file = '/some/where/pid_file',
}
```
