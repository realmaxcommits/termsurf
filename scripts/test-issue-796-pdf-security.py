#!/usr/bin/env python3
"""Security probes for Issue 796 PDF extension boundaries."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import socket
import struct
import subprocess
import sys
import time
from dataclasses import dataclass
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
ROAMIUM = ROOT / "chromium/src/out/Default/roamium"
TOOLBAR_HARNESS = ROOT / "scripts/test-issue-794-pdf-toolbar.py"
PDF_EXTENSION_ID = "mhjfbmdgcfjbbpaeojofohoefgiehjai"
FAKE_EXTENSION_ID = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"


def varint(value: int) -> bytes:
    out = bytearray()
    while value >= 0x80:
        out.append((value & 0x7F) | 0x80)
        value >>= 7
    out.append(value)
    return bytes(out)


def read_varint(buf: bytes, index: int) -> tuple[int, int]:
    shift = 0
    value = 0
    while index < len(buf):
        byte = buf[index]
        index += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, index
        shift += 7
    return 0, index


def field(number: int, wire_type: int) -> bytes:
    return varint((number << 3) | wire_type)


def string_field(number: int, value: str) -> bytes:
    data = value.encode("utf-8")
    return field(number, 2) + varint(len(data)) + data


def varint_field(number: int, value: int) -> bytes:
    return field(number, 0) + varint(value)


def bool_field(number: int, value: bool) -> bytes:
    return field(number, 0) + varint(1 if value else 0)


def double_field(number: int, value: float) -> bytes:
    return field(number, 1) + struct.pack("<d", value)


def wrap(inner_field: int, payload: bytes) -> bytes:
    return field(inner_field, 2) + varint(len(payload)) + payload


def send_message(conn: socket.socket, inner_field: int, payload: bytes) -> None:
    message = wrap(inner_field, payload)
    conn.sendall(struct.pack("<I", len(message)) + message)


def inner_payload(payload: bytes) -> tuple[int, bytes]:
    key, index = read_varint(payload, 0)
    length, index = read_varint(payload, index)
    return key >> 3, payload[index : index + length]


def tab_ready_id(payload: bytes) -> int | None:
    index = 0
    while index < len(payload):
        key, index = read_varint(payload, index)
        field_number = key >> 3
        wire_type = key & 7
        if wire_type == 0:
            value, index = read_varint(payload, index)
            if field_number == 2:
                return value
        elif wire_type == 2:
            length, index = read_varint(payload, index)
            index += length
        else:
            return None
    return None


def create_tab_payload(url: str, width: int, height: int) -> bytes:
    return (
        string_field(1, url)
        + string_field(2, "fake-pane")
        + varint_field(3, width)
        + varint_field(4, height)
        + bool_field(5, False)
    )


def resize_payload(tab_id: int, width: int, height: int) -> bytes:
    return (
        varint_field(1, tab_id)
        + varint_field(2, width)
        + varint_field(3, height)
        + double_field(4, 0.0)
        + double_field(5, 0.0)
        + double_field(6, float(width))
        + double_field(7, float(height))
        + double_field(8, 1.0)
    )


def read_text(path: pathlib.Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return ""


def collect_log_text(path: pathlib.Path) -> str:
    chunks: list[str] = []
    if path.is_file():
        return read_text(path)
    for child in sorted(path.rglob("*")):
        if child.is_file() and child.suffix not in {".png", ".pdf"}:
            chunks.append(f"\n===== {child.relative_to(path)} =====\n")
            chunks.append(read_text(child))
    return "\n".join(chunks)


def write_json(path: pathlib.Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


@dataclass
class FakeNavigationResult:
    server_register_received: bool = False
    create_tab_sent: bool = False
    tab_ready_id: int | None = None
    resize_sent: bool = False


def run_positive_pdf_probe(log_dir: pathlib.Path) -> dict[str, Any]:
    positive_dir = log_dir / "positive-pdf"
    cmd = [
        sys.executable,
        str(TOOLBAR_HARNESS),
        "--log-dir",
        str(positive_dir),
        "--serve-bitcoin-pdf",
        "--probe",
        "toolbar",
    ]
    proc = subprocess.run(
        cmd,
        cwd=str(ROOT),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    (log_dir / "positive-harness.stdout").write_text(proc.stdout, encoding="utf-8")
    (log_dir / "positive-harness.stderr").write_text(proc.stderr, encoding="utf-8")
    text = collect_log_text(positive_dir)
    required_labels = [
        "process-per-site",
        "process-map-insert",
        "pdf-activate-request",
        "chrome-resources-grant",
    ]
    return {
        "command": cmd,
        "returncode": proc.returncode,
        "required_labels": {
            label: label in text and PDF_EXTENSION_ID in text
            for label in required_labels
        },
    }


def run_fake_extension_navigation(
    log_dir: pathlib.Path,
    fake_url: str,
    width: int,
    height: int,
    timeout: float,
    settle_seconds: float,
) -> FakeNavigationResult:
    fake_dir = log_dir / "fake-extension"
    fake_dir.mkdir(parents=True, exist_ok=True)
    socket_path = fake_dir / "gui.sock"
    try:
        socket_path.unlink()
    except FileNotFoundError:
        pass

    result = FakeNavigationResult()
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(str(socket_path))
    listener.listen(1)
    listener.settimeout(timeout)

    stdout = (fake_dir / "roamium.stdout").open("wb")
    stderr = (fake_dir / "roamium.stderr").open("wb")
    env = os.environ.copy()
    proc = subprocess.Popen(
        [
            str(ROAMIUM),
            f"--ipc-socket={socket_path}",
            f"--user-data-dir={fake_dir / 'profile'}",
            "--no-sandbox",
        ],
        cwd=str(ROOT / "chromium/src"),
        stdout=stdout,
        stderr=stderr,
        env=env,
    )

    conn: socket.socket | None = None
    start = time.time()
    try:
        conn, _ = listener.accept()
        conn.settimeout(0.2)
        with (fake_dir / "messages.log").open("w", encoding="utf-8") as messages:
            while time.time() - start < timeout:
                try:
                    header = conn.recv(4)
                except socket.timeout:
                    continue
                if not header:
                    break
                size = struct.unpack("<I", header)[0]
                payload = bytearray()
                while len(payload) < size:
                    payload.extend(conn.recv(size - len(payload)))
                top, body = inner_payload(bytes(payload))
                messages.write(f"t={time.time() - start:.3f} top_field={top}\n")
                messages.flush()
                if top == 12 and not result.create_tab_sent:
                    result.server_register_received = True
                    send_message(conn, 1, create_tab_payload(fake_url, width, height))
                    result.create_tab_sent = True
                    messages.write("sent CreateTab fake extension URL\n")
                    messages.flush()
                elif top == 13:
                    result.tab_ready_id = tab_ready_id(body)
                    messages.write(f"tab_ready id={result.tab_ready_id}\n")
                    if result.tab_ready_id:
                        send_message(
                            conn,
                            3,
                            resize_payload(result.tab_ready_id, width, height),
                        )
                        result.resize_sent = True
                    break
        time.sleep(settle_seconds)
        return result
    finally:
        if conn:
            conn.close()
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        stdout.close()
        stderr.close()
        listener.close()


def inspect_fake_extension_logs(log_dir: pathlib.Path, fake_url: str) -> dict[str, Any]:
    fake_text = collect_log_text(log_dir / "fake-extension")
    forbidden_labels = [
        "handled-url",
        "process-per-site",
        "process-map-insert",
        "pdf-activate-request",
        "chrome-resources-grant",
        "chrome-resources-factory",
    ]
    matching_forbidden_lines = [
        line
        for line in fake_text.splitlines()
        if "[termsurf-pdf]" in line
        and (fake_url in line or FAKE_EXTENSION_ID in line)
        and any(label in line for label in forbidden_labels)
    ]
    return {
        "forbidden_labels": {
            label: label in fake_text and FAKE_EXTENSION_ID in fake_text
            for label in forbidden_labels
        },
        "matching_forbidden_lines": matching_forbidden_lines,
    }


def inspect_api_static_guards() -> dict[str, bool]:
    resources = (
        ROOT
        / "chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc"
    )
    title = (
        ROOT
        / "chromium/src/content/libtermsurf_chromium/extensions/ts_pdf_viewer_private_api.cc"
    )
    return {
        "resourcesPrivate_sender_guard": "IsTsPdfExtensionFunctionSender(this)"
        in read_text(resources),
        "pdfViewerPrivate_sender_guard": "IsTsPdfExtensionFunctionSender(this)"
        in read_text(title),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log-dir", required=True)
    parser.add_argument(
        "--fake-url",
        default=f"chrome-extension://{FAKE_EXTENSION_ID}/index.html",
    )
    parser.add_argument("--width", type=int, default=1024)
    parser.add_argument("--height", type=int, default=768)
    parser.add_argument("--setup-timeout", type=float, default=20.0)
    parser.add_argument("--settle-seconds", type=float, default=2.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    log_dir = pathlib.Path(args.log_dir).resolve()
    log_dir.mkdir(parents=True, exist_ok=True)

    positive = run_positive_pdf_probe(log_dir)
    fake_navigation = run_fake_extension_navigation(
        log_dir,
        args.fake_url,
        args.width,
        args.height,
        args.setup_timeout,
        args.settle_seconds,
    )
    fake_logs = inspect_fake_extension_logs(log_dir, args.fake_url)
    api_static_guards = inspect_api_static_guards()

    positive_pass = positive["returncode"] == 0 and all(
        positive["required_labels"].values()
    )
    fake_pass = (
        fake_navigation.create_tab_sent
        and fake_navigation.tab_ready_id is not None
        and fake_navigation.resize_sent
        and not any(fake_logs["forbidden_labels"].values())
        and not fake_logs["matching_forbidden_lines"]
    )
    api_pass = all(api_static_guards.values())
    status = "pass" if positive_pass and fake_pass and api_pass else "fail"

    summary = {
        "status": status,
        "positive": positive,
        "fake_navigation": fake_navigation.__dict__,
        "fake_logs": fake_logs,
        "api_static_guards": api_static_guards,
    }
    write_json(log_dir / "issue-796-pdf-security-summary.json", summary)
    print(json.dumps(summary, indent=2))
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
