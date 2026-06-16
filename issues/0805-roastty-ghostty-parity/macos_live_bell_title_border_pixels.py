#!/usr/bin/env python3
"""Live macOS bell title/border pixel guard for Issue 805 CFG-223."""

from __future__ import annotations

import json
import re
import shlex
import subprocess
import tempfile
import textwrap
import time
from pathlib import Path

from macos_window_padding_pixel_runtime import (
    APP,
    ROOT,
    Rect,
    capture_window_id,
    crash_reports,
    create_terminal_window,
    focus_evidence,
    focused_window,
    require,
    run_osascript,
    terminate_process,
    wait_for_app,
    wait_for_crash_report_settle,
    wait_for_file,
)


TITLE = "Issue805Exp190BellTitle"


def write_config(path: Path) -> None:
    path.write_text(
        "\n".join(
            [
                "macos-applescript = true",
                "quit-after-last-window-closed = true",
                "cursor-style-blink = false",
                "font-size = 16",
                "window-width = 100",
                "window-height = 34",
                "background = #102030",
                "foreground = #ffffff",
                "background-opacity = 1",
                "macos-titlebar-style = hidden",
                "window-padding-x = 0",
                "window-padding-y = 0",
                "bell-features = no-system,no-audio,no-attention,title,border",
                "",
            ]
        )
    )


def write_painter(path: Path, ready: Path, trigger: Path, bell: Path) -> None:
    path.write_text(
        textwrap.dedent(
            f"""
            from pathlib import Path
            import sys
            import time

            ready = Path({str(ready)!r})
            trigger = Path({str(trigger)!r})
            bell = Path({str(bell)!r})
            title = {TITLE!r}

            sys.stdout.write("\\x1b[?25l\\x1b[?7l\\x1b]2;" + title + "\\x07")
            sys.stdout.write("\\x1b[2J\\x1b[H")
            sys.stdout.write("\\x1b[10;20HIssue 805 Experiment 190")
            sys.stdout.write("\\x1b[12;20HWaiting for deterministic BEL")
            sys.stdout.flush()
            ready.write_text("ready")

            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if trigger.exists():
                    break
                time.sleep(0.05)
            else:
                raise SystemExit("timed out waiting for trigger")

            sys.stdout.write("\\a")
            sys.stdout.flush()
            bell.write_text("ready")
            time.sleep(90)
            """
        ).lstrip()
    )


