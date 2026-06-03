//! The run iterator — grouping a terminal row's cells into shaping runs.
//!
//! Faithful (in progress) port of upstream `font/shaper/run.zig`. This slice
//! provides the pure decision helpers of `RunIterator.next()`: the bold/italic
//! style mapping, the bad-ligature run split, and the grapheme presentation
//! derivation. The cell-walking `next()` loop (which extracts these values from a
//! terminal `Cell`), `comparableStyle`, the selection/cursor/spacer breaks, and
//! the `TextRun` value type are later sub-areas.

use crate::font::{Presentation, Style};

/// The font [`Style`] for a cell's bold/italic flags. Faithful port of upstream
/// `RunIterator.next()`'s `font_style` derivation (bold-with-italic is
/// bold-italic, not just bold).
pub(crate) fn font_style(bold: bool, italic: bool) -> Style {
    match (bold, italic) {
        (true, true) => Style::BoldItalic,
        (true, false) => Style::Bold,
        (false, true) => Style::Italic,
        (false, false) => Style::Regular,
    }
}

/// Whether a run should split between two adjacent plain codepoints to avoid a
/// commonly-undesirable ligature (`fl`, `fi`, `st`). Directional: `prev_cp`
/// precedes `cp`. Faithful port of upstream `RunIterator.next()`'s bad-ligature
/// break. (The caller applies the `content_tag == codepoint` guard — both cells
/// must be plain codepoints — before calling this.)
pub(crate) fn is_bad_ligature_break(prev_cp: u32, cp: u32) -> bool {
    // `const` bindings so the match arms read as the ASCII letters (a cast
    // expression like `b'f' as u32` is not a valid match pattern).
    const F: u32 = b'f' as u32;
    const L: u32 = b'l' as u32;
    const I: u32 = b'i' as u32;
    const S: u32 = b's' as u32;
    const T: u32 = b't' as u32;
    match prev_cp {
        F => cp == L || cp == I,
        S => cp == T,
        _ => false,
    }
}

/// The explicit presentation a grapheme's first codepoint forces, or `None`. A
/// variation selector `U+FE0E` forces text and `U+FE0F` forces emoji; any other
/// first codepoint leaves the presentation to the font grid's default. Faithful
/// port of upstream `RunIterator.next()`'s grapheme presentation derivation.
pub(crate) fn presentation_for_grapheme(first_cp: u32) -> Option<Presentation> {
    match first_cp {
        0xFE0E => Some(Presentation::Text),
        0xFE0F => Some(Presentation::Emoji),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_style_combinations() {
        assert_eq!(font_style(false, false), Style::Regular);
        assert_eq!(font_style(true, false), Style::Bold);
        assert_eq!(font_style(false, true), Style::Italic);
        assert_eq!(font_style(true, true), Style::BoldItalic);
    }

    #[test]
    fn bad_ligature_breaks() {
        // The undesirable ligatures split the run.
        assert!(is_bad_ligature_break('f' as u32, 'l' as u32));
        assert!(is_bad_ligature_break('f' as u32, 'i' as u32));
        assert!(is_bad_ligature_break('s' as u32, 't' as u32));
        // Everything else does not.
        assert!(!is_bad_ligature_break('f' as u32, 'x' as u32));
        assert!(!is_bad_ligature_break('s' as u32, 'x' as u32));
        assert!(!is_bad_ligature_break('a' as u32, 'b' as u32));
        // Directional: the reverse pair does not break.
        assert!(!is_bad_ligature_break('l' as u32, 'f' as u32));
        assert!(!is_bad_ligature_break('t' as u32, 's' as u32));
    }

    #[test]
    fn presentation_for_grapheme_selectors() {
        assert_eq!(presentation_for_grapheme(0xFE0E), Some(Presentation::Text));
        assert_eq!(presentation_for_grapheme(0xFE0F), Some(Presentation::Emoji));
        assert_eq!(presentation_for_grapheme('a' as u32), None);
        assert_eq!(presentation_for_grapheme(0x200D), None);
    }
}
