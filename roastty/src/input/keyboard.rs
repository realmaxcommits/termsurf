//! Keyboard layout (port of upstream `input/keyboard`).

use crate::input::key_mods::OptionAsAlt;

/// Keyboard layouts. These aren't heavily used; roastty only needs to distinguish a few
/// layouts for nice-to-have features such as the default for `macos-option-as-alt`
/// (upstream `input.keyboard.Layout`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Layout {
    /// Unknown, unmapped layout; make no assumptions about the keyboard layout.
    #[default]
    Unknown,
    UsStandard,
    UsInternational,
}

impl Layout {
    /// Map an Apple keyboard-layout ID (from Carbon's
    /// `TIKeyboardLayoutGetInputSourceProperty`) to a `Layout`, or `None` if the ID is
    /// unrecognized — so callers can detect that scenario.
    pub(crate) fn map_apple_id(id: &str) -> Option<Layout> {
        match id {
            "com.apple.keylayout.US" => Some(Layout::UsStandard),
            "com.apple.keylayout.USInternational" => Some(Layout::UsInternational),
            _ => None,
        }
    }

    /// The default `macos-option-as-alt` value for this layout. On US layouts the option
    /// key is typically wanted as alt (option-B ⇒ alt-B, not "∫"); on an unknown layout
    /// make no assumption.
    pub(crate) fn detect_option_as_alt(self) -> OptionAsAlt {
        match self {
            Layout::UsStandard | Layout::UsInternational => OptionAsAlt::True,
            Layout::Unknown => OptionAsAlt::False,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_apple_id_recognizes_exactly_the_us_layouts() {
        assert_eq!(
            Layout::map_apple_id("com.apple.keylayout.US"),
            Some(Layout::UsStandard),
        );
        assert_eq!(
            Layout::map_apple_id("com.apple.keylayout.USInternational"),
            Some(Layout::UsInternational),
        );
        // Unrecognized IDs return None (not Unknown), so callers can detect the scenario.
        assert_eq!(Layout::map_apple_id("com.apple.keylayout.German"), None);
        assert_eq!(Layout::map_apple_id(""), None);
    }

    #[test]
    fn detect_option_as_alt_matches_upstream() {
        assert_eq!(Layout::UsStandard.detect_option_as_alt(), OptionAsAlt::True);
        assert_eq!(
            Layout::UsInternational.detect_option_as_alt(),
            OptionAsAlt::True,
        );
        assert_eq!(Layout::Unknown.detect_option_as_alt(), OptionAsAlt::False);
    }

    #[test]
    fn default_layout_is_unknown() {
        assert_eq!(Layout::default(), Layout::Unknown);
    }
}
