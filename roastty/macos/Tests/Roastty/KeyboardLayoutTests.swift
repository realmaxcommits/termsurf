import Carbon
import RoasttyKit
import Testing

struct KeyboardLayoutTests {
    @Test func currentKeyboardLayoutUsesHostLayoutSource() throws {
        let observed = roastty_current_keyboard_layout()
        #expect(isValidKeyboardLayout(observed))

        guard let expected = try expectedHostLayout() else {
            return
        }

        #expect(observed == expected)
    }

    private func isValidKeyboardLayout(_ layout: roastty_keyboard_layout_e) -> Bool {
        switch layout {
        case ROASTTY_KEYBOARD_LAYOUT_UNKNOWN,
             ROASTTY_KEYBOARD_LAYOUT_US_STANDARD,
             ROASTTY_KEYBOARD_LAYOUT_US_INTERNATIONAL:
            return true
        default:
            return false
        }
    }

    private func expectedHostLayout() throws -> roastty_keyboard_layout_e? {
        guard let id = try currentKeyboardLayoutSourceID() else {
            return nil
        }

        switch id {
        case "com.apple.keylayout.US":
            return ROASTTY_KEYBOARD_LAYOUT_US_STANDARD
        case "com.apple.keylayout.USInternational":
            return ROASTTY_KEYBOARD_LAYOUT_US_INTERNATIONAL
        default:
            return nil
        }
    }

    private func currentKeyboardLayoutSourceID() throws -> String? {
        guard let source = TISCopyCurrentKeyboardLayoutInputSource()?.takeRetainedValue() else {
            return nil
        }
        guard let sourceIdPointer = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            return nil
        }

        let sourceId = unsafeBitCast(sourceIdPointer, to: CFString.self)
        return sourceId as String
    }
}
