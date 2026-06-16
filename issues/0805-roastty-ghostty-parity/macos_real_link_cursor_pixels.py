#!/usr/bin/env python3
"""Real macOS link cursor pixel guard for Issue 805 CFG-223."""

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
    terminate_process,
    wait_for_app,
    wait_for_crash_report_settle,
    wait_for_file,
)


INJECT = ROOT / "scripts/ghostty-app/inject.swift"
URL = "https://example.com/issue805-exp191-real-cursor"
LINK_ROW = 8
LINK_COL = 10
PROBE_ROW = 20
PROBE_COL = 70
RECT_SIZE = 180


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
                "link-previews = true",
                "",
            ]
        )
    )


def write_painter(path: Path, marker: Path) -> None:
    path.write_text(
        textwrap.dedent(
            f"""
            from pathlib import Path
            import sys
            import time

            marker = Path({str(marker)!r})
            url = {URL!r}
            sys.stdout.write("\\x1b[?25l\\x1b[?7l\\x1b[2J\\x1b[H")
            sys.stdout.write("\\x1b[{LINK_ROW};{LINK_COL}H" + url)
            sys.stdout.write("\\x1b[{PROBE_ROW};{PROBE_COL}H")
            sys.stdout.flush()
            marker.write_text("ready")
            time.sleep(120)
            """
        ).lstrip()
    )


