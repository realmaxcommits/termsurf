//! The `head` (Font Header) table.
//!
//! Faithful port of upstream `font/opentype/head.zig`. Field names follow the
//! spec (camelCase upstream → Rust `snake_case`).
//!
//! Reference: <https://learn.microsoft.com/en-us/typography/opentype/spec/head>

use super::sfnt::{Fixed, OpenTypeError, Reader};

/// Font Header Table (54 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Head {
    /// Major version number of the font header table — set to 1.
    pub major_version: u16,
    /// Minor version number of the font header table — set to 0.
    pub minor_version: u16,
    /// Set by font manufacturer.
    pub font_revision: Fixed,
    /// Checksum adjustment; ignored when used in a font collection.
    pub checksum_adjustment: u32,
    /// Set to `0x5F0F3CF5`.
    pub magic_number: u32,
    /// Header flags (see spec).
    pub flags: u16,
    /// Units per em — a value from 16 to 16384.
    pub units_per_em: u16,
    /// Seconds since midnight 1904-01-01 GMT.
    pub created: i64,
    /// Seconds since midnight 1904-01-01 GMT.
    pub modified: i64,
    /// Minimum x coordinate across all glyph bounding boxes.
    pub x_min: i16,
    /// Minimum y coordinate across all glyph bounding boxes.
    pub y_min: i16,
    /// Maximum x coordinate across all glyph bounding boxes.
    pub x_max: i16,
    /// Maximum y coordinate across all glyph bounding boxes.
    pub y_max: i16,
    /// Mac style bits (bold/italic/etc.).
    pub mac_style: u16,
    /// Smallest readable size in pixels.
    pub lowest_rec_ppem: u16,
    /// Deprecated font direction hint (set to 2).
    pub font_direction_hint: i16,
    /// 0 for short `loca` offsets, 1 for long.
    pub index_to_loc_format: i16,
    /// 0 for current format.
    pub glyph_data_format: i16,
}

impl Head {
    /// Parse the table from raw `head`-table bytes.
    pub(crate) fn from_bytes(data: &[u8]) -> Result<Head, OpenTypeError> {
        let mut r = Reader::new(data);
        Ok(Head {
            major_version: r.read_u16()?,
            minor_version: r.read_u16()?,
            font_revision: Fixed(r.read_i32()?),
            checksum_adjustment: r.read_u32()?,
            magic_number: r.read_u32()?,
            flags: r.read_u16()?,
            units_per_em: r.read_u16()?,
            created: r.read_i64()?,
            modified: r.read_i64()?,
            x_min: r.read_i16()?,
            y_min: r.read_i16()?,
            x_max: r.read_i16()?,
            y_max: r.read_i16()?,
            mac_style: r.read_u16()?,
            lowest_rec_ppem: r.read_u16()?,
            font_direction_hint: r.read_i16()?,
            index_to_loc_format: r.read_i16()?,
            glyph_data_format: r.read_i16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A hand-built 54-byte `head` table, big-endian, with known field values.
    #[rustfmt::skip]
    const HEAD_BYTES: [u8; 54] = [
        0x00, 0x01,                                     // major_version = 1
        0x00, 0x00,                                     // minor_version = 0
        0x00, 0x01, 0x00, 0x00,                         // font_revision = 1.0 (65536)
        0x12, 0x34, 0x56, 0x78,                         // checksum_adjustment
        0x5F, 0x0F, 0x3C, 0xF5,                         // magic_number
        0x00, 0x07,                                     // flags = 7
        0x08, 0x00,                                     // units_per_em = 2048
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // created = 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // modified = 2
        0xFF, 0xF6,                                     // x_min = -10
        0xFF, 0xEC,                                     // y_min = -20
        0x03, 0xE8,                                     // x_max = 1000
        0x07, 0xD0,                                     // y_max = 2000
        0x00, 0x00,                                     // mac_style = 0
        0x00, 0x07,                                     // lowest_rec_ppem = 7
        0x00, 0x02,                                     // font_direction_hint = 2
        0x00, 0x01,                                     // index_to_loc_format = 1
        0x00, 0x00,                                     // glyph_data_format = 0
    ];

    #[test]
    fn parse_head() {
        let head = Head::from_bytes(&HEAD_BYTES).unwrap();
        assert_eq!(
            head,
            Head {
                major_version: 1,
                minor_version: 0,
                font_revision: Fixed::from_f64(1.0),
                checksum_adjustment: 0x1234_5678,
                magic_number: 0x5F0F_3CF5,
                flags: 7,
                units_per_em: 2048,
                created: 1,
                modified: 2,
                x_min: -10,
                y_min: -20,
                x_max: 1000,
                y_max: 2000,
                mac_style: 0,
                lowest_rec_ppem: 7,
                font_direction_hint: 2,
                index_to_loc_format: 1,
                glyph_data_format: 0,
            }
        );
    }

    #[test]
    fn head_truncated() {
        assert_eq!(
            Head::from_bytes(&HEAD_BYTES[..53]),
            Err(OpenTypeError::EndOfStream)
        );
    }
}