def write_sampler(path: Path) -> None:
    path.write_text(
        r'''
import AppKit
import Foundation

struct Region: Codable {
    let x: Int
    let y: Int
    let width: Int
    let height: Int
    let changed: Int
    let samples: Int
    let maxDelta: Int
    let meanDelta: Double
}

struct Metrics: Codable {
    let width: Int
    let height: Int
    let topMask: Int
    let leftEdge: Region
    let rightEdge: Region
    let bottomEdge: Region
    let topSurfaceProbe: Region
    let center: Region
    let titlebarControl: Region
    let verdict: String
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write((message + "\n").data(using: .utf8)!)
    exit(1)
}

guard CommandLine.arguments.count == 3 else {
    fail("usage: sampler.swift <baseline.png> <bell.png>")
}

func loadBitmap(_ path: String) -> NSBitmapImageRep {
    guard let image = NSImage(contentsOfFile: path),
          let tiff = image.tiffRepresentation,
          let bitmap = NSBitmapImageRep(data: tiff)
    else {
        fail("failed to load image: \(path)")
    }
    return bitmap
}

let baseline = loadBitmap(CommandLine.arguments[1])
let bell = loadBitmap(CommandLine.arguments[2])
let width = baseline.pixelsWide
let height = baseline.pixelsHigh
if bell.pixelsWide != width || bell.pixelsHigh != height {
    fail("screenshot dimensions differ: \(width)x\(height) vs \(bell.pixelsWide)x\(bell.pixelsHigh)")
}

func colorAt(_ bitmap: NSBitmapImageRep, _ x: Int, _ y: Int) -> (Int, Int, Int) {
    guard let color = bitmap.colorAt(x: x, y: y)?.usingColorSpace(.sRGB) else {
        return (0, 0, 0)
    }
    return (
        Int((color.redComponent * 255).rounded()),
        Int((color.greenComponent * 255).rounded()),
        Int((color.blueComponent * 255).rounded())
    )
}

func clamp(_ value: Int, _ lower: Int, _ upper: Int) -> Int {
    return min(max(value, lower), upper)
}

func sampleRegion(x: Int, y: Int, w: Int, h: Int, sampleStride: Int) -> Region {
    let x0 = clamp(x, 0, width - 1)
    let y0 = clamp(y, 0, height - 1)
    let x1 = clamp(x + w - 1, 0, width - 1)
    let y1 = clamp(y + h - 1, 0, height - 1)
    if x1 <= x0 || y1 <= y0 {
        fail("empty sample region")
    }

    let step = max(1, sampleStride)
    var changed = 0
    var samples = 0
    var maxDelta = 0
    var totalDelta = 0
    for yy in stride(from: y0, through: y1, by: step) {
        for xx in stride(from: x0, through: x1, by: step) {
            let before = colorAt(baseline, xx, yy)
            let after = colorAt(bell, xx, yy)
            let redDelta = abs(after.0 - before.0)
            let greenDelta = abs(after.1 - before.1)
            let blueDelta = abs(after.2 - before.2)
            let delta = redDelta + greenDelta + blueDelta
            samples += 1
            totalDelta += delta
            maxDelta = max(maxDelta, delta)
            if delta >= 35 {
                changed += 1
            }
        }
    }
    return Region(
        x: x0,
        y: y0,
        width: x1 - x0 + 1,
        height: y1 - y0 + 1,
        changed: changed,
        samples: samples,
        maxDelta: maxDelta,
        meanDelta: Double(totalDelta) / Double(max(samples, 1))
    )
}

let topMask = min(140, max(70, height / 10))
let edge = min(22, max(10, width / 90))
let leftEdge = sampleRegion(x: 0, y: topMask, w: edge, h: height - topMask, sampleStride: 1)
let rightEdge = sampleRegion(x: width - edge, y: topMask, w: edge, h: height - topMask, sampleStride: 1)
let bottomEdge = sampleRegion(x: 0, y: height - edge, w: width, h: edge, sampleStride: 1)
let topSurfaceProbe = sampleRegion(x: 0, y: topMask, w: width, h: edge, sampleStride: 1)
let center = sampleRegion(
    x: width / 3,
    y: topMask + (height - topMask) / 3,
    w: width / 3,
    h: max(80, (height - topMask) / 3),
    sampleStride: 3
)
let titlebarControl = sampleRegion(x: 0, y: 0, w: width, h: topMask / 2, sampleStride: 3)

let edgeChanged = leftEdge.changed + rightEdge.changed + bottomEdge.changed + topSurfaceProbe.changed
let edgeSamples = leftEdge.samples + rightEdge.samples + bottomEdge.samples + topSurfaceProbe.samples
let edgeRatio = Double(edgeChanged) / Double(max(edgeSamples, 1))
let centerRatio = Double(center.changed) / Double(center.samples)
let titlebarRatio = Double(titlebarControl.changed) / Double(titlebarControl.samples)

if edgeChanged < 1200 {
    fail("surface edge bands changed too few pixels: \(edgeChanged)")
}
if edgeRatio < 0.025 {
    fail("surface edge changed ratio too low: \(edgeRatio)")
}
if leftEdge.changed < 250 || rightEdge.changed < 250 || bottomEdge.changed < 250 {
    fail("expected left/right/bottom edge deltas, got left=\(leftEdge.changed) right=\(rightEdge.changed) bottom=\(bottomEdge.changed)")
}
if centerRatio > 0.012 {
    fail("center control region changed too much: \(centerRatio)")
}
if titlebarRatio > 0.018 {
    fail("masked titlebar/control region changed too much: \(titlebarRatio)")
}
if edgeRatio < centerRatio * 5.0 {
    fail("edge delta is not sufficiently stronger than center delta: edge=\(edgeRatio) center=\(centerRatio)")
}

let metrics = Metrics(
    width: width,
    height: height,
    topMask: topMask,
    leftEdge: leftEdge,
    rightEdge: rightEdge,
    bottomEdge: bottomEdge,
    topSurfaceProbe: topSurfaceProbe,
    center: center,
    titlebarControl: titlebarControl,
    verdict: "pass"
)
let data = try! JSONEncoder().encode(metrics)
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
'''
    )


def wait_for_trace(trace: Path, needles: list[str], timeout: float = 15.0) -> str:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        text = trace.read_text(errors="replace") if trace.exists() else ""
        if all(needle in text for needle in needles):
            return text
        time.sleep(0.25)
    text = trace.read_text(errors="replace") if trace.exists() else ""
    missing = [needle for needle in needles if needle not in text]
    raise AssertionError(f"trace missing {missing}; trace was:\n{text}")


