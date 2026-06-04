//! Config entry formatting (port of upstream `config/formatter.zig`).
//!
//! `EntryFormatter` writes one `name = value\n` config line. The comptime,
//! field-dispatch generic `formatEntry` (auto-formatting fields with no custom
//! `formatEntry`) is ported later; this is the object the custom `formatEntry`
//! methods call.
#![allow(dead_code)]

use std::fmt::{Display, Write as _};

/// Writes a single `name = value\n` config entry (upstream
/// `config.formatter.EntryFormatter`).
pub(crate) struct EntryFormatter<'a> {
    name: &'a str,
    out: &'a mut String,
}

impl<'a> EntryFormatter<'a> {
    pub(crate) fn new(name: &'a str, out: &'a mut String) -> Self {
        EntryFormatter { name, out }
    }

    /// `name = value\n` (upstream the `[]const u8` / `[:0]const u8` case).
    pub(crate) fn entry_str(&mut self, value: &str) {
        let _ = writeln!(self.out, "{} = {}", self.name, value);
    }

    /// `name = true|false\n` (upstream the `bool` case).
    pub(crate) fn entry_bool(&mut self, value: bool) {
        let _ = writeln!(self.out, "{} = {}", self.name, value);
    }

    /// `name = <decimal>\n` (upstream the `int` case).
    pub(crate) fn entry_int(&mut self, value: impl Display) {
        let _ = writeln!(self.out, "{} = {}", self.name, value);
    }

    /// `name = \n` (upstream the `void` case).
    pub(crate) fn entry_void(&mut self) {
        let _ = writeln!(self.out, "{} = ", self.name);
    }

    /// `name = [no-]field,[no-]field…\n` (upstream the packed-struct case): each
    /// flag is its keyword, prefixed with `no-` when `false`.
    pub(crate) fn entry_flags(&mut self, fields: &[(&str, bool)]) {
        let joined = fields
            .iter()
            .map(|&(name, on)| {
                if on {
                    name.to_string()
                } else {
                    format!("no-{}", name)
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        self.entry_str(&joined);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_formatter_writes_primitive_lines() {
        let mut out = String::new();
        EntryFormatter::new("a", &mut out).entry_str("v");
        assert_eq!(out, "a = v\n");

        let mut out = String::new();
        EntryFormatter::new("a", &mut out).entry_bool(true);
        assert_eq!(out, "a = true\n");

        let mut out = String::new();
        EntryFormatter::new("a", &mut out).entry_bool(false);
        assert_eq!(out, "a = false\n");

        let mut out = String::new();
        EntryFormatter::new("a", &mut out).entry_int(42u8);
        assert_eq!(out, "a = 42\n");

        let mut out = String::new();
        EntryFormatter::new("a", &mut out).entry_void();
        assert_eq!(out, "a = \n");
    }

    #[test]
    fn entry_flags_writes_comma_joined_keywords() {
        let mut out = String::new();
        EntryFormatter::new("x", &mut out).entry_flags(&[("a", true), ("b", false)]);
        assert_eq!(out, "x = a,no-b\n");
    }
}
