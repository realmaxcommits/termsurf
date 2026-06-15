#!/usr/bin/env python3
"""Live macOS native menu guard for Issue 805 CFG-223."""

from __future__ import annotations

import os
import subprocess
import tempfile
import textwrap
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ISSUE = ROOT / "issues/0805-roastty-ghostty-parity"
APP = ROOT / "roastty/macos/build/Debug/Roastty.app"
BINARY = APP / "Contents/MacOS/roastty"
DIAGNOSTIC_REPORTS = Path.home() / "Library/Logs/DiagnosticReports"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


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


def quote_applescript(value: str | Path) -> str:
    text = str(value)
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def scoped_pids() -> set[int]:
    scoped = subprocess.run(
        ["pgrep", "-f", f"{APP}/Contents/MacOS/roastty"],
        text=True,
        capture_output=True,
    )
    return {int(pid_text) for pid_text in scoped.stdout.split()}


def crash_reports() -> set[Path]:
    if not DIAGNOSTIC_REPORTS.is_dir():
        return set()
    return set(DIAGNOSTIC_REPORTS.glob("roastty-*.ips"))


def wait_for_crash_report_settle(before: set[Path]) -> set[Path]:
    deadline = time.monotonic() + 5
    observed: set[Path] = set()
    while time.monotonic() < deadline:
        time.sleep(0.5)
        observed.update(crash_reports() - before)
    return observed


def launch_app(config: Path) -> int:
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
            "ROASTTY_USER_DEFAULTS_SUITE=com.termsurf.roastty.issue805.exp172",
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
        after = scoped_pids()
        created = sorted(after - before)
        if created:
            return created[0]
        time.sleep(0.25)
    raise AssertionError("open did not start a scoped debug Roastty process")


def wait_for_app(pid: int, timeout: float = 20.0) -> None:
    deadline = time.monotonic() + timeout
    app_literal = quote_applescript(APP)
    while time.monotonic() < deadline:
        if subprocess.run(["ps", "-p", str(pid)], stdout=subprocess.DEVNULL).returncode != 0:
            raise AssertionError("Roastty debug process exited before AppleScript was ready")
        try:
            result = run_osascript(
                f'tell application {app_literal} to count of windows',
                timeout=5,
            )
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
            run_osascript(f'tell application {quote_applescript(APP)} to quit', timeout=5)
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


def ensure_terminal_window(command: str) -> None:
    app_literal = quote_applescript(APP)
    command_literal = quote_applescript(command)
    script = textwrap.dedent(
        f"""
        tell application {app_literal}
          activate
          if (count of windows) > 0 then
            set w to front window
            if (id of focused terminal of selected tab of w) is not "" then return
          end if
          set cfg to new surface configuration from {{command:{command_literal}, wait after command:true}}
          new window with configuration cfg
          delay 1
          if (count of windows) < 1 then error "new window was not created"
          set w to front window
          if (id of focused terminal of selected tab of w) is "" then error "focused terminal id was empty"
        end tell
        """
    )
    run_osascript(script, timeout=30)


def app_count(expression: str) -> int:
    app_literal = quote_applescript(APP)
    result = run_osascript(f"tell application {app_literal} to {expression}", timeout=10)
    text = result.stdout.strip()
    require(text.isdigit(), f"expected numeric AppleScript result for {expression}: {text!r}")
    return int(text)


def wait_for_count(expression: str, expected: int, description: str) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if app_count(expression) == expected:
            return
        time.sleep(0.25)
    raise AssertionError(f"{description} did not become {expected}")


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


def list_menu_bar_items(pid: int) -> list[str]:
    result = system_events(
        """
        tell roasttyProc
          set AppleScript's text item delimiters to linefeed
          return (name of menu bar items of menu bar 1) as text
        end tell
        """,
        pid,
    )
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def assert_menu_items(pid: int, menu_name: str, expected: set[str]) -> None:
    result = system_events(
        f"""
        tell roasttyProc
          click menu bar item {quote_applescript(menu_name)} of menu bar 1
          delay 0.25
          set itemNames to {{}}
          repeat with menuItem in menu items of menu 1 of menu bar item {quote_applescript(menu_name)} of menu bar 1
            set itemName to name of menuItem
            if itemName is not missing value and itemName is not "" then set end of itemNames to itemName
          end repeat
          set AppleScript's text item delimiters to linefeed
          return itemNames as text
        end tell
        """,
        pid,
    )
    observed = {line.strip() for line in result.stdout.splitlines() if line.strip()}
    missing = sorted(expected - observed)
    require(not missing, f"{menu_name} menu missing items {missing}; observed={sorted(observed)}")