def write_sampler(path: Path) -> None:
    path.write_text(
        r'''
import AppKit
import Foundation

struct Box: Codable {
    let x: Int
    let y: Int
    let width: Int
    let height: Int
    let centerX: Double
    let centerY: Double
}

struct Mask: Codable {
    let changed: Int
    let stableChanged: Int
    let bbox: Box
}

struct Metrics: Codable {
    let width: Int
    let height: Int
    let nonLink: Mask
    let link: Mask
    let symmetricDifference: Int
    let largerMask: Int
    let bboxDelta: Double
    let verdict: String
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write((message + "\n").data(using: .utf8)!)
    exit(1)
}

guard CommandLine.arguments.count == 7 else {
    fail("usage: sampler.swift <nonlink-none-1> <nonlink-none-2> <nonlink-cursor> <link-none-1> <link-none-2> <link-cursor>")
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

let nonLinkNone1 = loadBitmap(CommandLine.arguments[1])
let nonLinkNone2 = loadBitmap(CommandLine.arguments[2])
let nonLinkCursor = loadBitmap(CommandLine.arguments[3])
let linkNone1 = loadBitmap(CommandLine.arguments[4])
let linkNone2 = loadBitmap(CommandLine.arguments[5])
let linkCursor = loadBitmap(CommandLine.arguments[6])
let width = nonLinkNone1.pixelsWide
let height = nonLinkNone1.pixelsHigh
for bitmap in [nonLinkNone2, nonLinkCursor, linkNone1, linkNone2, linkCursor] {
    if bitmap.pixelsWide != width || bitmap.pixelsHigh != height {
        fail("all captures must have identical dimensions")
    }
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

func delta(_ a: NSBitmapImageRep, _ b: NSBitmapImageRep, _ x: Int, _ y: Int) -> Int {
    let ca = colorAt(a, x, y)
    let cb = colorAt(b, x, y)
    return abs(ca.0 - cb.0) + abs(ca.1 - cb.1) + abs(ca.2 - cb.2)
}

func countChanged(_ a: NSBitmapImageRep, _ b: NSBitmapImageRep) -> Int {
    var changed = 0
    for y in 0..<height {
        for x in 0..<width {
            if delta(a, b, x, y) >= 45 {
                changed += 1
            }
        }
    }
    return changed
}

func makeMask(stableA: NSBitmapImageRep, stableB: NSBitmapImageRep, cursor: NSBitmapImageRep, name: String) -> Mask {
    let stableChanged = countChanged(stableA, stableB)
    if stableChanged >= 80 {
        fail("\(name) cursorless background changed too much: \(stableChanged)")
    }

    var changed = 0
    var minX = width
    var minY = height
    var maxX = -1
    var maxY = -1
    for y in 0..<height {
        for x in 0..<width {
            if delta(stableB, cursor, x, y) >= 45 {
                changed += 1
                minX = min(minX, x)
                minY = min(minY, y)
                maxX = max(maxX, x)
                maxY = max(maxY, y)
            }
        }
    }
    if changed < 150 {
        fail("\(name) cursor mask too small: \(changed)")
    }
    if minX > maxX || minY > maxY {
        fail("\(name) cursor mask has no bounds")
    }
    let boxWidth = maxX - minX + 1
    let boxHeight = maxY - minY + 1
    if boxWidth < 6 || boxWidth > 80 || boxHeight < 10 || boxHeight > 100 {
        fail("\(name) cursor mask bounds out of range: \(boxWidth)x\(boxHeight)")
    }
    let centerX = Double(minX + maxX) / 2.0
    let centerY = Double(minY + maxY) / 2.0
    if centerX < Double(width) * 0.25 || centerX > Double(width) * 0.75 ||
        centerY < Double(height) * 0.25 || centerY > Double(height) * 0.75 {
        fail("\(name) cursor mask not near capture center: \(centerX),\(centerY)")
    }
    return Mask(
        changed: changed,
        stableChanged: stableChanged,
        bbox: Box(
            x: minX,
            y: minY,
            width: boxWidth,
            height: boxHeight,
            centerX: centerX,
            centerY: centerY
        )
    )
}

func maskBit(_ base: NSBitmapImageRep, _ cursor: NSBitmapImageRep, _ x: Int, _ y: Int) -> Bool {
    return delta(base, cursor, x, y) >= 45
}

let nonLink = makeMask(stableA: nonLinkNone1, stableB: nonLinkNone2, cursor: nonLinkCursor, name: "non-link")
let link = makeMask(stableA: linkNone1, stableB: linkNone2, cursor: linkCursor, name: "link")

var symmetricDifference = 0
for y in 0..<height {
    for x in 0..<width {
        let a = maskBit(nonLinkNone2, nonLinkCursor, x, y)
        let b = maskBit(linkNone2, linkCursor, x, y)
        if a != b {
            symmetricDifference += 1
        }
    }
}

let largerMask = max(nonLink.changed, link.changed)
let bboxDelta = max(
    abs(Double(nonLink.bbox.width - link.bbox.width)),
    max(
        abs(Double(nonLink.bbox.height - link.bbox.height)),
        max(abs(nonLink.bbox.centerX - link.bbox.centerX), abs(nonLink.bbox.centerY - link.bbox.centerY))
    )
)
if symmetricDifference < Int((Double(largerMask) * 0.25).rounded(.up)) && bboxDelta < 8.0 {
    fail("link/non-link cursor masks are too similar: symmetricDifference=\(symmetricDifference) largerMask=\(largerMask) bboxDelta=\(bboxDelta)")
}

let metrics = Metrics(
    width: width,
    height: height,
    nonLink: nonLink,
    link: link,
    symmetricDifference: symmetricDifference,
    largerMask: largerMask,
    bboxDelta: bboxDelta,
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


def wait_for_resize(trace: Path, timeout: float = 10.0) -> dict[str, int]:
    deadline = time.monotonic() + timeout
    pattern = re.compile(r"resize rows=(\d+) cols=(\d+) width_px=(\d+) height_px=(\d+)")
    last_match = None
    while time.monotonic() < deadline:
        text = trace.read_text(errors="replace") if trace.exists() else ""
        for match in pattern.finditer(text):
            last_match = match
        if last_match:
            rows, cols, width_px, height_px = (int(value) for value in last_match.groups())
            return {
                "rows": rows,
                "cols": cols,
                "width_px": width_px,
                "height_px": height_px,
            }
        time.sleep(0.25)
    text = trace.read_text(errors="replace") if trace.exists() else ""
    raise AssertionError(f"trace never reported a terminal resize; trace was:\n{text}")


def global_point(window: Rect, row: int, col: int, cols: int, rows: int) -> tuple[float, float]:
    cell_width = window.width / cols
    cell_height = window.height / rows
    x = window.x + (col - 0.5) * cell_width
    y = window.y + (row - 0.5) * cell_height
    return x, y


def hover_trace_seen(trace_text: str) -> bool:
    return (
        "cursorShape raw=" in trace_text
        and "pointerStyle=link" in trace_text
        and f"mouseOverLink url={URL}" in trace_text
    )


def inject_move(x: float, y: float, *modifiers: str) -> None:
    result = subprocess.run(
        ["swift", str(INJECT), "move", f"{x:.1f}", f"{y:.1f}", *modifiers],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=10,
    )
    if result.returncode != 0:
        raise AssertionError(
            "mouse injection failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )


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
            "ROASTTY_USER_DEFAULTS_SUITE=com.termsurf.roastty.issue805.exp191.cursorpixels",
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


def capture_rect(center_x: float, center_y: float, scale: float, output: Path, include_cursor: bool) -> None:
    size = RECT_SIZE
    # screencapture -R takes global display coordinates in points, not Retina
    # backing pixels. The captured image may still be Retina-scaled, so keep the
    # measured scale in evidence but do not apply it to the source rectangle.
    _ = scale
    x = int(round(center_x - size / 2))
    y = int(round(center_y - size / 2))
    args = ["screencapture", "-x", f"-R{x},{y},{size},{size}"]
    if include_cursor:
        args.append("-C")
    args.append(str(output))
    result = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise AssertionError(
            "screencapture rect failed\n"
            f"args={args}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    require(output.is_file(), f"missing rect capture: {output}")


def sample_masks(sampler: Path, captures: dict[str, Path]) -> dict[str, object]:
    result = subprocess.run(
        [
            "swift",
            str(sampler),
            str(captures["nonlink_none_1"]),
            str(captures["nonlink_none_2"]),
            str(captures["nonlink_cursor"]),
            str(captures["link_none_1"]),
            str(captures["link_none_2"]),
            str(captures["link_cursor"]),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise AssertionError(
            "cursor pixel sampler failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return json.loads(result.stdout)


def main() -> int:
    require(APP.is_dir(), f"app not built: {APP}")
    before_crashes = crash_reports()

    with tempfile.TemporaryDirectory(prefix="termsurf-issue805-exp191-real-cursor-") as temp_dir:
        temp = Path(temp_dir)
        config = temp / "config.roastty"
        trace = temp / "trace.log"
        marker = temp / "marker.txt"
        painter = temp / "paint_link.py"
        sampler = temp / "sample_cursor_masks.swift"
        window_screenshot = temp / "window.png"
        evidence = temp / "evidence.json"
        captures = {
            "nonlink_none_1": temp / "nonlink-none-1.png",
            "nonlink_none_2": temp / "nonlink-none-2.png",
            "nonlink_cursor": temp / "nonlink-cursor.png",
            "link_none_1": temp / "link-none-1.png",
            "link_none_2": temp / "link-none-2.png",
            "link_cursor": temp / "link-cursor.png",
        }

        write_config(config)
        write_painter(painter, marker)
        write_sampler(sampler)

        pid = launch_with_trace(config, trace)

        try:
            wait_for_app(pid)
            command = f"{shlex.quote(str(Path('/usr/bin/python3')))} {shlex.quote(str(painter))}"
            terminal_id = create_terminal_window(command)
            wait_for_file(marker, "link painter")
            resize = wait_for_resize(trace)

            focus = focus_evidence(pid)
            focused_bounds = focus["focused_bounds"]
            require(isinstance(focused_bounds, Rect), f"unexpected focused bounds: {focused_bounds}")
            window = focused_window(pid, focused_bounds)
            window_width_px, _ = capture_window_id(window.id, window_screenshot)
            scale = window_width_px / window.bounds.width
            require(scale >= 1.0, f"unexpected capture scale: {scale}")

            nonlink_x, nonlink_y = global_point(
                window.bounds,
                PROBE_ROW,
                PROBE_COL,
                resize["cols"],
                resize["rows"],
            )
            inject_move(nonlink_x, nonlink_y)
            time.sleep(0.4)
            capture_rect(nonlink_x, nonlink_y, scale, captures["nonlink_none_1"], include_cursor=False)
            time.sleep(0.1)
            capture_rect(nonlink_x, nonlink_y, scale, captures["nonlink_none_2"], include_cursor=False)
            time.sleep(0.1)
            capture_rect(nonlink_x, nonlink_y, scale, captures["nonlink_cursor"], include_cursor=True)

            attempts = []
            target_cols = [LINK_COL + 4, LINK_COL + 8, LINK_COL + 12, LINK_COL + 20, LINK_COL + 32]
            vertical_offsets = [0, 15, 25, 35, 45, -15, -30, 60, 75, 90, 105]
            link_x = 0.0
            link_y = 0.0
            for target_col in target_cols:
                base_x, base_y = global_point(
                    window.bounds,
                    LINK_ROW,
                    target_col,
                    resize["cols"],
                    resize["rows"],
                )
                for vertical_offset in vertical_offsets:
                    x = base_x
                    y = base_y + vertical_offset
                    inject_move(x, y, "command")
                    time.sleep(0.15)
                    inject_move(x, y, "command")
                    time.sleep(0.35)
                    attempts.append(
                        {
                            "row": LINK_ROW,
                            "col": target_col,
                            "x": x,
                            "y": y,
                            "vertical_offset": vertical_offset,
                        }
                    )
                    trace_text = trace.read_text(errors="replace") if trace.exists() else ""
                    if hover_trace_seen(trace_text):
                        link_x = x
                        link_y = y
                        break
                else:
                    continue
                break

            trace_text = wait_for_trace(
                trace,
                [
                    "cursorShape raw=",
                    "pointerStyle=link",
                    f"mouseOverLink url={URL}",
                ],
                timeout=15,
            )
            require(link_x > 0 and link_y > 0, "link cursor target was not captured")
            time.sleep(0.4)
            capture_rect(link_x, link_y, scale, captures["link_none_1"], include_cursor=False)
            time.sleep(0.1)
            capture_rect(link_x, link_y, scale, captures["link_none_2"], include_cursor=False)
            time.sleep(0.1)
            capture_rect(link_x, link_y, scale, captures["link_cursor"], include_cursor=True)

            metrics = sample_masks(sampler, captures)
            data = {
                "pid": pid,
                "terminal_id": terminal_id,
                "window_id": window.id,
                "window_bounds": window.bounds.__dict__,
                "focused_bounds": focused_bounds.__dict__,
                "resize": resize,
                "scale": scale,
                "nonlink_point": {"x": nonlink_x, "y": nonlink_y},
                "link_point": {"x": link_x, "y": link_y},
                "attempts": attempts,
                "captures": {key: str(path) for key, path in captures.items()},
                "sampler_metrics": metrics,
                "trace_tail": trace_text.splitlines()[-24:],
            }
            evidence.write_text(json.dumps(data, indent=2, sort_keys=True))
            Path("/tmp/termsurf-issue805-exp191-real-cursor-latest.json").write_text(
                evidence.read_text()
            )
            for key, path in captures.items():
                Path(f"/tmp/termsurf-issue805-exp191-real-cursor-{key}.png").write_bytes(
                    path.read_bytes()
                )
        finally:
            terminate_process(pid)

    new_crashes = wait_for_crash_report_settle(before_crashes)
    require(
        not new_crashes,
        "Roastty wrote crash reports during real cursor pixel workflow: "
        + ", ".join(str(path) for path in sorted(new_crashes)),
    )

    print("macos_real_link_cursor_pixels=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