def launch_with_trace(config: Path, trace: Path) -> int:
    before = subprocess.run(
        ["pgrep", "-f", f"{APP}/Contents/MacOS/roastty"],
        text=True,
        capture_output=True,
    )
    require(not before.stdout.split(), f"debug Roastty app is already running: {before.stdout}")
    result = subprocess.run(
        [
            "open",
            "-n",
            "--env",
            f"ROASTTY_CONFIG_PATH={config}",
            "--env",
            "ROASTTY_CLEAR_USER_DEFAULTS=1",
            "--env",
            "ROASTTY_USER_DEFAULTS_SUITE=com.termsurf.roastty.issue805.exp190.belltitleborder",
            "--env",
            f"ROASTTY_UI_KEY_TRACE_PATH={trace}",
            str(APP),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    require(result.returncode == 0, f"open failed\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")

    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        after = subprocess.run(
            ["pgrep", "-f", f"{APP}/Contents/MacOS/roastty"],
            text=True,
            capture_output=True,
        )
        created = [int(value) for value in after.stdout.split()]
        if created:
            return sorted(created)[0]
        time.sleep(0.25)
    raise AssertionError("open did not start debug Roastty")


def focused_accessibility_title(pid: int) -> str:
    script = textwrap.dedent(
        f"""
        tell application "System Events"
          set roasttyProc to first application process whose unix id is {pid}
          set frontmost of roasttyProc to true
          delay 0.25
          set focusedWindow to value of attribute "AXFocusedWindow" of roasttyProc
          set axTitle to value of attribute "AXTitle" of focusedWindow
          return axTitle as text
        end tell
        """
    )
    return run_osascript(script, timeout=10).stdout.strip()


def sample_border(sampler: Path, baseline: Path, bell: Path) -> dict[str, object]:
    result = subprocess.run(
        ["swift", str(sampler), str(baseline), str(bell)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise AssertionError(
            "bell border sampler failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return json.loads(result.stdout)


def main() -> int:
    require(APP.is_dir(), f"app not built: {APP}")
    before_crashes = crash_reports()

    with tempfile.TemporaryDirectory(prefix="termsurf-issue805-exp190-bell-title-border-") as temp_dir:
        temp = Path(temp_dir)
        config = temp / "config.roastty"
        trace = temp / "trace.log"
        ready = temp / "ready.txt"
        trigger = temp / "trigger.txt"
        bell_marker = temp / "bell.txt"
        painter = temp / "paint_bell.py"
        sampler = temp / "sample_bell_border.swift"
        baseline = temp / "baseline.png"
        after_bell = temp / "after-bell.png"
        evidence = temp / "evidence.json"

        write_config(config)
        write_painter(painter, ready, trigger, bell_marker)
        write_sampler(sampler)

        pid = launch_with_trace(config, trace)

        try:
            wait_for_app(pid)
            command = f"{shlex.quote(str(Path('/usr/bin/python3')))} {shlex.quote(str(painter))}"
            terminal_id = create_terminal_window(command)
            wait_for_file(ready, "bell painter")

            focus = focus_evidence(pid)
            focused_bounds = focus["focused_bounds"]
            require(isinstance(focused_bounds, Rect), f"unexpected focused bounds: {focused_bounds}")
            window = focused_window(pid, focused_bounds)

            time.sleep(0.75)
            before_ax_title = focused_accessibility_title(pid)
            require(TITLE in before_ax_title, f"baseline AX title missing {TITLE!r}: {before_ax_title!r}")
            require(
                "🔔" not in before_ax_title,
                f"baseline AX title should not have bell prefix: {before_ax_title!r}",
            )
            baseline_width, baseline_height = capture_window_id(window.id, baseline)
            require(
                baseline_width > 0 and baseline_height > 0,
                f"empty baseline screenshot dimensions: {baseline_width}x{baseline_height}",
            )

            trigger.write_text("go")
            wait_for_file(bell_marker, "bell marker")
            trace_text = wait_for_trace(
                trace,
                [
                    "ringBell target=surface",
                    "surfaceBell state=true",
                    "appBell system=false audio=false attention=false",
                ],
                timeout=15,
            )
            time.sleep(0.6)
            after_ax_title = focused_accessibility_title(pid)
            require(TITLE in after_ax_title, f"bell AX title missing {TITLE!r}: {after_ax_title!r}")
            require(
                after_ax_title.startswith("🔔"),
                f"bell AX title should have bell prefix: {after_ax_title!r}",
            )

            bell_width, bell_height = capture_window_id(window.id, after_bell)
            require(
                (bell_width, bell_height) == (baseline_width, baseline_height),
                f"screenshot size changed: baseline={baseline_width}x{baseline_height} bell={bell_width}x{bell_height}",
            )
            metrics = sample_border(sampler, baseline, after_bell)

            data = {
                "pid": pid,
                "terminal_id": terminal_id,
                "window_id": window.id,
                "window_bounds": window.bounds.__dict__,
                "focused_bounds": focused_bounds.__dict__,
                "before_ax_title": before_ax_title,
                "after_ax_title": after_ax_title,
                "baseline_screenshot": str(baseline),
                "after_bell_screenshot": str(after_bell),
                "screenshot_size": {"width": baseline_width, "height": baseline_height},
                "sampler_metrics": metrics,
                "trace_tail": trace_text.splitlines()[-24:],
            }
            evidence.write_text(json.dumps(data, indent=2, sort_keys=True))
            Path("/tmp/termsurf-issue805-exp190-bell-title-border-latest.json").write_text(
                evidence.read_text()
            )
            Path("/tmp/termsurf-issue805-exp190-bell-title-border-baseline.png").write_bytes(
                baseline.read_bytes()
            )
            Path("/tmp/termsurf-issue805-exp190-bell-title-border-after.png").write_bytes(
                after_bell.read_bytes()
            )
        finally:
            terminate_process(pid)

    new_crashes = wait_for_crash_report_settle(before_crashes)
    require(
        not new_crashes,
        "Roastty wrote crash reports during live bell title/border workflow: "
        + ", ".join(str(path) for path in sorted(new_crashes)),
    )

    print("macos_live_bell_title_border_pixels=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
