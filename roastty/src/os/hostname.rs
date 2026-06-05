//! Hostname helpers (port of upstream `os/hostname`).

/// Error from validating whether a hostname is local (upstream
/// `os.hostname.LocalHostnameValidationError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalHostnameError {
    PermissionDenied,
    Unexpected,
}

/// Validates a hostname per [RFC 1123](https://www.rfc-editor.org/rfc/rfc1123) (upstream
/// `os.hostname.isValid`). Stricter than a permissive validator: rejects leading dots,
/// empty/over-long labels, and non-FQDN trailing junk.
pub(crate) fn is_valid(hostname: &[u8]) -> bool {
    if hostname.is_empty() {
        return false;
    }
    if hostname[0] == b'.' {
        return false;
    }

    // Ignore a single trailing dot (FQDN); it doesn't count toward the length.
    let end = if hostname[hostname.len() - 1] == b'.' {
        if hostname.len() == 1 {
            return false;
        }
        hostname.len() - 1
    } else {
        hostname.len()
    };

    if end > 253 {
        return false;
    }

    // Hostnames are divided into dot-separated "labels", which start with a letter or
    // digit, can contain letters/digits/hyphens, must end with a letter or digit, and have
    // 1..=63 characters.
    let mut label_start = 0usize;
    let mut label_len = 0usize;
    for i in 0..end {
        let c = hostname[i];
        match c {
            b'.' => {
                if label_len == 0 || label_len > 63 {
                    return false;
                }
                if !hostname[label_start].is_ascii_alphanumeric() {
                    return false;
                }
                if !hostname[i - 1].is_ascii_alphanumeric() {
                    return false;
                }
                label_start = i + 1;
                label_len = 0;
            }
            b'-' => label_len += 1,
            _ => {
                if !c.is_ascii_alphanumeric() {
                    return false;
                }
                label_len += 1;
            }
        }
    }

    // Validate the final label.
    if label_len == 0 || label_len > 63 {
        return false;
    }
    if !hostname[label_start].is_ascii_alphanumeric() {
        return false;
    }
    if !hostname[end - 1].is_ascii_alphanumeric() {
        return false;
    }

    true
}

/// True if `hostname` is `localhost` or matches this machine's `gethostname` (upstream
/// `os.hostname.isLocal`).
pub(crate) fn is_local(hostname: &[u8]) -> Result<bool, LocalHostnameError> {
    // A 'localhost' hostname is always considered local.
    if hostname == b"localhost" {
        return Ok(true);
    }

    // `posix.HOST_NAME_MAX` is 72 on the macOS/Darwin family (vendored Zig std), the same
    // bound upstream's `var buf: [posix.HOST_NAME_MAX]u8` uses.
    const HOST_NAME_MAX: usize = 72;
    let mut buf = [0u8; HOST_NAME_MAX];
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc != 0 {
        let errno = std::io::Error::last_os_error().raw_os_error();
        return Err(match errno {
            Some(libc::EPERM) => LocalHostnameError::PermissionDenied,
            _ => LocalHostnameError::Unexpected,
        });
    }

    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(hostname == &buf[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeated(byte: u8, count: usize) -> Vec<u8> {
        vec![byte; count]
    }

    #[test]
    fn is_valid_matches_upstream() {
        // Valid hostnames.
        assert!(is_valid(b"example"));
        assert!(is_valid(b"example.com"));
        assert!(is_valid(b"www.example.com"));
        assert!(is_valid(b"sub.domain.example.com"));
        assert!(is_valid(b"example.com."));
        assert!(is_valid(b"host-name.example.com."));
        assert!(is_valid(b"123.example.com."));
        assert!(is_valid(b"a-b.com"));
        assert!(is_valid(b"a.b.c.d.e.f.g"));
        assert!(is_valid(b"127.0.0.1")); // Also a valid hostname.

        // Label exactly 63 chars (valid): "a"*63 + ".com".
        let mut label63 = repeated(b'a', 63);
        label63.extend_from_slice(b".com");
        assert!(is_valid(&label63));

        // Total length 253 (valid): "a." * 126 + "a".
        let mut len253 = Vec::new();
        for _ in 0..126 {
            len253.extend_from_slice(b"a.");
        }
        len253.push(b'a');
        assert_eq!(len253.len(), 253);
        assert!(is_valid(&len253));

        // Invalid hostnames.
        assert!(!is_valid(b""));
        assert!(!is_valid(b".example.com"));
        assert!(!is_valid(b"example.com.."));
        assert!(!is_valid(b"host..domain"));
        assert!(!is_valid(b"-hostname"));
        assert!(!is_valid(b"hostname-"));
        assert!(!is_valid(b"a.-.b"));
        assert!(!is_valid(b"host_name.com"));
        assert!(!is_valid(b"."));
        assert!(!is_valid(b".."));

        // Label length 64 (too long): "a"*64 + ".com".
        let mut label64 = repeated(b'a', 64);
        label64.extend_from_slice(b".com");
        assert!(!is_valid(&label64));

        // Total length 254 (too long): "a." * 126 + "ab".
        let mut len254 = Vec::new();
        for _ in 0..126 {
            len254.extend_from_slice(b"a.");
        }
        len254.extend_from_slice(b"ab");
        assert_eq!(len254.len(), 254);
        assert!(!is_valid(&len254));
    }

    #[test]
    fn is_local_returns_true_for_localhost() {
        assert_eq!(is_local(b"localhost"), Ok(true));
    }

    #[test]
    fn is_local_returns_true_for_this_machine() {
        let mut buf = [0u8; 72];
        let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        assert_eq!(rc, 0, "gethostname should succeed");
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        assert_eq!(is_local(&buf[..len]), Ok(true));
    }

    #[test]
    fn is_local_returns_false_for_other_hostname() {
        assert_eq!(is_local(b"not-the-local-hostname"), Ok(false));
    }
}
