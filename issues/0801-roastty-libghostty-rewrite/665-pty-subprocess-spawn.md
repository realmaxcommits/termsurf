+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 665: PTY Subprocess Spawn

## Description

Experiment 664 added low-level PTY ownership and sizing. The next termio step is
starting a subprocess attached to that PTY, still without implementing the
persistent read loop or mailbox.

This experiment adds a small POSIX/macOS PTY subprocess primitive:

- open a PTY with an initial size;
- duplicate the slave descriptor for child stdin/stdout/stderr;
- run child pre-exec setup with `setsid()` and `ioctl(TIOCSCTTY)` so the child
  has the PTY as its controlling terminal, matching upstream Ghostty's
  `Pty.childPreExec`;
- spawn a configured command through `std::process::Command`;
- close the parent's slave descriptor after successful spawn so future EOF/read
  behavior is not held open by the parent;
- expose the PTY master fd and child handle internally for future termio read,
  write, resize, and wait-loop experiments.

This experiment does not add a background read thread, nonblocking master setup,
mailbox integration, environment construction, shell command selection, process
watchers, foreground process queries, or public ABI.

## Changes

- `roastty/src/os/pty.rs`
  - Extend `Pty` to represent post-spawn slave ownership precisely:
    `slave: Option<OwnedFd>`, `slave_fd() -> Option<RawFd>`, and a
    `close_slave()` or `take_slave()` helper that consumes the parent-owned
    slave. This avoids stale raw-fd access after the parent closes its slave
    side.
  - Add a `PtyCommand` or equivalent small builder for program, args, optional
    cwd, and initial `PtySize`.
  - Add `PtyChild` owning the `Pty` master side and `std::process::Child`.
  - Duplicate the slave fd for stdin/stdout/stderr with `dup`, wrap each
    duplicate in `OwnedFd`, and pass them to `Command` as `Stdio`.
  - Use `CommandExt::pre_exec` only for async-signal-safe libc calls: `setsid()`
    and `ioctl(slave_fd, TIOCSCTTY, 0)`.
  - Keep the original parent-owned slave fd open until `Command::spawn` returns
    so the `pre_exec` closure can use that raw fd for `TIOCSCTTY`; close the
    parent slave side only after successful spawn.
  - The `pre_exec` closure captures only the raw slave fd and converts
    `setsid`/`ioctl` failures into `io::Error`s.
  - Ensure all duplicated descriptors are `OwnedFd` so failures before spawn
    clean up automatically.
  - Define `PtyChild` cleanup explicitly:
    - provide `wait(&mut self) -> io::Result<ExitStatus>` for callers/tests;
    - implement `Drop` as a best-effort cleanup path that calls `try_wait`, then
      `kill` and `wait` if the process is still running, so dropping a
      `PtyChild` in tests does not leave a child process behind.
- Tests in `roastty/src/os/pty.rs`
  - Spawn `/bin/sh -c 'printf hello'`, poll the master fd with a timeout, and
    verify output is readable from the PTY master. Treat readable or hangup
    readiness as acceptable before reading because the child may exit quickly.
    Wait for the child and assert successful exit.
  - Spawn `/bin/sh -c 'test -t 0 && test -t 1 && test -t 2 && printf tty'` and
    verify the child sees all three stdio fds as TTYs. Wait for the child and
    assert successful exit.
  - Verify the parent-side slave fd is closed after successful spawn.
  - Verify dropping a long-running `PtyChild` reaps/kills it via the best-effort
    `Drop` cleanup path.

## Verification

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/665-pty-subprocess-spawn.md`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `cargo test -p roastty os::pty`
- `git diff --check`

## Design Review

**Result:** Approved after amendments.

Codex first found four concrete design gaps: post-spawn slave ownership needed
to avoid stale raw-fd access, `PtyChild` cleanup and wait semantics needed to be
defined, the pre-exec fd lifetime and async-signal-safe operation set needed to
be explicit, and the tests needed poll/readiness plus child wait/status
assertions.

The design now specifies `slave: Option<OwnedFd>` with optional raw-fd access
after spawn, a `PtyChild::wait` method plus best-effort `Drop` cleanup, a
pre-exec closure that keeps the original slave fd open until `spawn` succeeds
and performs only `setsid`/`ioctl` error conversion, and tests that poll with a
timeout, handle readable/hangup readiness, wait for successful child exit, and
cover drop cleanup. Codex re-reviewed the amended design and approved it for
plan commit and implementation with no remaining blockers.
