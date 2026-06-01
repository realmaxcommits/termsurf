use super::color::Rgb;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Separator {
    #[default]
    None,
    Semicolon,
    Colon,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Underline {
    #[default]
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Attribute {
    Unset,
    Unknown,
    Bold,
    ResetBold,
    Faint,
    Italic,
    ResetItalic,
    Underline(Underline),
    UnderlineColor(Rgb),
    PaletteUnderlineColor(u8),
    ResetUnderlineColor,
    Overline,
    ResetOverline,
    Blink,
    ResetBlink,
    Inverse,
    ResetInverse,
    Invisible,
    ResetInvisible,
    Strikethrough,
    ResetStrikethrough,
    DirectColorFg(Rgb),
    DirectColorBg(Rgb),
    PaletteFg(u8),
    PaletteBg(u8),
    ResetFg,
    ResetBg,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Parser<'a> {
    params: &'a [u16],
    separators: &'a [Separator],
    idx: usize,
}

impl<'a> Parser<'a> {
    pub(super) const fn new(params: &'a [u16], separators: &'a [Separator]) -> Self {
        Self {
            params,
            separators,
            idx: 0,
        }
    }

    pub(super) fn next(&mut self) -> Option<Attribute> {
        if self.idx >= self.params.len() {
            let reset = self.idx == 0;
            self.idx += 1;
            return reset.then_some(Attribute::Unset);
        }

        let slice = &self.params[self.idx..];
        let colon = self.separator_at(self.idx) == Separator::Colon;
        self.idx += 1;

        if colon {
            match slice[0] {
                4 | 38 | 48 | 58 => {}
                _ => {
                    self.consume_unknown_colon();
                    return Some(Attribute::Unknown);
                }
            }
        }

        let attr = match slice[0] {
            0 => Attribute::Unset,
            1 => Attribute::Bold,
            2 => Attribute::Faint,
            3 => Attribute::Italic,
            4 => {
                if colon {
                    if slice.len() < 2 || self.separator_at(self.idx) == Separator::Colon {
                        self.consume_unknown_colon();
                        Attribute::Unknown
                    } else {
                        self.idx += 1;
                        Attribute::Underline(match slice[1] {
                            0 => Underline::None,
                            1 => Underline::Single,
                            2 => Underline::Double,
                            3 => Underline::Curly,
                            4 => Underline::Dotted,
                            5 => Underline::Dashed,
                            _ => Underline::Single,
                        })
                    }
                } else {
                    Attribute::Underline(Underline::Single)
                }
            }
            5 | 6 => Attribute::Blink,
            7 => Attribute::Inverse,
            8 => Attribute::Invisible,
            9 => Attribute::Strikethrough,
            21 => Attribute::Underline(Underline::Double),
            22 => Attribute::ResetBold,
            23 => Attribute::ResetItalic,
            24 => Attribute::Underline(Underline::None),
            25 => Attribute::ResetBlink,
            27 => Attribute::ResetInverse,
            28 => Attribute::ResetInvisible,
            29 => Attribute::ResetStrikethrough,
            30..=37 => Attribute::PaletteFg((slice[0] - 30) as u8),
            38 => match self.parse_color(slice, colon) {
                Some(ColorAttr::Rgb(rgb)) => Attribute::DirectColorFg(rgb),
                Some(ColorAttr::Palette(idx)) => Attribute::PaletteFg(idx),
                None => Attribute::Unknown,
            },
            39 => Attribute::ResetFg,
            40..=47 => Attribute::PaletteBg((slice[0] - 40) as u8),
            48 => match self.parse_color(slice, colon) {
                Some(ColorAttr::Rgb(rgb)) => Attribute::DirectColorBg(rgb),
                Some(ColorAttr::Palette(idx)) => Attribute::PaletteBg(idx),
                None => Attribute::Unknown,
            },
            49 => Attribute::ResetBg,
            53 => Attribute::Overline,
            55 => Attribute::ResetOverline,
            58 => match self.parse_color(slice, colon) {
                Some(ColorAttr::Rgb(rgb)) => Attribute::UnderlineColor(rgb),
                Some(ColorAttr::Palette(idx)) => Attribute::PaletteUnderlineColor(idx),
                None => Attribute::Unknown,
            },
            59 => Attribute::ResetUnderlineColor,
            90..=97 => Attribute::PaletteFg((slice[0] - 82) as u8),
            100..=107 => Attribute::PaletteBg((slice[0] - 92) as u8),
            _ => Attribute::Unknown,
        };

        Some(attr)
    }

    fn parse_color(&mut self, slice: &[u16], colon: bool) -> Option<ColorAttr> {
        if slice.len() < 2 {
            if colon {
                self.consume_unknown_colon();
            }
            return None;
        }

        match slice[1] {
            2 => self.parse_direct_color(slice, colon).map(ColorAttr::Rgb),
            5 if slice.len() >= 3 => {
                self.idx += 2;
                Some(ColorAttr::Palette(slice[2] as u8))
            }
            _ => {
                if colon {
                    self.consume_unknown_colon();
                }
                None
            }
        }
    }

    fn parse_direct_color(&mut self, slice: &[u16], colon: bool) -> Option<Rgb> {
        if slice.len() < 5 {
            return None;
        }

        if !colon {
            self.idx += 4;
            return Some(Rgb::new(slice[2] as u8, slice[3] as u8, slice[4] as u8));
        }

        match self.count_colon() {
            3 => {
                self.idx += 4;
                Some(Rgb::new(slice[2] as u8, slice[3] as u8, slice[4] as u8))
            }
            4 => {
                if slice.len() < 6 {
                    self.consume_unknown_colon();
                    return None;
                }
                self.idx += 5;
                Some(Rgb::new(slice[3] as u8, slice[4] as u8, slice[5] as u8))
            }
            _ => {
                self.consume_unknown_colon();
                None
            }
        }
    }

    fn separator_at(&self, idx: usize) -> Separator {
        self.separators.get(idx).copied().unwrap_or(Separator::None)
    }

    fn count_colon(&self) -> usize {
        let mut count = 0;
        let mut idx = self.idx;
        while idx < self.params.len().saturating_sub(1)
            && self.separator_at(idx) == Separator::Colon
        {
            count += 1;
            idx += 1;
        }
        count
    }

    fn consume_unknown_colon(&mut self) {
        self.idx += self.count_colon() + 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorAttr {
    Rgb(Rgb),
    Palette(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underline_default_matches_upstream() {
        assert_eq!(Underline::default(), Underline::None);
    }

    #[test]
    fn sgr_parser_reset_and_basic_flags() {
        let mut parser = Parser::new(&[], &[]);
        assert_eq!(parser.next(), Some(Attribute::Unset));
        assert_eq!(parser.next(), None);

        let params = [1, 2, 3, 22, 23];
        let seps = [Separator::Semicolon; 5];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::Bold));
        assert_eq!(parser.next(), Some(Attribute::Faint));
        assert_eq!(parser.next(), Some(Attribute::Italic));
        assert_eq!(parser.next(), Some(Attribute::ResetBold));
        assert_eq!(parser.next(), Some(Attribute::ResetItalic));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn sgr_parser_colors() {
        let params = [31, 44, 90, 103, 38, 5, 161, 48, 2, 1, 2, 3];
        let seps = [Separator::Semicolon; 12];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::PaletteFg(1)));
        assert_eq!(parser.next(), Some(Attribute::PaletteBg(4)));
        assert_eq!(parser.next(), Some(Attribute::PaletteFg(8)));
        assert_eq!(parser.next(), Some(Attribute::PaletteBg(11)));
        assert_eq!(parser.next(), Some(Attribute::PaletteFg(161)));
        assert_eq!(
            parser.next(),
            Some(Attribute::DirectColorBg(Rgb::new(1, 2, 3)))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn sgr_parser_colon_forms() {
        let params = [4, 3, 38, 2, 0, 1, 2, 3, 58, 2, 1, 2, 3];
        let seps = [
            Separator::Colon,
            Separator::Semicolon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Semicolon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::None,
        ];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::Underline(Underline::Curly)));
        assert_eq!(
            parser.next(),
            Some(Attribute::DirectColorFg(Rgb::new(1, 2, 3)))
        );
        assert_eq!(
            parser.next(),
            Some(Attribute::UnderlineColor(Rgb::new(1, 2, 3)))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn sgr_parser_kakoune_inputs() {
        let params = [0, 4, 3, 38, 2, 175, 175, 215, 58, 2, 0, 190, 80, 70];
        let seps = [
            Separator::Semicolon,
            Separator::Colon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::Colon,
            Separator::None,
        ];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::Unset));
        assert_eq!(parser.next(), Some(Attribute::Underline(Underline::Curly)));
        assert_eq!(
            parser.next(),
            Some(Attribute::DirectColorFg(Rgb::new(175, 175, 215)))
        );
        assert_eq!(
            parser.next(),
            Some(Attribute::UnderlineColor(Rgb::new(190, 80, 70)))
        );
        assert_eq!(parser.next(), None);

        let params = [
            4, 3, 38, 2, 51, 51, 51, 48, 2, 170, 170, 170, 58, 2, 255, 97, 136,
        ];
        let seps = [
            Separator::Colon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::Semicolon,
            Separator::None,
        ];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::Underline(Underline::Curly)));
        assert_eq!(
            parser.next(),
            Some(Attribute::DirectColorFg(Rgb::new(51, 51, 51)))
        );
        assert_eq!(
            parser.next(),
            Some(Attribute::DirectColorBg(Rgb::new(170, 170, 170)))
        );
        assert_eq!(
            parser.next(),
            Some(Attribute::UnderlineColor(Rgb::new(255, 97, 136)))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn sgr_parser_trailing_colon_unknown_does_not_loop() {
        let params = [58, 4];
        let seps = [Separator::Colon, Separator::Colon];
        let mut parser = Parser::new(&params, &seps);
        assert_eq!(parser.next(), Some(Attribute::Unknown));
        assert_eq!(parser.next(), None);
    }
}
