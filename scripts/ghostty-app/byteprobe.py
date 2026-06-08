#!/usr/bin/env python3
# Issue 802 / Exp 5 — raw-mode PTY byte logger. Run INSIDE the target terminal:
#   python3 byteprobe.py <logfile> [modes]
# modes is a comma list of: mouse (1000+1002+1006), anymotion (1003), focus (1004),
# paste (2004). Puts the tty in raw mode with ISIG OFF (so Ctrl-C/D/Z arrive as bytes
# 03/04/1a, not signals), VMIN=1, and appends each read as hex to <logfile>, flushed
# per read. Exit: receive 0x1d 0x1d (Ctrl-] Ctrl-]) or be killed externally (pkill).
import sys, os, termios, atexit, signal

logpath = sys.argv[1] if len(sys.argv) > 1 else "/tmp/ghostty-exp5/bytes.log"
modes = sys.argv[2] if len(sys.argv) > 2 else ""
fd = sys.stdin.fileno()
old = termios.tcgetattr(fd)


def restore():
    termios.tcsetattr(fd, termios.TCSADRAIN, old)
    sys.stdout.write("\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1004l\x1b[?2004l")
    sys.stdout.flush()


atexit.register(restore)
# Make `pkill`/SIGTERM exit cleanly so atexit restore() runs — otherwise the terminal is
# left in mouse-reporting/raw mode and the next input class is corrupted.
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))

new = termios.tcgetattr(fd)
new[3] &= ~(termios.ICANON | termios.ECHO | termios.ISIG | termios.IEXTEN)
new[0] &= ~(termios.IXON | termios.ICRNL | termios.INLCR | termios.IGNCR | termios.BRKINT | termios.ISTRIP)
new[6][termios.VMIN] = 1
new[6][termios.VTIME] = 0
termios.tcsetattr(fd, termios.TCSANOW, new)

seq = ""
if "mouse" in modes:
    seq += "\x1b[?1000h\x1b[?1002h\x1b[?1006h"
if "anymotion" in modes:
    seq += "\x1b[?1003h"
if "focus" in modes:
    seq += "\x1b[?1004h"
if "paste" in modes:
    seq += "\x1b[?2004h"
if seq:
    sys.stdout.write(seq)
    sys.stdout.flush()

log = open(logpath, "w", buffering=1)
log.write("# byteprobe start modes=%r\n" % modes)
while True:
    b = os.read(fd, 64)
    if not b:
        break
    log.write(" ".join("%02x" % c for c in b) + "\n")
    log.flush()
    if b.endswith(b"\x1d\x1d"):
        break
