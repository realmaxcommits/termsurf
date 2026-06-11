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
    /// Return the current host keyboard layout.
    pub(crate) fn current() -> Layout {
        #[cfg(test)]
        {
            return CURRENT_LAYOUT_FOR_TEST
                .with(|layout| layout.get())
                .unwrap_or(Layout::Unknown);
        }

        #[cfg(not(test))]
        current_impl()
    }

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

#[cfg(target_os = "macos")]
fn current_impl() -> Layout {
    current_apple_id()
        .as_deref()
        .and_then(Layout::map_apple_id)
        .unwrap_or(Layout::Unknown)
}

#[cfg(not(target_os = "macos"))]
fn current_impl() -> Layout {
    Layout::Unknown
}

#[cfg(target_os = "macos")]
fn current_apple_id() -> Option<String> {
    use libc::c_void;
    use objc2_core_foundation::CFString;

    #[allow(non_upper_case_globals)]
    #[link(name = "Carbon", kind = "framework")]
    unsafe extern "C" {
        static kTISPropertyInputSourceID: *const c_void;

        fn TISCopyCurrentKeyboardLayoutInputSource() -> *mut c_void;
        fn TISGetInputSourceProperty(source: *mut c_void, key: *const c_void) -> *const c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(cf: *mut c_void);
    }

    unsafe {
        let source = TISCopyCurrentKeyboardLayoutInputSource();
        if source.is_null() {
            return None;
        }

        let id = TISGetInputSourceProperty(source, kTISPropertyInputSourceID);
        let id = if id.is_null() {
            None
        } else {
            Some((&*(id.cast::<CFString>())).to_string())
        };
        CFRelease(source);
        id
    }
}

#[cfg(test)]
thread_local! {
    static CURRENT_LAYOUT_FOR_TEST: std::cell::Cell<Option<Layout>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn with_current_layout_for_test<T>(layout: Layout, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<Layout>);

    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT_LAYOUT_FOR_TEST.with(|current| current.set(self.0));
        }
    }

    let previous = CURRENT_LAYOUT_FOR_TEST.with(|current| current.replace(Some(layout)));
    let _restore = Restore(previous);
    f()
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

    #[test]
    fn current_uses_test_override() {
        with_current_layout_for_test(Layout::UsStandard, || {
            assert_eq!(Layout::current(), Layout::UsStandard);
        });
    }
}
