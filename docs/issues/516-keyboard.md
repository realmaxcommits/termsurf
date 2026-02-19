# Issue 516: Keyboard Input Forwarding

## Goal

Type in input fields on web pages and press Enter to submit. The initial test:
type a search query into Google's search box and press Enter to see results.

## Background

Issues 514–515 established the mouse input pipeline (clicks, scrolling, hover,
cursor sync, text selection) and the Chromium focus lifecycle. Keyboard input is
the next piece — without it, the browser overlay is read-only.

The architecture follows the same pattern as mouse forwarding: intercept events
in CompositorXPC, forward via XPC to the Chromium Profile Server, construct
Blink input events, and forward to the renderer.

## Architecture

```
NSEvent (keyDown/keyUp)
    │
    ▼
CompositorXPC local event monitor
    │ (check: pane in browse mode?)
    │ (consume event, forward via XPC)
    ▼
XPC message: { action: "keyboard_event", ... }
    │
    ▼
Chromium Profile Server (shell_browser_main_parts.cc)
    │ (construct NativeWebKeyboardEvent)
    │ (ForwardKeyboardEvent to renderer)
    ▼
Blink renderer processes input
```

### Key Blink requirement: two events per keypress

Blink needs **two** events for text input on a keyDown:

1. `kRawKeyDown` — the physical key press (keyCode, modifiers)
2. `kChar` — the character to insert (text field)

For keyUp, only one event: `kKeyUp`.

If only `kRawKeyDown` is sent, Blink handles keyboard shortcuts but doesn't
insert text. The `kChar` event is what triggers text insertion in input fields.

### Modifier mapping

The existing mouse pipeline maps NSEvent modifier flags to Blink's
`WebInputEvent::Modifiers`:

| NSEvent   | Blink         | Value       |
| --------- | ------------- | ----------- |
| .shift    | kShiftKey     | 1 (1 << 0)  |
| .control  | kControlKey   | 2 (1 << 1)  |
| .option   | kAltKey       | 4 (1 << 2)  |
| .command  | kMetaKey      | 8 (1 << 3)  |
| isARepeat | kIsAutoRepeat | 32 (1 << 5) |

### Key code conversion

macOS uses hardware scan codes (`event.keyCode`, 0–127 range). Blink needs
Windows virtual key codes (`windows_key_code`). Chromium provides:

```cpp
#include "ui/events/keycodes/keyboard_code_conversion_mac.h"
ui::KeyboardCodeFromKeyCode(native_key_code)  // macOS → Windows VK
```

### System key detection

On macOS, Cmd+key combinations should be marked `is_system_key = true`. This
tells Blink not to handle them as text input (e.g., Cmd+Q should not type "q").

## Experiments

### Experiment 1: Basic text input

#### Goal

Type alphanumeric characters into Google's search box, press Enter, see results.
Backspace should delete characters.

#### Changes

##### CompositorXPC.swift

Add a local event monitor for `.keyDown` and `.keyUp`, registered after the
Ctrl+Esc monitor (so Ctrl+Esc is handled first):

```swift
// Register local event monitor for keyboard forwarding (Issue 516).
// In browse mode, all keys (except Ctrl+Esc, handled above) go to Chromium.
NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .keyUp]) {
    [weak self] event in
    guard let self = self else { return event }

    // Must have a key window with a focused SurfaceView.
    guard let window = NSApp.keyWindow, window.isKeyWindow else { return event }
    guard let surfaceView = window.firstResponder
            as? Ghostty.SurfaceView else { return event }
    let uuid = surfaceView.id

    // Only intercept in browse mode.
    let browsing = self.xpcQueue.sync { self.paneBrowsing[uuid] == true }
    guard browsing else { return event }

    // Map event type.
    let typeStr = event.type == .keyDown ? "down" : "up"

    // Map modifier flags.
    var mods: UInt64 = 0
    if event.modifierFlags.contains(.shift)   { mods |= 1 }
    if event.modifierFlags.contains(.control)  { mods |= 2 }
    if event.modifierFlags.contains(.option)   { mods |= 4 }
    if event.modifierFlags.contains(.command)  { mods |= 8 }
    if event.isARepeat                         { mods |= 32 }

    // Extract text.
    let chars = event.characters ?? ""
    let charsNoMods = event.charactersIgnoringModifiers ?? ""

    // Forward to Chromium.
    self.xpcQueue.async {
        guard let profile = self.paneProfiles[uuid],
              let controlConn = self.serverControlConnections[profile]
        else { return }

        let msg = xpc_dictionary_create(nil, nil, 0)
        xpc_dictionary_set_string(msg, "action", "keyboard_event")
        xpc_dictionary_set_string(msg, "pane_id", uuid.uuidString)
        xpc_dictionary_set_string(msg, "type", typeStr)
        xpc_dictionary_set_uint64(msg, "key_code", UInt64(event.keyCode))
        xpc_dictionary_set_uint64(msg, "modifiers", mods)
        xpc_dictionary_set_string(msg, "text", chars)
        xpc_dictionary_set_string(msg, "text_no_mods", charsNoMods)
        xpc_connection_send_message(controlConn, msg)
    }

    // Consume the event (prevent terminal from receiving it).
    return nil
}
```

The monitor fires after the Ctrl+Esc monitor (registered earlier in `start()`).
Ctrl+Esc in browse mode is already consumed by the first monitor, so this one
never sees it.

##### shell_browser_main_parts.h (Chromium)

Add declaration:

