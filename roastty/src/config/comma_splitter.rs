//! A comma splitter that respects double-quotes and Zig-style escapes (port of
//! upstream `cli/CommaSplitter`).
//!
//! Splits a string into comma-separated fields, where a comma inside a quoted
//! section (or an escape sequence) does NOT split. Quotes and escapes are NOT
//! decoded — that is a separate step; this only finds field boundaries.
//!
//! macOS resolves the upstream `escape_outside_quotes` constant to `true`, so a
//! backslash is an escape character both inside and outside quotes.
#![allow(dead_code)]

/// macOS: backslash escapes both inside and outside quotes (upstream
/// `escape_outside_quotes = os != windows`).
const ESCAPE_OUTSIDE_QUOTES: bool = true;

/// An error splitting a comma list (upstream `CommaSplitter.Error`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommaSplitError {
    /// A quoted section was not closed.
    UnclosedQuote,
    /// An escape sequence was cut off at end of input.
    UnfinishedEscape,
    /// An escape sequence was malformed.
    IllegalEscape,
}

/// Iterator splitting a string into comma-separated fields, quote/escape aware
/// (upstream `cli.CommaSplitter`).
pub(crate) struct CommaSplitter<'a> {
    input: &'a str,
    index: usize,
}

/// The state-machine states (upstream's labeled-`switch` states).
#[derive(Clone, Copy)]
enum State {
    Normal,
    Quoted,
    Escape,
    Hexescape,
    Unicodeescape,
}

/// The `unicodeescape` sub-state.
#[derive(Clone, Copy)]
enum UnicodeState {
    Start,
    Digits,
}

impl<'a> CommaSplitter<'a> {
    /// Initialize a splitter over the given string (upstream `init`).
    pub(crate) fn new(input: &'a str) -> Self {
        CommaSplitter { input, index: 0 }
    }

    /// The next comma-separated field (quotes/escapes intact), or `None` when
    /// exhausted (upstream `CommaSplitter.next`).
    pub(crate) fn next(&mut self) -> Result<Option<&'a str>, CommaSplitError> {
        let bytes = self.input.as_bytes();
        if self.index >= bytes.len() {
            return Ok(None);
        }

        let start = self.index;
        // The state to return to when an escape sequence completes.
        let mut last = State::Normal;
        // Digits seen in a `\xNN` hex escape.
        let mut hexescape_digits: usize = 0;
        // Sub-state of a `\u{…}` unicode escape.
        let mut unicode_state = UnicodeState::Start;
        // Digits seen / accumulated value of the unicode escape. The value is only
        // used for the `> 0x10ffff` overflow guard (never decoded), so the exact
        // accumulator arithmetic is copied verbatim from upstream.
        let mut unicode_digits: usize = 0;
        let mut unicode_value: usize = 0;

