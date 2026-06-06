//! The `glyf` (Glyph Data) table.
//!
//! Faithful port of upstream `font/opentype/glyf.zig`'s narrow validator. This
//! stores borrowed table/entry bytes, identifies simple vs. composite glyphs,
//! and computes the consumed byte size of simple glyph entries. Composite glyphs
//! and hinted glyphs are intentionally rejected, matching upstream's current
//! glyph-protocol validation behavior.
//!
//! Reference: <https://learn.microsoft.com/en-us/typography/opentype/spec/glyf>

use super::sfnt::{OpenTypeError, Reader};

const HEADER_SIZE: usize = 10;

/// Glyph Data Table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Glyf<'a> {
    data: &'a [u8],
}

impl<'a> Glyf<'a> {
    /// Borrow the raw `glyf` table bytes.
    pub(crate) fn from_bytes(data: &'a [u8]) -> Glyf<'a> {
        Glyf { data }
    }

    /// Retrieve the entry at byte `offset` in the table.
    pub(crate) fn entry(&self, offset: usize) -> Result<Entry<'a>, OpenTypeError> {
        let data = self.data.get(offset..).ok_or(OpenTypeError::EndOfStream)?;
        Entry::from_bytes(data)
    }
}

/// A single glyph entry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Entry<'a> {
    pub header: Header,
    /// Bytes immediately following the 10-byte glyph header.
    data: &'a [u8],
}

impl<'a> Entry<'a> {
    /// Borrow a glyph entry from its raw bytes.
    pub(crate) fn from_bytes(data: &'a [u8]) -> Result<Entry<'a>, OpenTypeError> {
        let mut r = Reader::new(data);
        let header = Header {
            number_of_contours: r.read_i16()?,
            x_min: r.read_i16()?,
            y_min: r.read_i16()?,
            x_max: r.read_i16()?,
            y_max: r.read_i16()?,
        };
        Ok(Entry {
            header,
            data: &data[r.pos()..],
        })
    }

    /// Whether this is a simple or composite glyph.
    pub(crate) fn entry_type(&self) -> EntryType {
        if self.header.number_of_contours >= 0 {
            EntryType::Simple
        } else {
            EntryType::Composite
        }
    }

    /// Determine the consumed byte size of this entry, including the header.
    pub(crate) fn size(&self) -> Result<usize, SizeError> {
        let EntryType::Simple = self.entry_type() else {
            return Err(SizeError::CompositeNotSupported);
        };

        let num_contours = self.header.number_of_contours as usize;
        if num_contours == 0 && self.data.len() < 2 {
            return Ok(HEADER_SIZE);
        }

        let mut r = Reader::new(self.data);
        let mut max_point_index: isize = -1;
        for _ in 0..num_contours {
            let index = r.read_u16()? as isize;
            if index <= max_point_index {
                return Err(SizeError::EndPointsOutOfOrder);
            }
            max_point_index = index;
        }

        let instructions_len = r.read_u16()?;
        if instructions_len > 0 {
            return Err(SizeError::InstructionsNotSupported);
        }

        let mut x_coords_len = 0usize;
        let mut y_coords_len = 0usize;
        if max_point_index >= 0 {
            let mut point = 0usize;
            let max_point_index = max_point_index as usize;
            while point <= max_point_index {
                let flag = SimpleFlags::from_byte(r.read_u8()?);
                x_coords_len += flag.x_bytes() as usize;
                y_coords_len += flag.y_bytes() as usize;

                if flag.repeat {
                    let repeat_count = r.read_u8()? as usize;
                    point += repeat_count;
                    x_coords_len += repeat_count * flag.x_bytes() as usize;
                    y_coords_len += repeat_count * flag.y_bytes() as usize;
                    if point > max_point_index {
                        return Err(SizeError::TooManyPoints);
                    }
                }

                point += 1;
            }
        }

        r.skip(x_coords_len)?;
        r.skip(y_coords_len)?;
        Ok(HEADER_SIZE + r.pos())
    }
}

/// The header at the start of every glyph entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header {
    pub number_of_contours: i16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

/// Glyph entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryType {
    Simple,
    Composite,
}

/// Simple glyph flag byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SimpleFlags {
    pub on_curve: bool,
    pub x_short: bool,
    pub y_short: bool,
    pub repeat: bool,
    pub x_repeat_or_sign: bool,
    pub y_repeat_or_sign: bool,
    pub overlap: bool,
    pub reserved: bool,
}

impl SimpleFlags {
    fn from_byte(byte: u8) -> SimpleFlags {
        SimpleFlags {
            on_curve: byte & (1 << 0) != 0,
            x_short: byte & (1 << 1) != 0,
            y_short: byte & (1 << 2) != 0,
            repeat: byte & (1 << 3) != 0,
            x_repeat_or_sign: byte & (1 << 4) != 0,
            y_repeat_or_sign: byte & (1 << 5) != 0,
            overlap: byte & (1 << 6) != 0,
            reserved: byte & (1 << 7) != 0,
        }
    }

    fn x_bytes(self) -> u8 {
        if self.x_short {
            1
        } else if self.x_repeat_or_sign {
            0
        } else {
            2
        }
    }

    fn y_bytes(self) -> u8 {
        if self.y_short {
            1
        } else if self.y_repeat_or_sign {
            0
        } else {
            2
        }
    }
}

/// Errors returned by [`Entry::size`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SizeError {
    EndOfStream,
    InstructionsNotSupported,
    CompositeNotSupported,
    EndPointsOutOfOrder,
    TooManyPoints,
}

