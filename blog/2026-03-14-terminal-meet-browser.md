+++
title = "Terminal, Meet Browser"
author = "Ryan X. Charles"
date = "2026-03-14"
+++

We live in terminals. We write code, run builds, tail logs, manage servers, and
navigate filesystems without touching a mouse. The terminal is fast. It is
composable. It is keyboard-driven. It is where focused work happens.

Then we need a browser.

`localhost:3000`. Documentation. A dashboard. A pull request. Whatever it is, we
alt-tab out, lose our place, and shatter the flow we spent minutes building. The
browser is a separate world — its own window manager, its own keybindings, its
own gravity. Every time we leave the terminal, we lose signal.

TermSurf kills the context switch.

## The Protocol

TermSurf is a protocol for jacking web browsers into terminal emulators. Type
`web localhost:3000` and the page renders right there — in your terminal pane,
next to your shell, next to your code. No new window. No alt-tab. No broken
flow.

This is not a text-mode browser. This is a real engine — full CSS, full
JavaScript, full GPU rendering — composited directly into the terminal window.
The same pixels you would see in Chrome or Safari, inside the tool you already
live in.

## How It Works

Three components. One protocol. Unix sockets and protobuf.

```
┌─────────┐  ┌─────────┐  ┌─────────┐
│  TUI 1  │  │  TUI 2  │  │  TUI N  │    N TUIs (e.g., `web`)
└────┬────┘  └────┬────┘  └────┬────┘
     │            │            │
     └────────────┼────────────┘
                  │  Unix socket
           ┌──────┴──────┐
           │     GUI     │                1 GUI (terminal emulator)
           │ (Wezboard)  │
           └──┬───┬───┬──┘
              │   │   │
              │   │   │  Unix sockets
              │   │   │
     ┌────────┘   │   └──────┐
     │            │          │
┌────┴────┐ ┌─────┴───┐ ┌────┴────┐
│ Roamium │ │ Surfari │ │ Roamium │    M engines (one per profile)
│ profile │ │ profile │ │ profile │
│   "A"   │ │   "B"   │ │   "C"   │
└─────────┘ └─────────┘ └─────────┘
```

**The GUI** is a terminal emulator that speaks the TermSurf protocol. It listens
on a Unix socket, accepts connections from TUIs and browser engines, and
composites browser content as overlays at pixel coordinates. The current GUI is
Wezboard — a fork of WezTerm.

**The TUI** is a terminal app that provides browser chrome — URL bar, navigation,
vim-style modes — inside a pane. The current TUI is `web`. It connects to the
GUI over the socket and fires protocol messages to control the browser.

**The engine** runs as a separate process. One process per profile. Each engine
connects to the GUI and renders web content into a GPU surface. On macOS, this
uses CALayerHost — zero-copy compositing. The browser's GPU output appears
directly in the terminal window. No pixel copying. No intermediate buffers.

The protocol — `termsurf.proto` — defines 30+ message types. Tab lifecycle.
Navigation. Input forwarding. GPU compositing. State sync. Request/reply pairs.
All length-prefixed protobuf over Unix domain sockets.

## Any Engine

Every engine is a separate process speaking the same protocol. TermSurf is not
locked to Chromium.

| Engine   | Binary    | Status  |
| -------- | --------- | ------- |
| Chromium | Roamium   | Working |
| WebKit   | Surfari   | Planned |
| Gecko    | Waterwolf | Planned |
| Ladybird | Girlbat   | Planned |

Same pattern for each: a C shared library wrapping the engine's embedding API,
linked by a Rust binary that handles IPC, protobuf, and process lifecycle. The
Rust binary is ~400 lines. Almost entirely reusable across engines.

One pane running Roamium. Another running Surfari. A third running Girlbat. Same
terminal window. Same protobuf messages. Different engines.

## Any Terminal

Any terminal emulator that implements the protocol can host browser overlays.
Wezboard is the current GUI. But the protocol is designed so that forks of
Ghostty, Kitty, Alacritty, and iTerm2 could all serve as TermSurf GUIs.

The protocol is the product. The apps are implementations.

## What Works Now

Wezboard and Roamium are functional on macOS. Here is what you can do today:

- Jack into any URL from a terminal pane: `web <url>`.
- Split panes — the overlay repositions and resizes instantly.
- Switch tabs — overlays hide and restore.
- Open DevTools for any browser pane.
- Navigate with keyboard-driven modes. Vim-style.
- Run multiple profiles with isolated cookies and storage.
- GPU-accelerated rendering. Zero pixel copying. CALayerHost compositing.

You split a terminal. Type `web localhost:3000`. Your app appears next to your
editor. Resize the split — the browser follows. Switch to another tab to run
tests. Switch back. The browser is exactly where you left it.

## What Is Next

TermSurf is early. The protocol works. The rendering works. The core workflow is
solid. The road ahead:

- WebKit and Gecko engine integrations.
- Linux and Windows.
- Forks of more terminals.
- Bookmarks, history, downloads.
- A richer TUI — tab bar, status line, search.

The most important work is on the protocol. Every feature starts as a protobuf
message. The protocol is versioned, extensible, and built to grow.

## Jack In

TermSurf is open source. Code, protocol, issues, experiments — all public.

- [GitHub](https://github.com/termsurf/termsurf)
- [Website](https://termsurf.com)

If you have ever wanted to see a web page without leaving your terminal, this is
for you.
