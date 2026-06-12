//! Width-only fast paths for Unicode codepoints (shaped after upstream
//! `simd/codepoint_width`).
//!
//! The full `unicode::Properties` table remains authoritative for grapheme and
//! emoji metadata, and for uncommon width ranges not covered by these hot paths.

#[inline(always)]
pub(super) fn fast_codepoint_width(codepoint: u32) -> Option<u8> {
    if codepoint <= 0xff {
        return Some(1);
    }

    if codepoint <= 0xffff {
        return fast_codepoint_width_16(codepoint);
    }
    fast_codepoint_width_32(codepoint)
}

#[inline(always)]
fn fast_codepoint_width_16(codepoint: u32) -> Option<u8> {
    if codepoint <= 0x02ff {
        return Some(1);
    }
    if codepoint <= 0x036f {
        return Some(0);
    }
    if codepoint < 0x1100 {
        return None;
    }
    if codepoint <= 0x115e {
        return Some(2);
    }
    if codepoint <= 0x11ff {
        return Some(0);
    }
    if matches!(codepoint, 0x231a..=0x231b | 0x2e3a..=0x2e3b) {
        return Some(2);
    }
    if (0x3400..=0x9fff).contains(&codepoint) {
        return Some(2);
    }
    if (0xf900..=0xfaff).contains(&codepoint) {
        return Some(2);
    }
    None
}

#[inline(always)]
fn fast_codepoint_width_32(codepoint: u32) -> Option<u8> {
    if (0x1f1e6..=0x1f1ff).contains(&codepoint)
        || (0x1f37e..=0x1f393).contains(&codepoint)
        || (0x1f5fb..=0x1f64f).contains(&codepoint)
        || (0x20000..=0x2fffd).contains(&codepoint)
        || (0x30000..=0x3fffd).contains(&codepoint)
    {
        return Some(2);
    }
    if (0xe0020..=0xe007f).contains(&codepoint) {
        return Some(0);
    }
    None
}
