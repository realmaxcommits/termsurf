+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 806: Crash Envelope Persistence

## Description

Port the local crash-envelope persistence decision from upstream
`crash/sentry.zig::Transport.sendInternal` into Roastty.

Experiments 804 and 805 added the crash-report directory/listing foundation and
the Sentry envelope parse/serialize foundation. The next useful slice is the
write path for an already serialized envelope: parse it, discard envelopes that
do not contain an event item, create the crash-report directory, and write the
serialized bytes as a local crash report.

This experiment still avoids native Sentry SDK initialization, crash callbacks,
report upload, CLI commands, and frontend flows. It is a persistence foundation
that future SDK transport glue can call.

## Changes

- `roastty/Cargo.toml`
  - Add `uuid` to Roastty only if the default production helper generates report
    filenames in this experiment. Prefer reusing the workspace-locked crate over
    adding a new random/UUID dependency.
- `roastty/src/crash.rs`
  - Add an envelope event check equivalent to upstream `shouldDiscard`: an
    envelope is persistable only if at least one item has `ItemType::Event`.
  - Add a public-in-crate persistence helper that accepts serialized envelope
    bytes, parses them with the existing `Envelope::parse`, discards non-event
    envelopes, creates the crash directory, writes the original serialized bytes
    for event envelopes, and returns whether a report was written.
  - Keep deterministic testing possible by separating the filename-injected
    write path from any production filename generation. Tests should not depend
    on randomness.
  - Use the `.roasttycrash` report extension for newly written reports, while
    keeping existing directory listing behavior extension-agnostic.
  - Add tests for:
    - event envelopes are written to a created crash directory;
    - persisted file bytes exactly match the serialized input bytes;
    - session-only or attachment-only envelopes are discarded before directory
      creation, so they do not create the crash directory and do not create a
      report file;
    - malformed envelopes return the parse error before directory creation, so
      they do not create the crash directory and do not write;
    - filename injection rejects path separators or otherwise cannot escape the
      crash directory;
    - generated production report filenames use the `.roasttycrash` extension.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - After implementation, update the crash/Sentry partial rows to mention local
    event-envelope persistence while keeping SDK initialization, crash
    callbacks, upload, CLI commands, and frontend flows open.

## Verification

- Inspect:
  - `vendor/ghostty/src/crash/sentry.zig`
  - `vendor/ghostty/src/crash/sentry_envelope.zig`
  - `vendor/ghostty/src/crash/dir.zig`
- Run:
  - `cargo fmt -p roastty`
  - `cargo test -p roastty crash -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/806-crash-envelope-persistence.md`
- Run:
  - `git diff --check`

The experiment passes if Roastty can persist event envelopes locally with tested
discard/error behavior while crash reporting remains partial. It is Partial if
event detection lands but the file write path needs follow-up. It fails if local
persistence cannot be separated cleanly from native Sentry SDK capture.

## Design Review

Codex reviewed the design and found two blocking verification gaps. First, the
discard/error tests did not explicitly require upstream's ordering: parse and
discard non-event envelopes before creating the crash-report directory. The plan
now requires session-only, attachment-only, and malformed-envelope tests to
assert that those paths do not create the crash directory. Second, the report
extension decision was too vague for Roastty's naming constraint. The plan now
requires generated production filenames to use the exact `.roasttycrash`
extension and to test that extension.

Codex re-reviewed the corrected design and approved it with no findings. The
approval confirmed that the no-directory side-effect tests match upstream's
discard-before-`makePath` ordering, the generated report extension is pinned to
`.roasttycrash`, and the scope remains limited to persistence without SDK
initialization, crash callbacks, upload, CLI commands, or frontend work.

## Result

**Result:** Pass

Roastty now has a local event-envelope persistence foundation in
`roastty/src/crash.rs`. `CrashDir::persist_event_envelope` parses the serialized
envelope bytes, discards envelopes without an `event` item, and then generates a
`.roasttycrash` report name only for event envelopes. A deterministic
`persist_event_envelope_with_name` helper shares the same parse/discard path for
tests and future call sites with an existing report name. Event envelopes create
the crash directory, write the original serialized bytes, and return that a
report was written.

`roastty/Cargo.toml` now depends on
`uuid = { version = "1.13", features = ["v4"] }` for generated report names.
`Cargo.lock` changed only to add `uuid` to Roastty's dependency list; the
package was already present in the workspace lock.

The Issue 801 checklist now records crash reporting as partial with local
directory/listing support, envelope parse/serialize/attachment decode support,
and local event-envelope persistence present. Sentry SDK initialization, crash
callbacks, report upload, CLI commands, and frontend flows remain open.

Verification:

- Inspected `vendor/ghostty/src/crash/sentry.zig`.
- Inspected `vendor/ghostty/src/crash/sentry_envelope.zig`.
- Inspected `vendor/ghostty/src/crash/dir.zig`.
- `cargo fmt -p roastty` — passed.
- `cargo test -p roastty crash -- --nocapture --test-threads=1` — passed, 19
  tests.
- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/806-crash-envelope-persistence.md`
  — passed.
- `git diff --check` — passed.

## Conclusion

Experiment 806 completes the local file persistence layer that future Sentry SDK
transport glue can call. The next crash-reporting work can focus on SDK
initialization and callback capture, then upload and CLI/frontend exposure, with
the local storage path already tested independently.

## Completion Review

Codex reviewed the staged result and found one blocking ordering mismatch:
`persist_event_envelope` generated the UUID report name before parsing and
discard checks, while upstream parses, discards non-event envelopes, and only
then generates the report filename. The implementation now parses and checks for
an event item before generating the `.roasttycrash` filename.

Codex re-reviewed the corrected staged result and approved it with no findings.
The approval confirmed that production and injected-name helpers now preserve
the upstream parse/discard-before-filename/write ordering, generated reports use
`.roasttycrash`, persisted files keep the original input bytes, dependency churn
is limited to adding the already-locked `uuid` crate to Roastty, and the docs do
not overclaim SDK initialization, callbacks, upload, CLI, or frontend work.
