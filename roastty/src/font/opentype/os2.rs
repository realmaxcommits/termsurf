//! The `OS/2` (OS/2 and Windows Metrics) table.
//!
//! Faithful port of upstream `font/opentype/os2.zig`. Upstream defines four
//! per-version `extern struct`s and a unified `OS2` with optional trailing
//! fields; this port uses a single version-gated [`Os2::from_bytes`] that reads
//! the same bytes in the same order and yields the same field values and the
//! same per-version `Option` presence. Field names follow the spec (camelCase
//! upstream → Rust `snake_case`).
//!
//! Reference: <https://learn.microsoft.com/en-us/typography/opentype/spec/os2>

use super::sfnt::{OpenTypeError, Reader};

/// The `fsSelection` bitfield (`u16`). Upstream is `packed struct(u16)` filling
/// from the least-significant bit, so `italic` is bit 0 and `use_typo_metrics`
/// is bit 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FsSelection(pub u16);

impl FsSelection {
    fn bit(self, n: u16) -> bool {
        self.0 & (1 << n) != 0
    }

    pub(crate) fn italic(self) -> bool {
        self.bit(0)
    }
    pub(crate) fn underscore(self) -> bool {
        self.bit(1)
    }
    pub(crate) fn negative(self) -> bool {
        self.bit(2)
    }
    pub(crate) fn outlined(self) -> bool {
        self.bit(3)
    }
    pub(crate) fn strikeout(self) -> bool {
        self.bit(4)
    }
    pub(crate) fn bold(self) -> bool {
        self.bit(5)
    }
    pub(crate) fn regular(self) -> bool {
        self.bit(6)
    }
    pub(crate) fn use_typo_metrics(self) -> bool {
        self.bit(7)
    }
    pub(crate) fn wws(self) -> bool {
        self.bit(8)
    }
    pub(crate) fn oblique(self) -> bool {
        self.bit(9)
    }
}

/// OS/2 and Windows Metrics Table. Trailing fields absent in older versions are
/// `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Os2 {
    pub version: u16,
    pub x_avg_char_width: i16,
    pub us_weight_class: u16,
    pub us_width_class: u16,
    pub fs_type: u16,
    pub y_subscript_x_size: i16,
    pub y_subscript_y_size: i16,
    pub y_subscript_x_offset: i16,
    pub y_subscript_y_offset: i16,
    pub y_superscript_x_size: i16,
    pub y_superscript_y_size: i16,
    pub y_superscript_x_offset: i16,
    pub y_superscript_y_offset: i16,
    pub y_strikeout_size: i16,
    pub y_strikeout_position: i16,
    pub s_family_class: i16,
    pub panose: [u8; 10],
    pub ul_unicode_range1: u32,
    pub ul_unicode_range2: u32,
    pub ul_unicode_range3: u32,
    pub ul_unicode_range4: u32,
    pub ach_vend_id: [u8; 4],
    pub fs_selection: FsSelection,
    pub us_first_char_index: u16,
    pub us_last_char_index: u16,
    pub s_typo_ascender: i16,
    pub s_typo_descender: i16,
    pub s_typo_line_gap: i16,
    pub us_win_ascent: u16,
    pub us_win_descent: u16,

    // v1+
    pub ul_code_page_range1: Option<u32>,
    pub ul_code_page_range2: Option<u32>,
    // v2+
    pub sx_height: Option<i16>,
    pub s_cap_height: Option<i16>,
    pub us_default_char: Option<u16>,
    pub us_break_char: Option<u16>,
    pub us_max_context: Option<u16>,
    // v5+
    pub us_lower_optical_point_size: Option<u16>,
    pub us_upper_optical_point_size: Option<u16>,
}