def menu_item_enabled(pid: int, menu_name: str, item_name: str) -> bool:
    result = system_events(
        f"""
        tell roasttyProc
          click menu bar item {quote_applescript(menu_name)} of menu bar 1
          delay 0.25
          return enabled of menu item {quote_applescript(item_name)} of menu 1 of menu bar item {quote_applescript(menu_name)} of menu bar 1
        end tell
        """,
        pid,
    )
    text = result.stdout.strip().lower()
    require(text in {"true", "false"}, f"unexpected enabled state for {menu_name}/{item_name}: {text!r}")
    return text == "true"


def click_menu_item(pid: int, menu_name: str, item_name: str) -> None:
    system_events(
        f"""
        tell roasttyProc
          click menu bar item {quote_applescript(menu_name)} of menu bar 1
          delay 0.25
          click menu item {quote_applescript(item_name)} of menu 1 of menu bar item {quote_applescript(menu_name)} of menu bar 1
        end tell
        """,
        pid,
    )


def dismiss_menus(pid: int) -> None:
    system_events("key code 53", pid, timeout=10)


def main() -> int:
    require(APP.is_dir(), f"app not built: {APP}")
    require(BINARY.is_file(), f"app binary not built: {BINARY}")

    crash_reports_before = crash_reports()

    with tempfile.TemporaryDirectory(prefix="termsurf-issue805-exp172-") as temp_dir:
        temp = Path(temp_dir)
        config = temp / "config.roastty"
        config.write_text("macos-applescript = true\nquit-after-last-window-closed = true\n")
        command = "/bin/sh -c 'sleep 60'"
        pid = launch_app(config)

        try:
            wait_for_app(pid)
            ensure_terminal_window(command)

            menu_bar_items = set(list_menu_bar_items(pid))
            expected_top_level = {"Roastty", "File", "Edit", "View", "Window", "Help"}
            require(
                expected_top_level <= menu_bar_items,
                f"missing top-level menus {sorted(expected_top_level - menu_bar_items)}; observed={sorted(menu_bar_items)}",
            )

            assert_menu_items(
                pid,
                "File",
                {
                    "New Window",
                    "New Tab",
                    "Split Right",
                    "Split Left",
                    "Split Down",
                    "Split Up",
                    "Close",
                },
            )
            assert_menu_items(
                pid,
                "Edit",
                {"Undo", "Redo", "Copy", "Paste", "Select All"},
            )
            assert_menu_items(
                pid,
                "View",
                {"Command Palette", "Quick Terminal", "Terminal Inspector"},
            )
            assert_menu_items(
                pid,
                "Window",
                {"Toggle Full Screen", "Float on Top", "Use as Default", "Bring All to Front"},
            )

            require(not menu_item_enabled(pid, "Edit", "Undo"), "Undo should be disabled with no undo stack")
            require(not menu_item_enabled(pid, "Edit", "Redo"), "Redo should be disabled with no redo stack")
            for menu_name, item_name in [
                ("File", "New Window"),
                ("File", "New Tab"),
                ("File", "Split Right"),
                ("View", "Quick Terminal"),
                ("View", "Command Palette"),
                ("Window", "Toggle Full Screen"),
                ("Window", "Float on Top"),
                ("Window", "Use as Default"),
            ]:
                require(
                    menu_item_enabled(pid, menu_name, item_name),
                    f"{menu_name}/{item_name} should be enabled with a primary terminal window",
                )

            dismiss_menus(pid)
            before_tabs = app_count("count of tabs of front window")
            click_menu_item(pid, "File", "New Tab")
            wait_for_count("count of tabs of front window", before_tabs + 1, "New Tab menu action")

            dismiss_menus(pid)
            before_terminals = app_count("count of terminals of selected tab of front window")
            click_menu_item(pid, "File", "Split Right")
            wait_for_count(
                "count of terminals of selected tab of front window",
                before_terminals + 1,
                "Split Right menu action",
            )
        finally:
            terminate_process(pid)

    new_crash_reports = wait_for_crash_report_settle(crash_reports_before)
    require(
        not new_crash_reports,
        "Roastty wrote crash reports during native menu workflow: "
        + ", ".join(str(path) for path in sorted(new_crash_reports)),
    )

    print("macos_native_menu_runtime=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
