//! Unicode codepoint-range parsing (port of upstream
//! `Config.RepeatableCodepointMap.UnicodeRangeParser`).
//!
//! Walks a key string like `U+1234-U+5678, U+9ABC` yielding `[start, end]`
//! codepoint ranges. The surrounding `RepeatableCodepointMap` (which needs the font
//! codepoint-map storage) is ported later.
#![allow(dead_code)]

/// A failure parsing a Unicode range (upstream `error.InvalidValue`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InvalidRange;

pub(crate) struct UnicodeRangeParser<'a> {
    input: &'a [u8],
    i: usize,
}

impl<'a> UnicodeRangeParser<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        UnicodeRangeParser { input, i: 0 }
    }

    /// Yield the next `[start, end]` range, `None` at end, or `InvalidRange`.
    pub(crate) fn next(&mut self) -> Result<Option<[u32; 2]>, InvalidRange> {
        if self.eof() {
            return Ok(None);
        }
        let start = self.parse_codepoint()?;
        if self.eof() {
            return Ok(Some([start, start]));
        }
        self.consume_whitespace();
        match self.byte() {
            b',' => {
                self.advance();
                self.consume_whitespace();
                if self.eof() {
                    return Err(InvalidRange); // trailing comma
                }
                Ok(Some([start, start]))
            }
            b'-' => {
                self.advance();
                self.consume_whitespace();
                if self.eof() {
                    return Err(InvalidRange);
                }
                let end = self.parse_codepoint()?;
                self.consume_whitespace();
                if !self.eof() && self.byte() != b',' {
                    return Err(InvalidRange);
                }
                self.advance();
                self.consume_whitespace();
                if start > end {
                    return Err(InvalidRange);
                }
                Ok(Some([start, end]))
            }
            _ => Err(InvalidRange),
        }
    }

    fn consume_whitespace(&mut self) {
        while !self.eof() {
            match self.byte() {
                b' ' | b'\t' => self.advance(),
                _ => return,
            }
        }
    }

    fn parse_codepoint(&mut self) -> Result<u32, InvalidRange> {
        if self.eof() || self.byte() != b'U' {
            return Err(InvalidRange);
        }
        self.advance();
        if self.eof() || self.byte() != b'+' {
            return Err(InvalidRange);
        }
        self.advance();
        if self.eof() {
            return Err(InvalidRange);
        }

        let start_i = self.i;
        loop {
            if !self.byte().is_ascii_hexdigit() {
                break;
            }
            self.advance();
            if self.eof() {
                break;
            }
        }
        if start_i == self.i {
            return Err(InvalidRange);
        }
        parse_hex_u21(&self.input[start_i..self.i]).ok_or(InvalidRange)
    }

    fn byte(&self) -> u8 {
        self.input[self.i]
    }

    fn advance(&mut self) {
        self.i += 1;
    }

    fn eof(&self) -> bool {
        self.i >= self.input.len()
    }
}

/// Parse a hex run as a `u21`-range codepoint (upstream `parseInt(u21, _, 16)`):
/// each digit accumulates, and a value exceeding the `u21` max (`0x1FFFFF`) is an
/// overflow. The bytes are already known to be hex.
fn parse_hex_u21(bytes: &[u8]) -> Option<u32> {
    let mut value: u32 = 0;
    for &c in bytes {
        let digit = (c as char).to_digit(16)?;
        // Checked arithmetic so a long hex run can never wrap/panic; the
        // `> 0x1FFFFF` check then enforces the `u21` bound.
        value = value.checked_mul(16)?.checked_add(digit)?;
        if value > 0x1FFFFF {
            return None; // exceeds u21
        }
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges(s: &str) -> Result<Vec<[u32; 2]>, InvalidRange> {
        let mut parser = UnicodeRangeParser::new(s.as_bytes());
        let mut out = Vec::new();
        while let Some(range) = parser.next()? {
            out.push(range);
        }
        Ok(out)
    }

    #[test]
    fn unicode_range_parser_singles_ranges_and_lists() {
        // Upstream `RepeatableCodepointMap.parseCLI` keys.
        assert_eq!(ranges("U+ABCD"), Ok(vec![[0xABCD, 0xABCD]]));
        assert_eq!(ranges("U+0001 - U+0005"), Ok(vec![[1, 5]]));
        assert_eq!(
            ranges("U+0006-U+0009, U+ABCD"),
            Ok(vec![[6, 9], [0xABCD, 0xABCD]])
        );

        // A comma list of single codepoints, and the empty input.
        assert_eq!(
            ranges("U+1234,U+5678"),
            Ok(vec![[0x1234, 0x1234], [0x5678, 0x5678]])
        );
        assert_eq!(ranges(""), Ok(vec![]));

        // Lowercase hex parses the same as uppercase.
        assert_eq!(ranges("U+abcd"), Ok(vec![[0xABCD, 0xABCD]]));
    }

    #[test]
    fn unicode_range_parser_errors() {
        assert_eq!(ranges("U+1234,"), Err(InvalidRange)); // trailing comma
        assert_eq!(ranges("X+1"), Err(InvalidRange)); // no `U`
        assert_eq!(ranges("U1"), Err(InvalidRange)); // no `+`
        assert_eq!(ranges("U+"), Err(InvalidRange)); // no hex
        assert_eq!(ranges("U+GG"), Err(InvalidRange)); // non-hex
        assert_eq!(ranges("U+5-U+1"), Err(InvalidRange)); // start > end
        assert_eq!(ranges("U+1-2"), Err(InvalidRange)); // range end not `U+`
        assert_eq!(ranges("U+200000"), Err(InvalidRange)); // exceeds u21
        assert_eq!(ranges("U+FFFFFFFFFFFFFFFF"), Err(InvalidRange)); // long overflow
    }
}
