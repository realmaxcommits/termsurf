#!/usr/bin/env python3
"""Live macOS user-notification delivery guard for Issue 805 Experiment 195."""

from __future__ import annotations

import json
import re
import subprocess
import tempfile
import textwrap
import time
from pathlib import Path

from macos_window_padding_pixel_runtime import (
    APP,
    ROOT,
    crash_reports,
    quote_applescript,
    require,
    run_osascript,
    scoped_pids,
    terminate_process,
    wait_for_app,
    wait_for_crash_report_settle,
)


LOGS = ROOT / "logs"
LATEST_JSON = LOGS / "issue805-exp195-user-notification-latest.json"
TITLE = "Issue805Exp195Notification"
BODY = "Issue 805 Experiment 195 Body"


def read(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(errors="replace")


def write_config(path: Path) -> None:
    path.write_text(
        "\n".join(
            [
                "macos-applescript = true",
                "quit-after-last-window-closed = true",
                "desktop-notifications = true",
                "font-size = 16",
                "window-width = 100",
                "window-height = 34",
                "background = #102030",
                "foreground = #ffffff",
                "background-opacity = 1",
                "macos-titlebar-style = hidden",
                "",
            ]
        )
    )


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
            "ROASTTY_USER_DEFAULTS_SUITE=com.termsurf.roastty.issue805.exp195.usernotification",
            "--env",
            f"ROASTTY_UI_KEY_TRACE_PATH={trace}",
            "--env",
            "ROASTTY_UI_TEST_ENABLE_USER_NOTIFICATION_ACTION=1",
            str(APP),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    require(result.returncode == 0, f"open failed\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")

    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        created = sorted(scoped_pids() - before)
        if created:
            return created[0]
        time.sleep(0.25)
    raise AssertionError("open did not start a scoped debug Roastty process")


def invoke_user_notification() -> None:
    app_literal = quote_applescript(APP)
    script = textwrap.dedent(
        f"""
        tell application {app_literal}
          activate
          set cfg to new surface configuration from {{command:"/bin/sh -c 'sleep 60'", wait after command:true}}
          new window with configuration cfg
          delay 1
          set t0 to focused terminal of selected tab of front window
          perform action "ui_test_user_notification" on t0
        end tell
        """
    )
    run_osascript(script, timeout=30)


def wait_for_notification_result(trace: Path, timeout: float = 12.0) -> tuple[str, dict[str, object]]:
    deadline = time.monotonic() + timeout
    trace_text = ""
    while time.monotonic() < deadline:
        trace_text = read(trace)
        settings = re.findall(
            r"userNotification settings status=(\d+) alert=(\d+) sound=(\d+)",
            trace_text,
        )
        if settings:
            status, alert, sound = [int(value) for value in settings[-1]]
            evidence: dict[str, object] = {
                "authorization_status": status,
                "alert_setting": alert,
                "sound_setting": sound,
            }
            if status == 2:
                delivered = re.findall(
                    r"userNotification delivered count=(\d+) (.*)",
                    trace_text,
                )
                if delivered:
                    count = int(delivered[-1][0])
                    summary = delivered[-1][1]
                    require(count == 1, f"expected one delivered notification, saw {count}: {summary}")
                    for needle in [
                        "id=issue805-exp195-",
                        f"title={TITLE}",
                        f"body={BODY}",
                        "category=com.mitchellh.roastty.userNotification",
                        "surface=",
                        "requireFocus=false",
                    ]:
                        require(needle in summary, f"missing delivered notification evidence {needle!r}: {summary}")
                    require("userNotification request id=issue805-exp195-" in trace_text, "missing request trace")
                    require(f"title={TITLE}" in trace_text, "missing request title trace")
                    require(f"body={BODY}" in trace_text, "missing request body trace")
                    require("requireFocus=false" in trace_text, "missing request requireFocus trace")
                    require("userNotification added id=issue805-exp195-" in trace_text, "missing added trace")
                    require("tracked=true" in trace_text, "missing tracked=true trace")
                    evidence["result"] = "delivered"
                    evidence["delivered_summary"] = summary
                    return trace_text, evidence
            else:
                blocked = f"userNotification uiTestAction=blocked status={status}"
                if blocked in trace_text:
                    evidence["result"] = "authorization-blocked"
                    return trace_text, evidence
        time.sleep(0.25)
    raise AssertionError(f"notification result trace missing; trace was:\n{trace_text}")


def main() -> int:
    require(APP.is_dir(), f"app not built: {APP}")
    LOGS.mkdir(parents=True, exist_ok=True)
    before_crashes = crash_reports()

    with tempfile.TemporaryDirectory(prefix="termsurf-issue805-exp195-user-notification-") as temp_dir:
        temp = Path(temp_dir)
        config = temp / "config.roastty"
        trace = temp / "trace.log"
        write_config(config)
        pid = launch_app(config, trace)
        try:
            wait_for_app(pid)
            invoke_user_notification()
            trace_text, evidence = wait_for_notification_result(trace)
            evidence["trace_tail"] = trace_text.splitlines()[-40:]
        finally:
            terminate_process(pid)

    new_crashes = sorted(str(path) for path in wait_for_crash_report_settle(before_crashes))
    evidence["new_crash_reports"] = new_crashes
    require(not new_crashes, f"new Roastty crash reports: {new_crashes}")

    LATEST_JSON.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    print("macos_live_user_notification_delivery=pass")
    print(json.dumps(evidence, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
