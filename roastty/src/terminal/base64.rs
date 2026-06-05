//! Public base64 decode entry point (port of upstream `simd/base64`, scalar path).
//!
//! Upstream dispatches to a C++ SIMD decoder or the scalar `base64_scalar.scalar_decoder`; roastty
//! has no SIMD base64 artifact, so only the scalar path is ported (upstream's `if comptime
//! options.simd` branches and the C++ SIMD `extern` declarations drop). Used for OSC 52 and Kitty
//! Graphics payloads.

use super::base64_scalar::scalar_decoder;

/// Invalid base64 input (upstream `error{Base64Invalid}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Base64Invalid;

/// The maximum decoded length for `input` (upstream `maxLen` / `maxLenScalar`). Returns 0 on a
/// sizing error (upstream logs and returns 0).
pub(crate) fn max_len(input: &[u8]) -> usize {
    scalar_decoder()
        .calc_size_for_slice(scalar_input(input))
        .unwrap_or(0)
}

/// Decode `input` into `output` (which must be at least `max_len(input)` bytes), returning the
/// decoded prefix (upstream `decode` / `decodeScalar`).
pub(crate) fn decode<'o>(input: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], Base64Invalid> {
    let stripped = scalar_input(input);
    let size = max_len(stripped);
    if size == 0 {
        return Ok(&[]);
    }
    assert!(output.len() >= size);
    scalar_decoder()
        .decode(output, stripped)
        .map_err(|_| Base64Invalid)?;
    Ok(&output[..size])
}

/// Trim trailing `=` padding so the no-pad scalar decoder accepts padded input and matches the SIMD
/// path's output (upstream `scalarInput`). Counts the padding safely, so empty / all-`=` input
/// yields an empty slice instead of upstream's index underflow.
fn scalar_input(input: &[u8]) -> &[u8] {
    let trailing = input.iter().rev().take_while(|&&b| b == b'=').count();
    &input[..input.len() - trailing]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_vec(input: &[u8]) -> Result<Vec<u8>, Base64Invalid> {
        let mut buf = vec![0u8; max_len(input)];
        decode(input, &mut buf).map(|s| s.to_vec())
    }

    #[test]
    fn max_len_of_padded_input() {
        assert_eq!(max_len(b"aGVsbG8gd29ybGQ="), 11);
    }

    #[test]
    fn decode_padded_input() {
        assert_eq!(decode_vec(b"aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
    }

    #[test]
    fn decode_unpadded_input() {
        assert_eq!(decode_vec(b"TWFu").unwrap(), b"Man");
    }

    #[test]
    fn decode_strips_multiple_padding() {
        assert_eq!(decode_vec(b"TWE==").unwrap(), b"Ma");
    }

    #[test]
    fn decode_invalid_input_errors() {
        // `*` is not in the standard alphabet; sizing succeeds, decode fails.
        assert_eq!(decode_vec(b"TW*u"), Err(Base64Invalid));
    }

    #[test]
    fn max_len_of_empty_is_zero() {
        assert_eq!(max_len(b""), 0);
        assert_eq!(max_len(b"===="), 0);
    }

    #[test]
    fn decode_empty_is_empty() {
        let mut buf = [0u8; 4];
        assert_eq!(decode(b"", &mut buf).unwrap(), b"");
        assert_eq!(decode(b"====", &mut buf).unwrap(), b"");
    }
}
