# Troubleshooting

## Review logs/error messages

If things aren't working out, there may be an issue printed in the logs.
Read on to learn more about how to see those logs.

### Debug Overlay

By default, pressing <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>L</kbd> will activate
the debug overlay and allow you to review the most recently logged issues.
It also gives you access to a Lua REPL for evaluating built-in lua functions.

See [ShowDebugOverlay](config/lua/keyassignment/ShowDebugOverlay.md) for more
information on this key assignment.

### Log Files

You can find log files in `$XDG_RUNTIME_DIR/wezboard` on unix systems,
or `$HOME/.local/share/termsurf/wezboard` on macOS and Windows systems.

### Increasing Log Verbosity

The `WEZBOARD_LOG` environment variable can be used to adjust the level
of logging for different modules within wezboard.

To see maximum verbosity, you can start wezboard like this:

```
WEZBOARD_LOG=debug wezboard
```

to see debug level logs for everything on stdout.

On Windows systems you'll usually need to set the environment variable separately:

Using `cmd.exe`:

```
C:\> set WEZBOARD_LOG=debug
C:\> wezboard
```

Using powershell:

```
PS C:\> $env:WEZBOARD_LOG="debug"
PS C:\> wezboard
```

When using a flatpak you must first enter the flatpak container by running:

```
flatpak run --command=sh --devel com.termsurf.wezboard
```

Before then running `wezboard`.

Each log line will include the module name, which is a colon separated
namespace; in the output below the modules are `config`,
`wezboard_gui::frontend`, `wezboard_font::ftwrap` and `wezboard_gui::termwindow`:

```
10:29:24.451  DEBUG  config                    > Reloaded configuration! generation=2
10:29:24.452  DEBUG  wezboard_gui::frontend     > workspace is default, fixup windows
10:29:24.459  DEBUG  wezboard_font::ftwrap      > set_char_size computing 12 dpi=124 (pixel height=20.666666666666668)
10:29:24.461  DEBUG  wezboard_font::ftwrap      > set_char_size computing 12 dpi=124 (pixel height=20.666666666666668)
10:29:24.494  DEBUG  wezboard_gui::termwindow   > FocusChanged(true)
10:29:24.495  DEBUG  wezboard_gui::termwindow   > FocusChanged(false)
```

Those modules generally match up to directories and file names within the
wezboard source code, or to external modules that wezboard depends upon.

You can set a more restrictive filter to focus in on just the things you want.
For example, if you wanted to debug only configuration related things you might
set:

```
WEZBOARD_LOG=config=debug,info
```

which says:

* log `config` at `debug` level
* everything else at `info` level

You can add more comma-separated items:

```
WEZBOARD_LOG=config=debug,wezboard_font=debug,info
```

See Rust's [env_logger
documentation](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
for more details on the syntax/possibilities.

## Debugging Keyboard Related issues

Turn on [debug_key_events](config/lua/config/debug_key_events.md) to log
information about key presses.

Use [wezboard show-keys](cli/show-keys.md) or `wezboard show-keys --lua` to show
the effective set of key and mouse assignments defined by your config.

Consider changing [use_ime](config/lua/config/use_ime.md) to see that is
influencing your keyboard usage.

Double check to see if you have some system level utility/software that might
be intercepting or changing the behavior of a keyboard shortcut that you're
trying to use.

## Debugging Font Display

Use `wezboard ls-fonts` to explain which fonts will be used for different styles
of text.

Use `wezboard ls-fonts --list-system` to get a list of fonts available on your
system, in a form that you can use in your config file.

Use `wezboard ls-fonts --text foo` to explain how wezboard will render the text
`foo`, and `wezboard ls-fonts --text foo --rasterize-ascii` to show an ascii art
rendition of that text.