impl Os2 {
    /// Parse the table from raw `OS/2`-table bytes. Versions 0–5 are supported;
    /// a higher version is [`OpenTypeError::UnsupportedVersion`].
    pub(crate) fn from_bytes(data: &[u8]) -> Result<Os2, OpenTypeError> {
        let mut r = Reader::new(data);

        let version = r.read_u16()?;
        if version > 5 {
            return Err(OpenTypeError::UnsupportedVersion);
        }

        // v0 common block, in spec order.
        let x_avg_char_width = r.read_i16()?;
        let us_weight_class = r.read_u16()?;
        let us_width_class = r.read_u16()?;
        let fs_type = r.read_u16()?;
        let y_subscript_x_size = r.read_i16()?;
        let y_subscript_y_size = r.read_i16()?;
        let y_subscript_x_offset = r.read_i16()?;
        let y_subscript_y_offset = r.read_i16()?;
        let y_superscript_x_size = r.read_i16()?;
        let y_superscript_y_size = r.read_i16()?;
        let y_superscript_x_offset = r.read_i16()?;
        let y_superscript_y_offset = r.read_i16()?;
        let y_strikeout_size = r.read_i16()?;
        let y_strikeout_position = r.read_i16()?;
        let s_family_class = r.read_i16()?;
        let panose = r.read_bytes::<10>()?;
        let ul_unicode_range1 = r.read_u32()?;
        let ul_unicode_range2 = r.read_u32()?;
        let ul_unicode_range3 = r.read_u32()?;
        let ul_unicode_range4 = r.read_u32()?;
        let ach_vend_id = r.read_bytes::<4>()?;
        let fs_selection = FsSelection(r.read_u16()?);
        let us_first_char_index = r.read_u16()?;
        let us_last_char_index = r.read_u16()?;
        let s_typo_ascender = r.read_i16()?;
        let s_typo_descender = r.read_i16()?;
        let s_typo_line_gap = r.read_i16()?;
        let us_win_ascent = r.read_u16()?;
        let us_win_descent = r.read_u16()?;

        // v1+: code page ranges.
        let (ul_code_page_range1, ul_code_page_range2) = if version >= 1 {
            (Some(r.read_u32()?), Some(r.read_u32()?))
        } else {
            (None, None)
        };

        // v2+: x/cap heights and default/break/max-context.
        let (sx_height, s_cap_height, us_default_char, us_break_char, us_max_context) =
            if version >= 2 {
                (
                    Some(r.read_i16()?),
                    Some(r.read_i16()?),
                    Some(r.read_u16()?),
                    Some(r.read_u16()?),
                    Some(r.read_u16()?),
                )
            } else {
                (None, None, None, None, None)
            };

        // v5+: optical point sizes.
        let (us_lower_optical_point_size, us_upper_optical_point_size) = if version >= 5 {
            (Some(r.read_u16()?), Some(r.read_u16()?))
        } else {
            (None, None)
        };

        Ok(Os2 {
            version,
            x_avg_char_width,
            us_weight_class,
            us_width_class,
            fs_type,
            y_subscript_x_size,
            y_subscript_y_size,
            y_subscript_x_offset,
            y_subscript_y_offset,
            y_superscript_x_size,
            y_superscript_y_size,
            y_superscript_x_offset,
            y_superscript_y_offset,
            y_strikeout_size,
            y_strikeout_position,
            s_family_class,
            panose,
            ul_unicode_range1,
            ul_unicode_range2,
            ul_unicode_range3,
            ul_unicode_range4,
            ach_vend_id,
            fs_selection,
            us_first_char_index,
            us_last_char_index,
            s_typo_ascender,
            s_typo_descender,
            s_typo_line_gap,
            us_win_ascent,
            us_win_descent,
            ul_code_page_range1,
            ul_code_page_range2,
            sx_height,
            s_cap_height,
            us_default_char,
            us_break_char,
            us_max_context,
            us_lower_optical_point_size,
            us_upper_optical_point_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(v: &mut Vec<u8>, x: u16) {
        v.extend_from_slice(&x.to_be_bytes());
    }
    fn push_i16(v: &mut Vec<u8>, x: i16) {
        v.extend_from_slice(&x.to_be_bytes());
    }
    fn push_u32(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_be_bytes());
    }

    const PANOSE: [u8; 10] = [2, 0, 5, 3, 0, 0, 0, 0, 0, 0];
    const VEND: [u8; 4] = *b"TEST";
    // regular (bit 6) + use_typo_metrics (bit 7).
    const FS: u16 = (1 << 6) | (1 << 7);

    /// Build an `OS/2` table for the given version with known field values,
    /// appending the version-gated trailing blocks.
    fn build(version: u16) -> Vec<u8> {
        let mut d = Vec::new();
        push_u16(&mut d, version);
        push_i16(&mut d, 500); // x_avg_char_width
        push_u16(&mut d, 400); // us_weight_class
        push_u16(&mut d, 5); // us_width_class
        push_u16(&mut d, 0); // fs_type
        push_i16(&mut d, 650); // y_subscript_x_size
        push_i16(&mut d, 600); // y_subscript_y_size
        push_i16(&mut d, 0); // y_subscript_x_offset
        push_i16(&mut d, 75); // y_subscript_y_offset
        push_i16(&mut d, 650); // y_superscript_x_size
        push_i16(&mut d, 600); // y_superscript_y_size
        push_i16(&mut d, 0); // y_superscript_x_offset
        push_i16(&mut d, 350); // y_superscript_y_offset
        push_i16(&mut d, 50); // y_strikeout_size
        push_i16(&mut d, 250); // y_strikeout_position
        push_i16(&mut d, 0); // s_family_class
        d.extend_from_slice(&PANOSE); // panose
        push_u32(&mut d, 0xE000_02FF); // ul_unicode_range1
        push_u32(&mut d, 0); // ul_unicode_range2
        push_u32(&mut d, 0); // ul_unicode_range3
        push_u32(&mut d, 0); // ul_unicode_range4
        d.extend_from_slice(&VEND); // ach_vend_id
        push_u16(&mut d, FS); // fs_selection
        push_u16(&mut d, 32); // us_first_char_index
        push_u16(&mut d, 0xFFFF); // us_last_char_index
        push_i16(&mut d, 1500); // s_typo_ascender
        push_i16(&mut d, -500); // s_typo_descender
        push_i16(&mut d, 0); // s_typo_line_gap
        push_u16(&mut d, 1900); // us_win_ascent
        push_u16(&mut d, 500); // us_win_descent
        assert_eq!(d.len(), 78, "v0 common block must be 78 bytes");

        if version >= 1 {
            push_u32(&mut d, 0x0000_0001); // ul_code_page_range1
            push_u32(&mut d, 0); // ul_code_page_range2
        }
        if version >= 2 {
            push_i16(&mut d, 1100); // sx_height
            push_i16(&mut d, 1400); // s_cap_height
            push_u16(&mut d, 0); // us_default_char
            push_u16(&mut d, 32); // us_break_char
            push_u16(&mut d, 1); // us_max_context
        }
        if version >= 5 {
            push_u16(&mut d, 6); // us_lower_optical_point_size
            push_u16(&mut d, 0xFFFF); // us_upper_optical_point_size
        }
        d
    }

    #[test]
    fn fs_selection_bits() {
        let typo = FsSelection(1 << 7);
        assert!(typo.use_typo_metrics());
        assert!(!typo.italic());
        assert!(!typo.regular());

        assert!(FsSelection(1 << 0).italic());
        assert!(FsSelection(1 << 9).oblique());

        let combined = FsSelection((1 << 6) | (1 << 7));
        assert!(combined.regular());
        assert!(combined.use_typo_metrics());
        assert!(!combined.bold());
    }

    fn assert_common(os2: &Os2, version: u16) {
        assert_eq!(os2.version, version);
        assert_eq!(os2.x_avg_char_width, 500);
        assert_eq!(os2.us_weight_class, 400);
        assert_eq!(os2.panose, PANOSE);
        assert_eq!(os2.ul_unicode_range1, 0xE000_02FF);
        assert_eq!(os2.ach_vend_id, VEND);
        assert_eq!(os2.fs_selection, FsSelection(FS));
        assert!(os2.fs_selection.use_typo_metrics());
        assert_eq!(os2.y_strikeout_size, 50);
        assert_eq!(os2.y_strikeout_position, 250);
        assert_eq!(os2.s_typo_ascender, 1500);
        assert_eq!(os2.s_typo_descender, -500);
        assert_eq!(os2.s_typo_line_gap, 0);
        assert_eq!(os2.us_win_ascent, 1900);
        assert_eq!(os2.us_win_descent, 500);
    }

    #[test]
    fn parse_os2_v4() {
        let os2 = Os2::from_bytes(&build(4)).unwrap();
        assert_common(&os2, 4);
        assert_eq!(os2.ul_code_page_range1, Some(0x0000_0001));
        assert_eq!(os2.ul_code_page_range2, Some(0));
        assert_eq!(os2.sx_height, Some(1100));
        assert_eq!(os2.s_cap_height, Some(1400));
        assert_eq!(os2.us_max_context, Some(1));
        // v5 optical sizes absent in v4.
        assert_eq!(os2.us_lower_optical_point_size, None);
        assert_eq!(os2.us_upper_optical_point_size, None);
    }

    #[test]
    fn parse_os2_v0() {
        let os2 = Os2::from_bytes(&build(0)).unwrap();
        assert_common(&os2, 0);
        assert_eq!(os2.ul_code_page_range1, None);
        assert_eq!(os2.sx_height, None);
        assert_eq!(os2.s_cap_height, None);
        assert_eq!(os2.us_max_context, None);
        assert_eq!(os2.us_upper_optical_point_size, None);
    }

    #[test]
    fn parse_os2_v5() {
        let os2 = Os2::from_bytes(&build(5)).unwrap();
        assert_common(&os2, 5);
        assert_eq!(os2.sx_height, Some(1100));
        assert_eq!(os2.us_lower_optical_point_size, Some(6));
        assert_eq!(os2.us_upper_optical_point_size, Some(0xFFFF));
    }

    #[test]
    fn os2_unsupported_version() {
        let mut d = build(5);
        // Overwrite the version field (first u16) with 6.
        d[0] = 0;
        d[1] = 6;
        assert_eq!(Os2::from_bytes(&d), Err(OpenTypeError::UnsupportedVersion));
    }

    #[test]
    fn os2_truncated() {
        // A v4 table cut short inside the trailing block.
        let d = build(4);
        assert_eq!(Os2::from_bytes(&d[..80]), Err(OpenTypeError::EndOfStream));
    }
}
