# Issue 674: Configurable Homepage

Add a configurable homepage URL so `web` (without arguments) opens a default
page instead of exiting with a usage error.

## Background

Currently `web` requires a URL argument:

```bash
web google.com        # works
web                   # exits with "Usage: web [url] <url> ..."
```

Users should be able to type `web` and get a homepage. The default will be
`https://termsurf.com/welcome`, configurable via the standard TermSurf config.

## Architecture

Follow the proven `TERMSURF_PANE_ID` pattern: the GUI reads the config and
propagates the homepage as an environment variable to all child processes. The
TUI reads the env var as a fallback when no URL is given on the CLI.

Data flow:

```
~/.config/termsurf/config  →  Config.zig (homepage field)
                               ↓
                           Surface.zig (env.put TERMSURF_HOMEPAGE)
                               ↓
                           shell spawns `web`
                               ↓
                           main.rs (env::var fallback)
```

The TUI remains stateless — it doesn't read config files directly.

## Experiment 1: Homepage via environment variable

### Hypothesis

Adding a `homepage` config field and propagating it as `TERMSURF_HOMEPAGE` will
let `web` (without arguments) open the configured homepage.

### Changes

#### 1. Config.zig — new field

After the browser-related config fields:

```zig
/// The default homepage URL opened when `web` is run without arguments.
@"homepage": [:0]const u8 = "https://termsurf.com/welcome",
```

#### 2. Surface.zig — propagate as environment variable

After the `TERMSURF_PANE_ID` line (line 670):

```zig
// Propagate homepage so `web` without arguments opens the configured page.
try env.put("TERMSURF_HOMEPAGE", self.config.homepage);
```

#### 3. tui/src/main.rs — use as fallback

Change the URL resolution (line 154-160) to fall back to `TERMSURF_HOMEPAGE`:

```rust
let mut url = match cli.command {
    Some(Commands::Url { url }) => url,
    None => cli.url.unwrap_or_else(|| {
        std::env::var("TERMSURF_HOMEPAGE")
            .unwrap_or_else(|_| "https://termsurf.com/welcome".to_string())
    }),
};
```

This gives three levels of precedence:

1. `web url <url>` — explicit subcommand wins
2. `web <url>` — positional argument
3. `web` — falls back to `TERMSURF_HOMEPAGE` env var, then hardcoded default

### Test

1. `cd gui && zig build` — compiles without errors.
2. `cd tui && cargo build` — compiles without errors.
3. Type `web` with no arguments — opens `https://termsurf.com/welcome`.
4. Type `web google.com` — opens Google (env var ignored).
5. Set `homepage = https://example.com` in config, restart terminal.
6. Type `web` — opens `https://example.com`.
7. Outside TermSurf (no env var), `web` still opens the hardcoded default.
