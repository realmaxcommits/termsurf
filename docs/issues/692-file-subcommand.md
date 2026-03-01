# Issue 692: `web file` Subcommand

Add a `web file <path>` subcommand that opens local files in the browser pane.
`web file index.html` resolves the path to an absolute `file:///` URL and
navigates to it. No more typing `file:///Users/ryan/dev/project/index.html` by
hand.

## Background

The browser already supports `file:///` URLs — you can type
`web file:///absolute/path/to/file.html` and it works. But this is painful:

- You have to know the absolute path
- You have to type the `file:///` prefix
- Relative paths don't work
- Tab completion doesn't help

`web file index.html` should just work. The TUI resolves the relative path
against `$PWD`, canonicalizes it, and converts it to a `file:///` URL.

## Design

### New subcommand: `web file <path>`

Add a `File` variant to the `Commands` enum in `main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    Url { url: String },
    Last,
    Status,
    /// Open a local file in the browser pane
    File {
        /// Path to the file (relative or absolute)
        path: String,
    },
}
```

### Path resolution

In the URL resolution block (line 269), handle the new subcommand:

```rust
Some(Commands::File { path }) => {
    let absolute = std::fs::canonicalize(&path).unwrap_or_else(|e| {
        eprintln!("Error: {}: {}", path, e);
        std::process::exit(1);
    });
    format!("file://{}", absolute.display())
}
```

`std::fs::canonicalize` resolves `.`, `..`, symlinks, and relative paths against
the current working directory. If the file doesn't exist, it returns an error —
which is the right behavior (no point navigating to a nonexistent file).

### Also support `file` in the Edit mode URL bar

When the user types a path into the URL bar (Edit mode) and presses Enter,
`normalize_url` currently prepends `https://` if it sees a dot. A bare filename
like `index.html` would become `https://index.html`.

Add file path detection to `normalize_url`:

```rust
fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.contains("://") {
        return trimmed.to_string();
    }
    // Check if it looks like a file path (starts with / or ./ or ../).
    if trimmed.starts_with('/') || trimmed.starts_with("./") || trimmed.starts_with("../") {
        if let Ok(absolute) = std::fs::canonicalize(trimmed) {
            return format!("file://{}", absolute.display());
        }
    }
    // ... existing localhost/https logic ...
}
```

This only triggers for paths that are unambiguously file paths (absolute or
explicitly relative). A bare `index.html` in the URL bar still goes to
`https://index.html` — use `./index.html` to be explicit.

## Test

1. `web file index.html` — opens `file:///absolute/path/to/index.html`
2. `web file ./src/page.html` — relative path with `./`
3. `web file ../other/file.html` — relative path with `../`
4. `web file /tmp/test.html` — absolute path
5. `web file nonexistent.html` — prints error and exits
6. In Edit mode, type `./index.html` and press Enter — navigates to file URL
7. In Edit mode, type `/tmp/test.html` and press Enter — navigates to file URL
8. In Edit mode, type `index.html` and press Enter — still goes to
   `https://index.html` (no ambiguity)
9. `web file:///tmp/test.html` — existing behavior still works (not broken)
