//! Temporary-path helpers (port of upstream `os/file`).

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;

/// The url-safe, no-padding base64 alphabet (`std.base64.url_safe_no_pad`).
const BASE64_URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Number of random bytes behind a basename.
const RANDOM_BASENAME_BYTES: usize = 16;

/// The encoded length of a basename (`calcSize(16) = 22`).
pub(crate) const RANDOM_BASENAME_LEN: usize = (RANDOM_BASENAME_BYTES * 4 + 2) / 3;

/// Encode bytes as url-safe base64 without padding (upstream
/// `std.base64.url_safe_no_pad.Encoder`).
fn base64_url_no_pad(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() * 4 + 2) / 3);
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let n = (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | chunk[2] as u32;
        out.push(BASE64_URL[(n >> 18 & 63) as usize] as char);
        out.push(BASE64_URL[(n >> 12 & 63) as usize] as char);
        out.push(BASE64_URL[(n >> 6 & 63) as usize] as char);
        out.push(BASE64_URL[(n & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(BASE64_URL[(n >> 18 & 63) as usize] as char);
            out.push(BASE64_URL[(n >> 12 & 63) as usize] as char);
        }
        2 => {
            let n = (rem[0] as u32) << 16 | (rem[1] as u32) << 8;
            out.push(BASE64_URL[(n >> 18 & 63) as usize] as char);
            out.push(BASE64_URL[(n >> 12 & 63) as usize] as char);
            out.push(BASE64_URL[(n >> 6 & 63) as usize] as char);
        }
        _ => {}
    }
    out
}

/// A random filesystem-safe base64 basename of length `RANDOM_BASENAME_LEN` (upstream
/// `os.file.randomBasename`). Always allocated (Rust owns the buffer), so the upstream
/// `BufferTooSmall` case does not arise.
pub(crate) fn random_basename() -> String {
    let mut bytes = [0u8; RANDOM_BASENAME_BYTES];
    // arc4random_buf is a CSPRNG on macOS (faithful to std.crypto.random) and never fails.
    unsafe { libc::arc4random_buf(bytes.as_mut_ptr() as *mut libc::c_void, bytes.len()) };
    base64_url_no_pad(&bytes)
}

/// The recommended temp directory with any trailing separator stripped (upstream
/// `os.file.allocTmpDir`): `$TMPDIR`, else `$TMP`, else `/tmp`.
pub(crate) fn tmp_dir() -> OsString {
    resolve_tmp_dir(std::env::var_os("TMPDIR").or_else(|| std::env::var_os("TMP")))
}

/// The temp-dir resolution core, parameterized over the resolved env value for testability.
fn resolve_tmp_dir(value: Option<OsString>) -> OsString {
    match value {
        Some(dir) => trim_end_separators(&dir),
        None => OsString::from("/tmp"),
    }
}

/// Strip all trailing `/` bytes (faithful to `std.mem.trimEnd(.., '/')`).
fn trim_end_separators(dir: &OsStr) -> OsString {
    let bytes = dir.as_bytes();
    let end = bytes.iter().rposition(|&b| b != b'/').map_or(0, |i| i + 1);
    OsStr::from_bytes(&bytes[..end]).to_os_string()
}

/// `{tmp}/{prefix}{random}` (upstream `os.file.randomTmpPath`). Nothing is created on disk.
pub(crate) fn random_tmp_path(prefix: &OsStr) -> OsString {
    let tmp = tmp_dir();
    let basename = random_basename();
    let mut path = OsString::with_capacity(tmp.len() + 1 + prefix.len() + basename.len());
    path.push(&tmp);
    path.push("/");
    path.push(prefix);
    path.push(basename);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_url_no_pad_known_vectors() {
        assert_eq!(base64_url_no_pad(b"Man"), "TWFu");
        assert_eq!(base64_url_no_pad(b"Ma"), "TWE");
        assert_eq!(base64_url_no_pad(b"M"), "TQ");
        assert_eq!(base64_url_no_pad(&[0u8; 16]), "A".repeat(22));
    }

    #[test]
    fn random_basename_is_filesystem_safe() {
        let name = random_basename();
        assert_eq!(name.len(), RANDOM_BASENAME_LEN);
        assert_eq!(RANDOM_BASENAME_LEN, 22);
        for c in name.bytes() {
            assert!(
                c.is_ascii_alphanumeric() || c == b'-' || c == b'_',
                "unexpected basename char {c:#x}",
            );
        }
        // Two basenames should (with overwhelming probability) differ.
        assert_ne!(random_basename(), random_basename());
    }

    #[test]
    fn resolve_tmp_dir_trims_trailing_separators() {
        assert_eq!(
            resolve_tmp_dir(Some(OsString::from("/foo/"))),
            OsString::from("/foo")
        );
        assert_eq!(
            resolve_tmp_dir(Some(OsString::from("/foo//"))),
            OsString::from("/foo"),
        );
        assert_eq!(
            resolve_tmp_dir(Some(OsString::from("/tmp"))),
            OsString::from("/tmp")
        );
        assert_eq!(resolve_tmp_dir(None), OsString::from("/tmp"));
    }

    #[test]
    fn random_tmp_path_composes_tmp_prefix_basename() {
        let tmp = tmp_dir();
        let prefix = OsStr::new("roastty-");
        let path = random_tmp_path(prefix);

        let bytes = path.as_bytes();
        assert!(bytes.starts_with(tmp.as_bytes()));
        // The byte after tmp is the separator, then the prefix.
        let after_tmp = &bytes[tmp.as_bytes().len()..];
        assert!(after_tmp.starts_with(b"/roastty-"));
        // Total length = tmp + '/' + prefix + 22-char basename.
        assert_eq!(
            bytes.len(),
            tmp.as_bytes().len() + 1 + prefix.as_bytes().len() + RANDOM_BASENAME_LEN,
        );
        // Two paths should differ.
        assert_ne!(random_tmp_path(prefix), random_tmp_path(prefix));
    }
}
