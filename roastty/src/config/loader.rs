//! Config-file loading (port of upstream `cli/args` `LineIterator`).
//!
//! Parses config-file lines into `(key, value)` pairs that drive `Config::set`. The
//! multi-line driver and file IO are layered on top of this per-line extraction.
#![allow(dead_code)]

/// Parse one config-file line into a `(key, value)` pair (upstream
/// `cli.args.LineIterator.next`'s per-line logic). Returns `None` for a blank line or
/// a `#` comment. A line with `=` yields `(key, Some(value))` with the key and value
/// `" \t"`-trimmed and the value's surrounding double quotes stripped (not decoded —
/// the per-field parsers decode any inner escapes); a line with no `=` yields
/// `(key, None)` (a bare key).
///
/// The line must already be split on `\n` (no trailing newline); the surrounding
/// trim removes `" \t\r"` (so a CRLF line's `\r` is handled).
pub(crate) fn parse_config_line(line: &str) -> Option<(&str, Option<&str>)> {
    let edge = |c: char| c == ' ' || c == '\t' || c == '\r';
    let ws = |c: char| c == ' ' || c == '\t';

    let trimmed = line.trim_matches(edge);
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    match trimmed.find('=') {
        Some(idx) => {
            let key = trimmed[..idx].trim_matches(ws);
            let mut value = trimmed[idx + 1..].trim_matches(ws);
            if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
                value = &value[1..value.len() - 1];
            }
            Some((key, Some(value)))
        }
        None => Some((trimmed, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_line_extracts_key_value() {
        // A `key = value` line; whitespace around the key and value is trimmed.
        assert_eq!(
            parse_config_line("key = value"),
            Some(("key", Some("value")))
        );
        assert_eq!(
            parse_config_line("  key  =  value  "),
            Some(("key", Some("value")))
        );
        // A surrounding double-quoted value is unwrapped (not decoded).
        assert_eq!(
            parse_config_line("key = \"a b\""),
            Some(("key", Some("a b")))
        );
        // An empty value after the `=`.
        assert_eq!(parse_config_line("key ="), Some(("key", Some(""))));
        // A bare key with no `=` carries no value.
        assert_eq!(parse_config_line("flag"), Some(("flag", None)));
        // A CRLF line: the trailing `\r` is trimmed.
        assert_eq!(
            parse_config_line("key=value\r"),
            Some(("key", Some("value")))
        );
    }

    #[test]
    fn parse_config_line_skips_blank_and_comments() {
        assert_eq!(parse_config_line(""), None);
        assert_eq!(parse_config_line("   "), None);
        assert_eq!(parse_config_line("# a comment"), None);
        // A comment is detected after the surrounding trim (leading spaces).
        assert_eq!(parse_config_line("   # x = y"), None);
    }
}
