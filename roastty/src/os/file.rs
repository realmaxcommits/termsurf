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

/// Maximize the number of open file descriptors (`RLIMIT_NOFILE`) and return the previous
/// limit so it can be restored (upstream `os.file.fixMaxFiles`). Each window/pane consumes
/// several fds, so we raise the soft limit toward the hard limit. `None` if the limit can't
/// be queried.
pub(crate) fn fix_max_files() -> Option<libc::rlimit> {
    let mut old = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // Oh well; we tried. (Upstream logs a warning that max windows may be limited.)
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut old) } != 0 {
        return None;
    }

    // If we're already at the max, we're done.
    if old.rlim_cur >= old.rlim_max {
        return Some(old);
    }

    // Binary search for the limit.
    let mut min: libc::rlim_t = old.rlim_cur;
    let mut max: libc::rlim_t = 1 << 20;
    // If there's a defined upper bound, don't search — just set it.
    if old.rlim_max != libc::RLIM_INFINITY {
        min = old.rlim_max;
        max = old.rlim_max;
    }

    loop {
        let mut lim = old;
        lim.rlim_cur = min + (max - min) / 2;
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &lim) } == 0 {
            min = lim.rlim_cur;
        } else {
            max = lim.rlim_cur;
        }
        if min + 1 >= max {
            break;
        }
    }

    Some(old)
}

/// Restore a file-descriptor limit previously returned by `fix_max_files` (upstream
/// `os.file.restoreMaxFiles`). Errors are ignored.
pub(crate) fn restore_max_files(lim: libc::rlimit) {
    unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &lim) };
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

    #[test]
    fn fix_max_files_raises_then_restores() {
        fn query() -> libc::rlimit {
            let mut lim = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            assert_eq!(
                unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut lim) },
                0,
                "getrlimit failed",
            );
            lim
        }

        let old = fix_max_files().expect("rlimit should be queryable");

        // The soft limit is never lowered by fixing.
        let after = query();
        assert!(after.rlim_cur >= old.rlim_cur);

        // Restoring returns the limit to exactly the old value.
        restore_max_files(old);
        let restored = query();
        assert_eq!(restored.rlim_cur, old.rlim_cur);
        assert_eq!(restored.rlim_max, old.rlim_max);
    }
}