```cpp
void HandleKeyboardEvent(const std::string& pane_id,
                         const std::string& type,
                         uint16_t key_code,
                         uint64_t modifiers,
                         const std::string& text,
                         const std::string& text_no_mods);
```

##### shell_browser_main_parts.cc (Chromium)

Add includes:

```cpp
#include "components/input/native_web_keyboard_event.h"
#include "third_party/blink/public/common/input/web_keyboard_event.h"
#include "ui/events/keycodes/keyboard_code_conversion_mac.h"
```

Add XPC handler in `StartDynamicMode()` (alongside existing mouse handlers):

```cpp
} else if (action && std::string_view(action) == "keyboard_event") {
    const char* pane = xpc_dictionary_get_string(event, "pane_id");
    const char* type_str = xpc_dictionary_get_string(event, "type");
    uint64_t key_code = xpc_dictionary_get_uint64(event, "key_code");
    uint64_t modifiers = xpc_dictionary_get_uint64(event, "modifiers");
    const char* text_str = xpc_dictionary_get_string(event, "text");
    const char* text_no_mods = xpc_dictionary_get_string(event, "text_no_mods");
    std::string s_pane(pane ? pane : "");
    std::string s_type(type_str ? type_str : "");
    std::string s_text(text_str ? text_str : "");
    std::string s_text_no_mods(text_no_mods ? text_no_mods : "");
    content::GetUIThreadTaskRunner({})->PostTask(
        FROM_HERE,
        base::BindOnce(&ShellBrowserMainParts::HandleKeyboardEvent,
                       base::Unretained(self), s_pane, s_type,
                       static_cast<uint16_t>(key_code), modifiers,
                       s_text, s_text_no_mods));
}
```

Add handler method:

```cpp
void ShellBrowserMainParts::HandleKeyboardEvent(
    const std::string& pane_id,
    const std::string& type,
    uint16_t key_code,
    uint64_t modifiers,
    const std::string& text,
    const std::string& text_no_mods) {
  DCHECK_CURRENTLY_ON(BrowserThread::UI);

  TabState* tab = nullptr;
  for (auto& t : tabs_) {
    if (t->pane_id == pane_id) { tab = t.get(); break; }
  }
  if (!tab) return;

  auto* view = tab->shell->web_contents()->GetRenderWidgetHostView();
  if (!view) return;

  int web_modifiers = static_cast<int>(modifiers & 0x2F);  // shift,ctrl,alt,meta,autorepeat

  // Convert macOS key code to Windows virtual key code.
  int windows_key_code = ui::KeyboardCodeFromKeyCode(key_code);

  // System key: Cmd+key should not insert text.
  bool is_system_key = (web_modifiers & blink::WebInputEvent::kMetaKey) != 0;

  if (type == "down") {
    // 1. Send kRawKeyDown (physical key press).
    blink::WebKeyboardEvent raw_down(
        blink::WebInputEvent::Type::kRawKeyDown,
        web_modifiers,
        base::TimeTicks::Now());
    raw_down.windows_key_code = windows_key_code;
    raw_down.native_key_code = key_code;
    raw_down.is_system_key = is_system_key;

    // Set unmodified text for shortcut matching.
    if (!text_no_mods.empty()) {
      raw_down.unmodified_text[0] =
          static_cast<char16_t>(text_no_mods[0]);
    }

    input::NativeWebKeyboardEvent native_raw_down(raw_down, nullptr);
    view->GetRenderWidgetHost()->ForwardKeyboardEvent(native_raw_down);

    // 2. Send kChar (text insertion) — only if there's text and not a
    //    system key (Cmd+key should not insert text).
    if (!text.empty() && !is_system_key) {
      blink::WebKeyboardEvent char_event(
          blink::WebInputEvent::Type::kChar,
          web_modifiers,
          base::TimeTicks::Now());
      char_event.windows_key_code = static_cast<int>(text[0]);
      char_event.native_key_code = key_code;
      char_event.text[0] = static_cast<char16_t>(text[0]);
      char_event.unmodified_text[0] =
          text_no_mods.empty()
              ? static_cast<char16_t>(text[0])
              : static_cast<char16_t>(text_no_mods[0]);

      input::NativeWebKeyboardEvent native_char(char_event, nullptr);
      view->GetRenderWidgetHost()->ForwardKeyboardEvent(native_char);
    }
  } else if (type == "up") {
    blink::WebKeyboardEvent key_up(
        blink::WebInputEvent::Type::kKeyUp,
        web_modifiers,
        base::TimeTicks::Now());
    key_up.windows_key_code = windows_key_code;
    key_up.native_key_code = key_code;
    key_up.is_system_key = is_system_key;

    input::NativeWebKeyboardEvent native_up(key_up, nullptr);
    view->GetRenderWidgetHost()->ForwardKeyboardEvent(native_up);
  }
}
```

#### Verification

```bash
cd chromium/src && git checkout -b 146.0.7650.0-issue-516 146.0.7650.0-issue-515
export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
autoninja -C out/Default chromium_profile_server

open ts5/zig-out/TermSurf.app --stderr ~/dev/termsurf/logs/overlay.log
# In a pane:
cargo run -p web -- https://google.com
```

1. Enter browse mode — Google search box has blinking cursor.
2. Type "hello world" — characters appear in the search box.
3. Press Backspace — last character is deleted.
4. Press Enter — Google search results page loads.

Pass: basic text input works in browse mode. Characters are inserted, Backspace
deletes, Enter submits.
