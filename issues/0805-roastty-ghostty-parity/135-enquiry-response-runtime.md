# Experiment 135: Enquiry Response Runtime

## Description

`RUNTIME-009B2B2B3B2B` still groups the remaining terminal behavior effects
after OSC 7, title, shell integration, and scrollback slices were separated. One
narrow unproven behavior in that gap is `enquiry-response`: pinned Ghostty
stores the configured response string in `termio/stream_handler.zig` and writes
it back to the PTY when the child sends ENQ (`0x05`).

Roastty already parses and formats `enquiry-response`, and its terminal stream
recognizes ENQ. However, the current runtime path only answers ENQ through a C
callback. That callback path is not available to PTY-backed `TermioWorker`
terminals because workers reject terminals with installed callbacks. The normal
config-driven PTY runtime therefore needs an owned response string that can be
set from parsed config and written without callbacks.

This experiment will split the remaining terminal row:

- `RUNTIME-009B2B2B3B2B1`: **Oracle complete** for config-driven
  `enquiry-response` ENQ replies through terminal core and PTY-backed `Termio`
  runtime, including startup config and runtime config update wiring.
- `RUNTIME-009B2B2B3B2B2`: **Gap** for other remaining terminal behavior
  effects.

This experiment will not claim broader terminal parity beyond ENQ response
behavior.

## Changes

- `roastty/src/terminal/terminal.rs`
  - Add owned `enquiry_response` state to terminal initialization/options.
  - Update ENQ handling so configured bytes are written to the PTY response
    buffer without requiring a callback.
  - Preserve the existing callback API for embedded/direct users, with tests
    proving the config-driven path does not regress the callback path.
  - Add focused terminal tests for default empty response, configured response,
    runtime response update, and callback compatibility.
- `roastty/src/termio.rs`
  - Add `enquiry_response` to `TermioSpawnOptions`.
  - Pass it into `TerminalInitOptions`.
  - Add a PTY-backed runtime test proving a child process that sends ENQ can
    read the configured response.
- `roastty/src/lib.rs`
  - Thread parsed `Config.enquiry_response` into initial surface `Termio` spawn
    options.
  - Update existing live surfaces when app config changes so ENQ responses use
    the latest parsed config.
  - Add or extend focused app/surface config tests for startup and update
    propagation.
- `issues/0805-roastty-ghostty-parity/enquiry_response_runtime_parity.py`
  - Add a static guard checking pinned Ghostty markers: `@"enquiry-response"`,
    `enquiry_response`, `changeConfig`,
    `self.enquiry_response = config.enquiry_response`, `.enquiry`, and
    `writeReq`.
  - Check Roastty markers for parser coverage, terminal owned response state,
    Termio spawn wiring, app config startup/update wiring, focused runtime
    tests, and the inventory split.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split `RUNTIME-009B2B2B3B2B` into the ENQ complete row and the reduced
    remaining-terminal gap row.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Regenerate from the inventory script.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-223 summary. It must remain `Gap`.
- Existing CFG-223 static guards that hard-code current runtime row counts or
  the remaining terminal gap row
  - Update expected counts after the split: 44 runtime rows, 37 Oracle complete
    rows, 39 closed rows, and 5 remaining runtime gaps.
  - Update references from the old remaining terminal gap row to
    `RUNTIME-009B2B2B3B2B2`.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- Pinned Ghostty evidence shows `enquiry-response` is a configured string that
  `StreamHandler.enquiry` writes back to the PTY on ENQ.
- Pinned Ghostty evidence also shows runtime config updates assign
  `config.enquiry_response` into the active stream handler through
  `StreamHandler.changeConfig`.
- Roastty terminal core writes configured ENQ response bytes without requiring a
  callback.
- The existing embedded callback path is preserved.
- PTY-backed `Termio` runtime proves a child-visible ENQ response using parsed
  spawn options.
- Initial app/surface config and live config updates both propagate
  `enquiry-response` to the active terminal runtime.
- `RUNTIME-009B2B2B3B2B1` is Oracle complete and cites the terminal, Termio,
  app/surface, and static guard evidence.
- `RUNTIME-009B2B2B3B2B2` remains `Gap` for other remaining terminal behavior
  effects.
- `CFG-223` remains `Gap`.

Commands:

```bash
cargo test --manifest-path roastty/Cargo.toml terminal_stream_enquiry_response
cargo test --manifest-path roastty/Cargo.toml termio_enquiry_response
cargo test --manifest-path roastty/Cargo.toml surface_enquiry_response
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/enquiry_response_runtime_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
cargo fmt --manifest-path roastty/Cargo.toml
cargo fmt --manifest-path roastty/Cargo.toml --check
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/135-enquiry-response-runtime.md
git diff --check
```

Fail criteria:

- ENQ response is only proven through parser/default tests.
- PTY-backed terminals still require an installed terminal callback to answer
  ENQ.
- Runtime config update changes stored config but not the active terminal
  response.
- The experiment promotes unrelated terminal behavior from the remaining gap.
- CFG-223 is marked complete.

## Design Review

**Reviewer:** Codex adversarial subagent with fresh context.

**Initial verdict:** Changes required.

The reviewer found one required issue: the design claimed runtime config update
wiring for `enquiry-response`, but the pinned Ghostty verification and static
guard only required the ENQ write path. Pinned Ghostty also updates the active
stream handler in `StreamHandler.changeConfig` with
`self.enquiry_response = config.enquiry_response`.

**Fix:** The design now requires the static guard to check Ghostty
`changeConfig` and `self.enquiry_response = config.enquiry_response`, and the
pass criteria explicitly require pinned Ghostty runtime config update evidence
through `StreamHandler.changeConfig`.

**Final verdict:** Approved. The reviewer confirmed the prior required finding
was resolved and found no new required issues.
