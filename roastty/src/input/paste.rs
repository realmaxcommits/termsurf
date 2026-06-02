//! Paste safety checks and xterm-compatible paste encoding.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PasteOptions {
    pub(crate) bracketed: bool,
}

const BRACKETED_START: &[u8] = b"\x1b[200~";
const BRACKETED_END: &[u8] = b"\x1b[201~";
const UNSAFE_BRACKETED_END: &[u8] = b"\x1b[201~";

const STRIP_BYTES: &[u8] = &[
    0x00, // NUL
    0x08, // BS
    0x05, // ENQ
    0x04, // EOT
    0x1b, // ESC
    0x7f, // DEL
    0x03, // VINTR
    0x1c, // VQUIT
    0x15, // VKILL
    0x1a, // VSUSP
    0x11, // VSTART
    0x13, // VSTOP
    0x17, // VWERASE
    0x16, // VLNEXT
    0x12, // VREPRINT
    0x0f, // VDISCARD
];

pub(crate) fn is_safe(data: &[u8]) -> bool {
    !data.contains(&b'\n')
        && data
            .windows(UNSAFE_BRACKETED_END.len())
            .all(|window| window != UNSAFE_BRACKETED_END)
}

pub(crate) fn encode(data: &mut [u8], options: PasteOptions) -> [&[u8]; 3] {
    for byte in data.iter_mut() {
        if STRIP_BYTES.contains(byte) {
            *byte = b' ';
        }
    }

    if options.bracketed {
        return [BRACKETED_START, data, BRACKETED_END];
    }

    for byte in data.iter_mut() {
        if *byte == b'\n' {
            *byte = b'\r';
        }
    }

    [b"", data, b""]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_is_safe_matches_upstream_cases() {
        assert!(is_safe(b"hello"));
        assert!(is_safe(b""));
        assert!(!is_safe(b"hello\n"));
        assert!(!is_safe(b"hello\nworld"));
        assert!(!is_safe(b"he\x1b[201~llo"));
    }

    #[test]
    fn paste_encode_bracketed() {
        let mut data = b"hello".to_vec();
        let result = encode(&mut data, PasteOptions { bracketed: true });
        assert_eq!(result, [b"\x1b[200~".as_slice(), b"hello", b"\x1b[201~"]);
    }

    #[test]
    fn paste_encode_unbracketed_no_newlines() {
        let mut data = b"hello".to_vec();
        let result = encode(&mut data, PasteOptions { bracketed: false });
        assert_eq!(result, [b"".as_slice(), b"hello", b""]);
    }

    #[test]
    fn paste_encode_unbracketed_newlines() {
        let mut data = b"hello\nworld".to_vec();
        let result = encode(&mut data, PasteOptions { bracketed: false });
        assert_eq!(result, [b"".as_slice(), b"hello\rworld", b""]);
        assert_eq!(data, b"hello\rworld");
    }

    #[test]
    fn paste_encode_strips_unsafe_bytes() {
        let mut data = b"hel\x1blo\x00world".to_vec();
        let result = encode(&mut data, PasteOptions { bracketed: true });
        assert_eq!(
            result,
            [
                b"\x1b[200~".as_slice(),
                b"hel lo world".as_slice(),
                b"\x1b[201~".as_slice()
            ]
        );
        assert_eq!(data, b"hel lo world");
    }

    #[test]
    fn paste_encode_strips_representative_control_bytes() {
        let mut data = vec![b'a', 0x03, b'b', 0x7f, b'c', 0x15, b'd'];
        let result = encode(&mut data, PasteOptions { bracketed: false });
        assert_eq!(result, [b"".as_slice(), b"a b c d".as_slice(), b""]);
        assert_eq!(data, b"a b c d");
    }
}
