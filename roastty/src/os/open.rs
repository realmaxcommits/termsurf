//! Open a URL/file in the default handler (port of upstream `os/open`).

use std::process::{Command, Stdio};

/// The kind of URL being opened, which selects the opener arguments (upstream
/// `apprt.action.OpenUrl.Kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Text,
    Html,
    Unknown,
}

/// The macOS `open` argv for a URL of the given `kind` (upstream's per-`kind` `Child.init`):
/// `open -t <url>` for text, `open <url>` otherwise.
fn open_command_args(kind: Kind, url: &str) -> Vec<&str> {
    match kind {
        Kind::Text => vec!["open", "-t", url],
        Kind::Html | Kind::Unknown => vec!["open", url],
    }
}

/// Open `url` in the default handling application (upstream `os.open.open`). stdout is
/// ignored; stderr is drained and the child reaped on a detached thread so this never
/// blocks. Returns an error if the opener fails to spawn or the reaper thread fails to start.
pub(crate) fn open(kind: Kind, url: &str) -> std::io::Result<()> {
    let args = open_command_args(kind, url);
    let mut command = Command::new(args[0]);
    command.args(&args[1..]);
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());

    // Spawn on this thread so a spawn failure is detected synchronously.
    let mut child = command.spawn()?;

    // Drain stderr and reap on a detached thread (some `open` implementations block, some
    // don't), matching upstream's `openThread`. A thread-creation failure propagates
    // (upstream uses `try std.Thread.spawn`); the returned `JoinHandle` is dropped to detach.
    std::thread::Builder::new().spawn(move || {
        if let Some(mut stderr) = child.stderr.take() {
            // Upstream logs each stderr line; roastty has no logger here, so we drain it to a
            // sink (a bounded internal buffer) so the pipe can't fill and stall the child.
            let _ = std::io::copy(&mut stderr, &mut std::io::sink());
        }
        let _ = child.wait();
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_uses_dash_t() {
        assert_eq!(
            open_command_args(Kind::Text, "https://example.com"),
            ["open", "-t", "https://example.com"],
        );
    }

    #[test]
    fn html_and_unknown_use_plain_open() {
        assert_eq!(
            open_command_args(Kind::Html, "https://example.com"),
            ["open", "https://example.com"],
        );
        assert_eq!(
            open_command_args(Kind::Unknown, "https://example.com"),
            ["open", "https://example.com"],
        );
    }
}
