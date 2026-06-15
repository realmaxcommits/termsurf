#!/usr/bin/env python3
"""Live macOS notification/link/bell trace guard for Issue 805 CFG-223."""

from __future__ import annotations

import os
import shlex
import subprocess
import tempfile
import textwrap
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
APP = ROOT / "roastty/macos/build/Debug/Roastty.app"
CLICK = ROOT / "scripts/ghostty-app/inject.swift"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def quote_applescript(value: str | Path) -> str:
    text = str(value)
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def run_osascript(script: str, timeout: int = 30) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["osascript", "-e", script],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise AssertionError(
            "osascript failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}\n"
            f"script:\n{script}"
        )
    return result


def scoped_pids() -> set[int]:
    scoped = subprocess.run(
        ["pgrep", "-f", f"{APP}/Contents/MacOS/roastty"],
        text=True,
        capture_output=True,
    )
    return {int(pid_text) for pid_text in scoped.stdout.split()}


def launch_app(config: Path, trace: Path) -> int:
    before = scoped_pids()
    require(not before, f"debug Roastty app is already running: {sorted(before)}")
    result = subprocess.run(
        [
            "open",
            "-n",
            "--env",
            f"ROASTTY_CONFIG_PATH={config}",
            "--env",
            "ROASTTY_CLEAR_USER_DEFAULTS=1",
            "--env",
            "ROASTTY_USER_DEFAULTS_SUITE=com.termsurf.roastty.issue805.exp186",
            "--env",
            f"ROASTTY_UI_KEY_TRACE_PATH={trace}",
            "--env",
            "ROASTTY_UI_TEST_SUPPRESS_OPEN_URL=1",
            str(APP),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        raise AssertionError(
            "open failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        created = sorted(scoped_pids() - before)
        if created:
            return created[0]
        time.sleep(0.25)
    raise AssertionError("open did not start a scoped debug Roastty process")


def wait_for_app(pid: int, timeout: float = 20.0) -> None:
    app_literal = quote_applescript(APP)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if subprocess.run(["ps", "-p", str(pid)], stdout=subprocess.DEVNULL).returncode != 0:
            raise AssertionError("Roastty debug process exited before AppleScript was ready")
        try:
            result = run_osascript(f"tell application {app_literal} to count of windows", timeout=5)
        except (AssertionError, subprocess.TimeoutExpired):
            time.sleep(0.5)
            continue
        if result.stdout.strip().isdigit():
            return
        time.sleep(0.5)
    raise AssertionError("Roastty did not become AppleScript-addressable in time")


def terminate_process(pid: int) -> None:
    try:
        try:
            run_osascript(f"tell application {quote_applescript(APP)} to quit", timeout=5)
        except Exception:
            pass
        for _ in range(20):
            if pid not in scoped_pids():
                return
            time.sleep(0.25)
    finally:
        if pid in scoped_pids():
            try:
                os.kill(pid, 9)
            except ProcessLookupError:
                pass


def create_terminal_window(command: str) -> str:
    app_literal = quote_applescript(APP)
    command_literal = quote_applescript(command)
    script = textwrap.dedent(
        f"""
        tell application {app_literal}
          activate
          set cfg to new surface configuration from {{command:{command_literal}, wait after command:true}}
          new window with configuration cfg
          delay 1
          set w to front window
          set t0 to focused terminal of selected tab of w
          if (id of t0) is "" then error "terminal id was empty"
          return id of t0
        end tell
        """
    )
    return run_osascript(script, timeout=45).stdout.strip()


def start_terminal_window(command: str) -> subprocess.Popen[str]:
    app_literal = quote_applescript(APP)
    command_literal = quote_applescript(command)
    script = textwrap.dedent(
        f"""
        tell application {app_literal}
          activate
          set cfg to new surface configuration from {{command:{command_literal}, wait after command:true}}
          new window with configuration cfg
        end tell
        """
    )
    return subprocess.Popen(
        ["osascript", "-e", script],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def focus_bounds(pid: int) -> tuple[int, int, int, int]:
    script = textwrap.dedent(
        f"""
        tell application "System Events"
          set roasttyProc to first application process whose unix id is {pid}
          set frontmost of roasttyProc to true
          delay 0.25
          perform action "AXRaise" of window 1 of roasttyProc
          delay 0.25
          set focusedWindow to value of attribute "AXFocusedWindow" of roasttyProc
          set focusedPosition to value of attribute "AXPosition" of focusedWindow
          set focusedSize to value of attribute "AXSize" of focusedWindow
          return (item 1 of focusedPosition as integer) & linefeed & (item 2 of focusedPosition as integer) & linefeed & (item 1 of focusedSize as integer) & linefeed & (item 2 of focusedSize as integer)
        end tell
        """
    )
    result = run_osascript(script, timeout=15)
    parts = [int(line.strip().strip(",")) for line in result.stdout.splitlines() if line.strip()]
    require(len(parts) == 4, f"unexpected focus bounds: {result.stdout!r}")
    return parts[0], parts[1], parts[2], parts[3]


def system_events(script_body: str, pid: int, timeout: int = 30) -> subprocess.CompletedProcess[str]:
    script = textwrap.dedent(
        f"""
        tell application "System Events"
          set roasttyProc to first application process whose unix id is {pid}
          set frontmost of roasttyProc to true
          delay 0.25
          set frontPID to unix id of first application process whose frontmost is true
          if frontPID is not {pid} then error "frontmost PID mismatch: " & frontPID
          {script_body}
        end tell
        """
    )
    return run_osascript(script, timeout=timeout)


def paste_shell_command(pid: int, command: str) -> None:
    subprocess.run(["pbcopy"], input=command, text=True, check=True, timeout=5)
    system_events(
        """
        tell roasttyProc
          click menu bar item "Edit" of menu bar 1
          delay 0.25
          click menu item "Paste" of menu 1 of menu bar item "Edit" of menu bar 1
        end tell
        """,
        pid,
        timeout=10,
    )


def read_trace(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(errors="replace")


def wait_for_trace(path: Path, needles: list[str], timeout: float = 15.0) -> str:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        text = read_trace(path)
        if all(needle in text for needle in needles):
            return text
        time.sleep(0.25)
    text = read_trace(path)
    missing = [needle for needle in needles if needle not in text]
    raise AssertionError(f"trace did not contain {missing}; trace was:\n{text}")


def write_config(path: Path, sound_path: Path, bell_features: str) -> None:
    path.write_text(
        "\n".join(
            [
                "macos-applescript = true",
                "font-size = 16",
                "background = #102030",
                "foreground = #f0f0f0",
                "background-opacity = 1",
                "desktop-notifications = true",
                f"bell-features = {bell_features}",
                f"bell-audio-path = {sound_path}",
                "bell-audio-volume = 0.125",
                "right-click-action = context-menu",
                "link-previews = true",
                "",
            ]
        )
    )


def write_trigger(path: Path, ready: Path, include_notification: bool, include_bell: bool) -> None:
    notification_line = (
        'sys.stdout.write("\\x1b]777;notify;Issue805Exp186;Notification Body\\x1b\\\\")'
        if include_notification
        else ""
    )
    bell_line = 'sys.stdout.write("\\a")' if include_bell else ""
    path.write_text(
        textwrap.dedent(
            f"""
            from pathlib import Path
            import sys
            import time

            Path({str(ready)!r}).write_text("ready")
            sys.stdout.write("https://example.com/issue805-exp186\\n")
            {notification_line}
            {bell_line}
            sys.stdout.flush()
            time.sleep(30)
            """
        ).lstrip()
    )


def run_trace_case(
    name: str,
    temp: Path,
    sound: Path,
    bell_features: str,
    include_notification: bool,
    include_bell: bool,
    trace_needles: list[str],
    require_notification_denied: bool = False,
) -> str:
    config = temp / f"{name}.roastty"
    trace = temp / f"{name}.trace.log"
    trigger = temp / f"{name}.trigger.py"
    ready = temp / f"{name}.ready.txt"
    write_config(config, sound, bell_features)
    write_trigger(trigger, ready, include_notification, include_bell)

    pid = launch_app(config, trace)
    window_proc: subprocess.Popen[str] | None = None
    try:
        wait_for_app(pid)
        command = f"python3 {shlex.quote(str(trigger))}"
        window_proc = start_terminal_window(command)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and not ready.exists():
            time.sleep(0.25)
        require(ready.exists(), f"{name}: trigger command did not start")

        wait_for_trace(trace, trace_needles, timeout=20)

        trace_text = read_trace(trace)
        if require_notification_denied:
            require(
                "desktopNotification authorizationStatus=1" in trace_text,
                f"{name}: VM should record denied notification authorization for split evidence",
            )
        return trace_text
    finally:
        if window_proc is not None and window_proc.poll() is None:
            window_proc.terminate()
            try:
                window_proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                window_proc.kill()
        terminate_process(pid)


def run_context_menu_case(temp: Path, sound: Path) -> str:
    config = temp / "context-menu.roastty"
    trace = temp / "context-menu.trace.log"
    write_config(config, sound, "false")

    pid = launch_app(config, trace)
    try:
        wait_for_app(pid)
        terminal_id = create_terminal_window("/bin/sh -c 'sleep 60'")
        require(terminal_id, "context-menu: terminal id sanity check failed")
        x, y, width, height = focus_bounds(pid)
        click_x = x + max(40, width // 2)
        click_y = y + max(60, height // 2)
        subprocess.run(
            ["swift", str(CLICK), "click", str(click_x), str(click_y), "right"],
            cwd=ROOT,
            check=True,
            timeout=10,
        )
        wait_for_trace(trace, ["contextMenu items=Paste", "Split Right", "Change Terminal Title"], timeout=10)
        return read_trace(trace)
    finally:
        terminate_process(pid)


def main() -> int:
    require(APP.is_dir(), f"app not built: {APP}")
    require(CLICK.is_file(), f"mouse injector missing: {CLICK}")

    with tempfile.TemporaryDirectory(prefix="termsurf-issue805-exp186-") as temp_dir:
        temp = Path(temp_dir)
        sound = Path("/System/Library/Sounds/Ping.aiff")
        require(sound.is_file(), f"expected built-in sound file: {sound}")
        run_trace_case(
            "notification-bell",
            temp,
            sound,
            "system,no-audio,attention,title,border",
            include_notification=True,
            include_bell=True,
            trace_needles=[
                "desktopNotification request title=Issue805Exp186 body=Notification Body",
                "desktopNotification authorizationStatus=",
                "ringBell target=surface",
                "surfaceBell state=true",
                "appBell system=true audio=false attention=true",
                "appBell attention=true",
            ],
            require_notification_denied=True,
        )
        run_trace_case(
            "audio-bell",
            temp,
            sound,
            "no-system,audio,no-attention,no-title,no-border",
            include_notification=False,
            include_bell=True,
            trace_needles=[
                "ringBell target=surface",
                "appBell system=false audio=true attention=false",
                f"appBell audio path={sound}",
            ],
        )

    print("macos_notification_link_bell_trace_runtime=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