        let mut state = State::Normal;
        loop {
            match state {
                State::Normal => {
                    if self.index >= bytes.len() {
                        return Ok(Some(&self.input[start..]));
                    }
                    match bytes[self.index] {
                        b',' => {
                            self.index += 1;
                            return Ok(Some(&self.input[start..self.index - 1]));
                        }
                        b'"' => {
                            self.index += 1;
                            state = State::Quoted;
                        }
                        b'\\' => {
                            self.index += 1;
                            if ESCAPE_OUTSIDE_QUOTES {
                                last = State::Normal;
                                state = State::Escape;
                            } else {
                                state = State::Normal;
                            }
                        }
                        _ => {
                            self.index += 1;
                            state = State::Normal;
                        }
                    }
                }
                State::Quoted => {
                    if self.index >= bytes.len() {
                        return Err(CommaSplitError::UnclosedQuote);
                    }
                    match bytes[self.index] {
                        b'"' => {
                            self.index += 1;
                            state = State::Normal;
                        }
                        b'\\' => {
                            self.index += 1;
                            last = State::Quoted;
                            state = State::Escape;
                        }
                        _ => {
                            self.index += 1;
                            state = State::Quoted;
                        }
                    }
                }
                State::Escape => {
                    if self.index >= bytes.len() {
                        return Err(CommaSplitError::UnfinishedEscape);
                    }
                    match bytes[self.index] {
                        b'n' | b'r' | b't' | b'\\' | b'\'' | b'"' => {
                            self.index += 1;
                            state = last;
                        }
                        b'x' => {
                            self.index += 1;
                            hexescape_digits = 0;
                            state = State::Hexescape;
                        }
                        b'u' => {
                            self.index += 1;
                            unicode_state = UnicodeState::Start;
                            unicode_digits = 0;
                            unicode_value = 0;
                            state = State::Unicodeescape;
                        }
                        _ => return Err(CommaSplitError::IllegalEscape),
                    }
                }
                State::Hexescape => {
                    if self.index >= bytes.len() {
                        return Err(CommaSplitError::UnfinishedEscape);
                    }
                    match bytes[self.index] {
                        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                            self.index += 1;
                            hexescape_digits += 1;
                            if hexescape_digits == 2 {
                                state = last;
                            }
                        }
                        _ => return Err(CommaSplitError::IllegalEscape),
                    }
                }
                State::Unicodeescape => {
                    if self.index >= bytes.len() {
                        return Err(CommaSplitError::UnfinishedEscape);
                    }
                    match unicode_state {
                        UnicodeState::Start => match bytes[self.index] {
                            b'{' => {
                                self.index += 1;
                                unicode_value = 0;
                                unicode_state = UnicodeState::Digits;
                            }
                            _ => return Err(CommaSplitError::IllegalEscape),
                        },
                        UnicodeState::Digits => {
                            match bytes[self.index] {
                                b'}' => {
                                    self.index += 1;
                                    if unicode_digits == 0 {
                                        return Err(CommaSplitError::IllegalEscape);
                                    }
                                    state = last;
                                    continue;
                                }
                                d @ b'0'..=b'9' => {
                                    self.index += 1;
                                    unicode_digits += 1;
                                    unicode_value <<= 4;
                                    unicode_value += (d - b'0') as usize;
                                }
                                d @ b'a'..=b'f' => {
                                    self.index += 1;
                                    unicode_digits += 1;
                                    unicode_value <<= 4;
                                    unicode_value += (d - b'a') as usize;
                                }
                                d @ b'A'..=b'F' => {
                                    self.index += 1;
                                    unicode_digits += 1;
                                    unicode_value <<= 4;
                                    unicode_value += (d - b'A') as usize;
                                }
                                _ => return Err(CommaSplitError::IllegalEscape),
                            }
                            if unicode_value > 0x10ffff {
                                return Err(CommaSplitError::IllegalEscape);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(input: &str) -> Result<Vec<String>, CommaSplitError> {
        let mut s = CommaSplitter::new(input);
        let mut out = Vec::new();
        while let Some(field) = s.next()? {
            out.push(field.to_string());
        }
        Ok(out)
    }

    #[test]
    fn comma_splitter_basic_fields() {
        assert_eq!(collect("a,b,c").unwrap(), ["a", "b", "c"]);
        assert_eq!(collect("").unwrap(), Vec::<String>::new());
        assert_eq!(collect("a").unwrap(), ["a"]);
        // An escape is consumed but not decoded; the field is the raw string.
        assert_eq!(collect(r"\x5a").unwrap(), [r"\x5a"]);
        // Single quotes are not special.
        assert_eq!(collect("'a',b").unwrap(), ["'a'", "b"]);
        assert_eq!(collect("'a,b',c").unwrap(), ["'a", "b'", "c"]);
        // Double quotes protect the comma.
        assert_eq!(collect("\"a,b\",c").unwrap(), ["\"a,b\"", "c"]);
        // Whitespace is preserved.
        assert_eq!(collect(" a , b ").unwrap(), [" a ", " b "]);
    }

    #[test]
    fn comma_splitter_errors() {
        assert_eq!(collect(r"\x"), Err(CommaSplitError::UnfinishedEscape));
        assert_eq!(collect(r"\x5"), Err(CommaSplitError::UnfinishedEscape));
        assert_eq!(collect(r"\u"), Err(CommaSplitError::UnfinishedEscape));
        assert_eq!(collect(r"\u{"), Err(CommaSplitError::UnfinishedEscape));
        assert_eq!(collect(r"\u{}"), Err(CommaSplitError::IllegalEscape));
        // An unclosed quote.
        assert_eq!(collect("\"a"), Err(CommaSplitError::UnclosedQuote));
        // A non-escape character after a backslash.
        assert_eq!(collect(r"\d"), Err(CommaSplitError::IllegalEscape));
    }

    #[test]
    fn comma_splitter_unicode_overflow_quirk() {
        // Under upstream's exact accumulator (`a`-`f` add `d - 'a'`, i.e. 0–5, not
        // +10), these consume without overflowing the `> 0x10ffff` guard.
        assert_eq!(collect(r"\u{10ffff}").unwrap(), [r"\u{10ffff}"]);
        assert_eq!(collect(r"\u{afffff}").unwrap(), [r"\u{afffff}"]);
        // …but a value that does exceed the guard errors.
        assert_eq!(collect(r"\u{110000}"), Err(CommaSplitError::IllegalEscape));
    }
}
