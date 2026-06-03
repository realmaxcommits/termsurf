//! The `hhea` (Horizontal Header) table.
//!
//! Faithful port of upstream `font/opentype/hhea.zig`. Field names follow the
//! spec (camelCase upstream → Rust `snake_case`).
//!
//! Reference: <https://learn.microsoft.com/en-us/typography/opentype/spec/hhea>

use super::sfnt::{OpenTypeError, Reader};

/// Horizontal Header Table (36 bytes). `FWORD`/`UFWORD` fields are signed/
/// unsigned 16-bit font-design-unit values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hhea {
    /// Major version number — set to 1.
    pub major_version: u16,
    /// Minor version number — set to 0.
    pub minor_version: u16,
    /// Typographic ascent (FWORD).
    pub ascender: i16,
    /// Typographic descent (FWORD).
    pub descender: i16,
    /// Typographic line gap (FWORD).
    pub line_gap: i16,
    /// Maximum advance width value in `hmtx` (UFWORD).
    pub advance_width_max: u16,
    /// Minimum left sidebearing value in `hmtx` (FWORD).
    pub min_left_side_bearing: i16,
    /// Minimum right sidebearing value (FWORD).
    pub min_right_side_bearing: i16,
    /// `max(lsb + (xMax - xMin))` (FWORD).
    pub x_max_extent: i16,
    /// Caret slope rise (1 for vertical).
    pub caret_slope_rise: i16,
    /// Caret slope run (0 for vertical).
    pub caret_slope_run: i16,
    /// Caret offset.
    pub caret_offset: i16,
    /// Reserved, set to 0.
    pub _reserved0: i16,
    /// Reserved, set to 0.
    pub _reserved1: i16,
    /// Reserved, set to 0.
    pub _reserved2: i16,
    /// Reserved, set to 0.
    pub _reserved3: i16,
    /// 0 for current format.
    pub metric_data_format: i16,
    /// Number of `hMetric` entries in the `hmtx` table.
    pub number_of_h_metrics: u16,
}

impl Hhea {
    /// Parse the table from raw `hhea`-table bytes.
    pub(crate) fn from_bytes(data: &[u8]) -> Result<Hhea, OpenTypeError> {
        let mut r = Reader::new(data);
        Ok(Hhea {
            major_version: r.read_u16()?,
            minor_version: r.read_u16()?,
            ascender: r.read_i16()?,
            descender: r.read_i16()?,
            line_gap: r.read_i16()?,
            advance_width_max: r.read_u16()?,
            min_left_side_bearing: r.read_i16()?,
            min_right_side_bearing: r.read_i16()?,
            x_max_extent: r.read_i16()?,
            caret_slope_rise: r.read_i16()?,
            caret_slope_run: r.read_i16()?,
            caret_offset: r.read_i16()?,
            _reserved0: r.read_i16()?,
            _reserved1: r.read_i16()?,
            _reserved2: r.read_i16()?,
            _reserved3: r.read_i16()?,
            metric_data_format: r.read_i16()?,
            number_of_h_metrics: r.read_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A hand-built 36-byte `hhea` table, big-endian, with known field values.
    #[rustfmt::skip]
    const HHEA_BYTES: [u8; 36] = [
        0x00, 0x01, // major_version = 1
        0x00, 0x00, // minor_version = 0
        0x07, 0x6C, // ascender = 1900
        0xFE, 0x3E, // descender = -450
        0x00, 0x00, // line_gap = 0
        0x04, 0xB0, // advance_width_max = 1200
        0xFC, 0x18, // min_left_side_bearing = -1000
        0xF8, 0x9F, // min_right_side_bearing = -1889
        0x0C, 0x11, // x_max_extent = 3089
        0x00, 0x01, // caret_slope_rise = 1
        0x00, 0x00, // caret_slope_run = 0
        0x00, 0x00, // caret_offset = 0
        0x00, 0x00, // _reserved0 = 0
        0x00, 0x00, // _reserved1 = 0
        0x00, 0x00, // _reserved2 = 0
        0x00, 0x00, // _reserved3 = 0
        0x00, 0x00, // metric_data_format = 0
        0x00, 0x02, // number_of_h_metrics = 2
    ];

    #[test]
    fn parse_hhea() {
        let hhea = Hhea::from_bytes(&HHEA_BYTES).unwrap();
        assert_eq!(
            hhea,
            Hhea {
                major_version: 1,
                minor_version: 0,
                ascender: 1900,
                descender: -450,
                line_gap: 0,
                advance_width_max: 1200,
                min_left_side_bearing: -1000,
                min_right_side_bearing: -1889,
                x_max_extent: 3089,
                caret_slope_rise: 1,
                caret_slope_run: 0,
                caret_offset: 0,
                _reserved0: 0,
                _reserved1: 0,
                _reserved2: 0,
                _reserved3: 0,
                metric_data_format: 0,
                number_of_h_metrics: 2,
            }
        );
    }

    #[test]
    fn hhea_truncated() {
        assert_eq!(
            Hhea::from_bytes(&HHEA_BYTES[..35]),
            Err(OpenTypeError::EndOfStream)
        );
    }
}
