# Command Line

This section documents the wezboard command line.

*Note that `wezboard --help` or `wezboard SUBCOMMAND --help` will show the precise
set of options that are applicable to your installed version of wezboard.*

wezboard is deployed with two major executables:

* `wezboard` (or `wezboard.exe` on Windows) - for interacting with wezboard from the terminal
* `wezboard-gui` (or `wezboard-gui.exe` on Windows) - for spawning wezboard from a desktop environment

You will typically use `wezboard` when scripting wezboard; it knows when to
delegate to `wezboard-gui` under the covers.

If you are setting up a launcher for wezboard to run in the Windows GUI
environment then you will want to explicitly target `wezboard-gui` so that
Windows itself doesn't pop up a console host for its logging output.

!!! note
    `wezboard-gui.exe --help` will not output anything to a console when
    run on Windows systems, because it runs in the Windows GUI subsystem and has no
    connection to the console.  You can use `wezboard.exe --help` to see information
    about the various commands; it will delegate to `wezboard-gui.exe` when
    appropriate.

## Synopsis

```console
{% include "../examples/cmd-synopsis-wezboard--help.txt" %}
```
