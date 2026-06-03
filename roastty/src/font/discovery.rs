//! Font discovery descriptors.
//!
//! Faithful port of the font-search data types from upstream `font/discovery.zig`
//! (and the `Variation` from `font/face.zig`). A [`Descriptor`] describes a font
//! to search for; a [`Variation`] is a font-variation axis setting. The discovery
//! logic that turns a descriptor into a loaded face (CoreText font matching) is a
//! later sub-area.

/// A font-variation axis setting (e.g. weight `wght`, slant `slnt`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Variation {
    /// The axis identifier — a four-character code packed big-endian into a
    /// `u32` (e.g. `wght` is `2003265652`).
    pub id: u32,
    /// The axis value.
    pub value: f64,
}

impl Variation {
    /// Pack a four-character axis tag into its `u32` identifier. Faithful to
    /// upstream's `Variation.Id` (a `wght` tag yields `2003265652`).
    pub(crate) fn id_from_tag(tag: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*tag)
    }
}

/// Describes a font to search for. Faithful port of upstream
/// `discovery.Descriptor` (owned `String`s replace the caller-owned Zig strings).
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Descriptor {
    /// The font family to search for (e.g. `"Fira Code"`, `"monospace"`).
    pub family: Option<String>,
    /// A specific font style string to filter by.
    pub style: Option<String>,
    /// A codepoint the font must be able to render (`0` = none).
    pub codepoint: u32,
    /// The font size in points the font should support (`0.0` = unspecified).
    pub size: f32,
    /// Search for a bold font.
    pub bold: bool,
    /// Search for an italic font.
    pub italic: bool,
    /// Search for a monospace font.
    pub monospace: bool,
    /// Variation axes to apply (preferred when searching).
    pub variations: Vec<Variation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variation_id_from_tag() {
        // Upstream-verified packed identifiers.
        assert_eq!(Variation::id_from_tag(b"wght"), 2003265652);
        assert_eq!(Variation::id_from_tag(b"slnt"), 1936486004);
    }

    #[test]
    fn descriptor_default() {
        let d = Descriptor::default();
        assert_eq!(d.codepoint, 0);
        assert_eq!(d.size, 0.0);
        assert!(!d.bold && !d.italic && !d.monospace);
        assert!(d.variations.is_empty());
        assert!(d.family.is_none() && d.style.is_none());
    }
}
