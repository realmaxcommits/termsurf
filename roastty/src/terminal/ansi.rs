//! ANSI / VT constants (port of upstream `terminal/ansi`).

/// C0 (7-bit) ANSI control characters (upstream `terminal.ansi.C0`). Only the
/// control codes the terminal handles are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum C0 {
    /// Null.
    Nul = 0x00,
    /// Start of heading.
    Soh = 0x01,
    /// Start of text.
    Stx = 0x02,
    /// Enquiry.
    Enq = 0x05,
    /// Bell.
    Bel = 0x07,
    /// Backspace.
    Bs = 0x08,
    /// Horizontal tab.
    Ht = 0x09,
    /// Line feed.
    Lf = 0x0A,
    /// Vertical tab.
    Vt = 0x0B,
    /// Form feed.
    Ff = 0x0C,
    /// Carriage return.
    Cr = 0x0D,
    /// Shift out.
    So = 0x0E,
    /// Shift in.
    Si = 0x0F,
}

impl C0 {
    /// The byte value of this control code.
    pub(crate) fn value(self) -> u8 {
        self as u8
    }

    /// The named C0 control code for a byte, or `None` for an unrecognized byte
    /// (upstream's non-exhaustive `@enumFromInt`: a parser matches the named codes and
    /// treats the rest as "not a recognized C0").
    pub(crate) fn from_byte(byte: u8) -> Option<C0> {
        Some(match byte {
            0x00 => C0::Nul,
            0x01 => C0::Soh,
            0x02 => C0::Stx,
            0x05 => C0::Enq,
            0x07 => C0::Bel,
            0x08 => C0::Bs,
            0x09 => C0::Ht,
            0x0A => C0::Lf,
            0x0B => C0::Vt,
            0x0C => C0::Ff,
            0x0D => C0::Cr,
            0x0E => C0::So,
            0x0F => C0::Si,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c0_round_trips_and_rejects_unknown() {
        // Each named control code round-trips through its byte value.
        for c in [
            C0::Nul,
            C0::Soh,
            C0::Stx,
            C0::Enq,
            C0::Bel,
            C0::Bs,
            C0::Ht,
            C0::Lf,
            C0::Vt,
            C0::Ff,
            C0::Cr,
            C0::So,
            C0::Si,
        ] {
            assert_eq!(C0::from_byte(c.value()), Some(c));
        }

        // Exact byte values.
        assert_eq!(C0::Nul.value(), 0x00);
        assert_eq!(C0::Bel.value(), 0x07);
        assert_eq!(C0::Lf.value(), 0x0A);
        assert_eq!(C0::Cr.value(), 0x0D);
        assert_eq!(C0::Si.value(), 0x0F);

        // Unrecognized bytes are `None` (upstream's non-exhaustive `_`).
        assert_eq!(C0::from_byte(0x03), None); // ETX (not handled)
        assert_eq!(C0::from_byte(0x04), None); // EOT
        assert_eq!(C0::from_byte(0x20), None); // space
        assert_eq!(C0::from_byte(0x7F), None); // DEL
    }
}