impl From<OpenTypeError> for SizeError {
    fn from(err: OpenTypeError) -> SizeError {
        match err {
            OpenTypeError::EndOfStream => SizeError::EndOfStream,
            OpenTypeError::UnsupportedVersion => unreachable!("glyf has no table version"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(number_of_contours: i16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&number_of_contours.to_be_bytes());
        bytes.extend_from_slice(&(-1i16).to_be_bytes());
        bytes.extend_from_slice(&(-2i16).to_be_bytes());
        bytes.extend_from_slice(&(10i16).to_be_bytes());
        bytes.extend_from_slice(&(20i16).to_be_bytes());
        bytes
    }

    fn simple_one_contour(end: u16, instructions_len: u16, tail: &[u8]) -> Vec<u8> {
        let mut bytes = header(1);
        bytes.extend_from_slice(&end.to_be_bytes());
        bytes.extend_from_slice(&instructions_len.to_be_bytes());
        bytes.extend_from_slice(tail);
        bytes
    }

    fn simple_mixed_flags() -> Vec<u8> {
        let mut bytes = simple_one_contour(
            3,
            0,
            &[
                0x37, // point 0: x/y short positive, y same
                0x11, // point 1: x same, y 16-bit
                0x27, // point 2: x 16-bit, y short positive
                0x3B, // point 3: repeat flag, x short positive, y same
                0x00, // repeat count = 0
                0x05, // x coordinate for point 0
                0x01, // x coordinate for point 2, high byte
                0x02, // x coordinate for point 2, low byte
                0x07, // x coordinate for point 3
                0x00, // y coordinate for point 1, high byte
                0x03, // y coordinate for point 1, low byte
                0x09, // y coordinate for point 2
            ],
        );
        bytes.extend_from_slice(&[0xAA, 0xBB]); // legal trailing bytes.
        bytes
    }

    #[test]
    fn simple_flags_bit_mapping_and_coordinate_sizes() {
        let flags = SimpleFlags::from_byte(0xFF);
        assert!(flags.on_curve);
        assert!(flags.x_short);
        assert!(flags.y_short);
        assert!(flags.repeat);
        assert!(flags.x_repeat_or_sign);
        assert!(flags.y_repeat_or_sign);
        assert!(flags.overlap);
        assert!(flags.reserved);

        assert_eq!(SimpleFlags::from_byte(0x00).x_bytes(), 2);
        assert_eq!(SimpleFlags::from_byte(0x10).x_bytes(), 0);
        assert_eq!(SimpleFlags::from_byte(0x02).x_bytes(), 1);
        assert_eq!(SimpleFlags::from_byte(0x00).y_bytes(), 2);
        assert_eq!(SimpleFlags::from_byte(0x20).y_bytes(), 0);
        assert_eq!(SimpleFlags::from_byte(0x04).y_bytes(), 1);
    }

    #[test]
    fn entry_from_bytes_parses_header_and_entry_type() {
        let simple_bytes = header(2);
        let entry = Entry::from_bytes(&simple_bytes).unwrap();
        assert_eq!(entry.header.number_of_contours, 2);
        assert_eq!(entry.header.x_min, -1);
        assert_eq!(entry.header.y_min, -2);
        assert_eq!(entry.header.x_max, 10);
        assert_eq!(entry.header.y_max, 20);
        assert_eq!(entry.entry_type(), EntryType::Simple);

        let composite_bytes = header(-1);
        let composite = Entry::from_bytes(&composite_bytes).unwrap();
        assert_eq!(composite.entry_type(), EntryType::Composite);
    }

    #[test]
    fn glyf_entry_slices_from_offset() {
        let mut table = vec![0xAA, 0xBB, 0xCC];
        table.extend_from_slice(&header(0));
        let glyf = Glyf::from_bytes(&table);

        let entry = glyf.entry(3).unwrap();

        assert_eq!(entry.header.number_of_contours, 0);
        assert_eq!(entry.size(), Ok(HEADER_SIZE));
    }

    #[test]
    fn simple_size_counts_flags_coordinates_and_allows_trailing_bytes() {
        let bytes = simple_mixed_flags();
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Ok(26));
    }

    #[test]
    fn zero_contour_header_only_size() {
        let bytes = header(0);
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Ok(HEADER_SIZE));
    }

    #[test]
    fn zero_contour_with_empty_instruction_length_size() {
        let mut bytes = header(0);
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&[0xAA, 0xBB]);
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Ok(HEADER_SIZE + 2));
    }

    #[test]
    fn rejects_composite_glyphs() {
        let bytes = header(-1);
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Err(SizeError::CompositeNotSupported));
    }

    #[test]
    fn rejects_hinted_glyphs() {
        let bytes = simple_one_contour(0, 1, &[0x00]);
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Err(SizeError::InstructionsNotSupported));
    }

    #[test]
    fn rejects_truncated_header_and_data() {
        assert!(matches!(
            Entry::from_bytes(&[0u8; 9]),
            Err(OpenTypeError::EndOfStream)
        ));

        let bytes = simple_one_contour(0, 0, &[0x00, 0x00]);
        let entry = Entry::from_bytes(&bytes).unwrap();
        assert_eq!(entry.size(), Err(SizeError::EndOfStream));
    }

    #[test]
    fn rejects_endpoints_out_of_order() {
        let mut bytes = header(2);
        bytes.extend_from_slice(&1u16.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Err(SizeError::EndPointsOutOfOrder));
    }

    #[test]
    fn rejects_too_many_points_from_repeat() {
        let bytes = simple_one_contour(0, 0, &[0x08, 0x01]);
        let entry = Entry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.size(), Err(SizeError::TooManyPoints));
    }
}
