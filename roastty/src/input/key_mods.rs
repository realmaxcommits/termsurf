#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mod {
    Shift,
    Ctrl,
    Alt,
    Super,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptionAsAlt {
    False,
    True,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ModSides {
    pub(crate) shift: Side,
    pub(crate) ctrl: Side,
    pub(crate) alt: Side,
    pub(crate) super_: Side,
}

impl Default for Side {
    fn default() -> Self {
        Self::Left
    }
}

impl ModSides {
    fn int(self) -> u16 {
        ((self.shift as u16) << 6)
            | ((self.ctrl as u16) << 7)
            | ((self.alt as u16) << 8)
            | ((self.super_ as u16) << 9)
    }

    fn from_int(value: u16) -> Self {
        Self {
            shift: side_from_bit(value, 6),
            ctrl: side_from_bit(value, 7),
            alt: side_from_bit(value, 8),
            super_: side_from_bit(value, 9),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ModKeys {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) super_: bool,
}

impl ModKeys {
    pub(crate) fn int(self) -> u8 {
        self.shift as u8
            | ((self.ctrl as u8) << 1)
            | ((self.alt as u8) << 2)
            | ((self.super_ as u8) << 3)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Mods {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) super_: bool,
    pub(crate) caps_lock: bool,
    pub(crate) num_lock: bool,
    pub(crate) sides: ModSides,
}

impl Mods {
    pub(crate) const fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            super_: false,
            caps_lock: false,
            num_lock: false,
            sides: ModSides {
                shift: Side::Left,
                ctrl: Side::Left,
                alt: Side::Left,
                super_: Side::Left,
            },
        }
    }

    pub(crate) fn for_mod(modifier: Mod, side: Side) -> Self {
        let mut mods = Self::new();
        match modifier {
            Mod::Shift => {
                mods.shift = true;
                mods.sides.shift = side;
            }
            Mod::Ctrl => {
                mods.ctrl = true;
                mods.sides.ctrl = side;
            }
            Mod::Alt => {
                mods.alt = true;
                mods.sides.alt = side;
            }
            Mod::Super => {
                mods.super_ = true;
                mods.sides.super_ = side;
            }
        }
        mods
    }

    pub(crate) fn int(self) -> u16 {
        self.shift as u16
            | ((self.ctrl as u16) << 1)
            | ((self.alt as u16) << 2)
            | ((self.super_ as u16) << 3)
            | ((self.caps_lock as u16) << 4)
            | ((self.num_lock as u16) << 5)
            | self.sides.int()
    }

    pub(crate) fn from_int(value: u16) -> Self {
        Self {
            shift: value & (1 << 0) != 0,
            ctrl: value & (1 << 1) != 0,
            alt: value & (1 << 2) != 0,
            super_: value & (1 << 3) != 0,
            caps_lock: value & (1 << 4) != 0,
            num_lock: value & (1 << 5) != 0,
            sides: ModSides::from_int(value),
        }
    }

    pub(crate) fn empty(self) -> bool {
        self.int() == 0
    }

    pub(crate) fn keys(self) -> ModKeys {
        ModKeys {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            super_: self.super_,
        }
    }

    pub(crate) fn binding(self) -> Self {
        Self {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            super_: self.super_,
            ..Self::new()
        }
    }

    pub(crate) fn unset(self, other: Self) -> Self {
        Self::from_int(self.int() & !other.int())
    }

    pub(crate) fn without_locks(self) -> Self {
        Self {
            caps_lock: false,
            num_lock: false,
            ..self
        }
    }

    pub(crate) fn translation(self, option_as_alt: OptionAsAlt) -> Self {
        let mut result = self;
        if !self.alt {
            return result;
        }

        match option_as_alt {
            OptionAsAlt::False => return result,
            OptionAsAlt::True => {}
            OptionAsAlt::Left if self.sides.alt == Side::Right => return result,
            OptionAsAlt::Right if self.sides.alt == Side::Left => return result,
            OptionAsAlt::Left | OptionAsAlt::Right => {}
        }

        result.alt = false;
        result
    }

    pub(crate) fn ctrl_or_super(self) -> bool {
        self.super_
    }
}

fn side_from_bit(value: u16, bit: u16) -> Side {
    if value & (1 << bit) == 0 {
        Side::Left
    } else {
        Side::Right
    }
}

pub(crate) fn ctrl_or_super(mut mods: Mods) -> Mods {
    mods.super_ = true;
    mods
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RemapSetParseError {
    MissingAssignment,
    InvalidMod,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RemapSet {
    map: Vec<(Mods, Mods)>,
    mask: RemapMask,
}

impl PartialEq for RemapSet {
    fn eq(&self, other: &Self) -> bool {
        self.map.len() == other.map.len()
            && self.map.iter().all(|(from, to)| {
                other
                    .map
                    .iter()
                    .any(|(other_from, other_to)| from == other_from && to == other_to)
            })
    }
}

impl Eq for RemapSet {}

impl RemapSet {
    pub(crate) fn parse_cli(&mut self, input: Option<&str>) -> Result<(), RemapSetParseError> {
        let value = input.unwrap_or("");
        if value.is_empty() {
            self.map.clear();
            self.mask = RemapMask::default();
            return Ok(());
        }

        self.parse(value)
    }

    pub(crate) fn parse(&mut self, input: &str) -> Result<(), RemapSetParseError> {
        let Some(eql_idx) = input.find('=') else {
            return Err(RemapSetParseError::MissingAssignment);
        };

        let to_raw = parse_remap_mod(&input[eql_idx + 1..])?;
        let to = init_mods(to_raw.0, to_raw.1.unwrap_or(Side::Left));

        let from_raw = parse_remap_mod(&input[..eql_idx])?;
        if let Some(side) = from_raw.1 {
            let from = init_mods(from_raw.0, side);
            self.put_mapping(from, to);
            return Ok(());
        }

        self.put_mapping(init_mods(from_raw.0, Side::Left), to);
        self.put_mapping(init_mods(from_raw.0, Side::Right), to);
        Ok(())
    }

    pub(crate) fn finalize(&mut self) {
        self.map
            .sort_by_key(|(from, _)| !remap_mods_has_right_side(*from));
    }

    pub(crate) fn is_remapped(&self, mods: Mods) -> bool {
        self.mask.matches(mods)
    }

    pub(crate) fn apply(&self, mods: Mods) -> Mods {
        if !self.is_remapped(mods) {
            return mods;
        }

        let mods_binding = mods.keys().int();
        let mods_sides = mod_sides_bits(mods.sides);
        for (from, to) in &self.map {
            let from_binding = from.keys().int();
            if mods_binding & from_binding != from_binding {
                continue;
            }

            let from_sides = mod_sides_bits(from.sides);
            if (mods_sides ^ from_sides) & from_binding != 0 {
                continue;
            }

            let mods_int = (mods.int() & !from.int()) | to.int();
            return Mods::from_int(mods_int);
        }

        unreachable!("RemapSet mask matched but no mapping applied");
    }

    pub(crate) fn format_entries(&self) -> Vec<String> {
        if self.map.is_empty() {
            return vec![String::new()];
        }

        self.map
            .iter()
            .map(|(from, to)| format!("{}={}", format_remap_mod(*from), format_remap_mod(*to)))
            .collect()
    }

    fn put_mapping(&mut self, from: Mods, to: Mods) {
        if let Some((_, existing_to)) = self
            .map
            .iter_mut()
            .find(|(existing_from, _)| *existing_from == from)
        {
            *existing_to = to;
        } else {
            self.map.push((from, to));
        }
        self.mask.update(from);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RemapMask {
    keys: u8,
    left_sides: u8,
    right_sides: u8,
}

impl RemapMask {
    fn update(&mut self, mods: Mods) {
        let keys = mods.keys().int();
        let sides = mod_sides_bits(mods.sides);

        self.keys |= keys;
        self.left_sides |= keys & !sides;
        self.right_sides |= keys & sides;
    }

    fn matches(self, mods: Mods) -> bool {
        let active = mods.keys().int() & self.keys;
        if active == 0 {
            return false;
        }

        let sides = mod_sides_bits(mods.sides);
        let side_match = (!sides & self.left_sides) | (sides & self.right_sides);
        active & side_match != 0
    }
}

fn init_mods(modifier: Mod, side: Side) -> Mods {
    Mods::for_mod(modifier, side)
}

fn parse_remap_mod(input: &str) -> Result<(Mod, Option<Side>), RemapSetParseError> {
    let (side_str, mod_str) = input
        .split_once('_')
        .map_or(("", input), |(side, modifier)| (side, modifier));

    let modifier = match mod_str {
        "shift" => Mod::Shift,
        "ctrl" | "control" => Mod::Ctrl,
        "alt" | "opt" | "option" => Mod::Alt,
        "super" | "cmd" | "command" => Mod::Super,
        _ => return Err(RemapSetParseError::InvalidMod),
    };

    let side = match side_str {
        "" => None,
        "left" => Some(Side::Left),
        "right" => Some(Side::Right),
        _ => return Err(RemapSetParseError::InvalidMod),
    };

    Ok((modifier, side))
}

fn format_remap_mod(mods: Mods) -> String {
    for (name, active, side) in [
        ("shift", mods.shift, mods.sides.shift),
        ("ctrl", mods.ctrl, mods.sides.ctrl),
        ("alt", mods.alt, mods.sides.alt),
        ("super", mods.super_, mods.sides.super_),
    ] {
        if active {
            let side = match side {
                Side::Left => "left",
                Side::Right => "right",
            };
            return format!("{side}_{name}");
        }
    }
    String::new()
}

fn remap_mods_has_right_side(mods: Mods) -> bool {
    mods.int() & MODS_SIDE_MASK != 0
}

fn mod_sides_bits(sides: ModSides) -> u8 {
    ((sides.int() >> 6) & 0x0f) as u8
}

const MODS_SIDE_MASK: u16 = (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mods_layout_matches_upstream_examples() {
        assert_eq!(Mods::new().int(), 0);
        assert_eq!(
            Mods {
                shift: true,
                ..Mods::new()
            }
            .int(),
            0b0000_0001
        );
        assert_eq!(Mods::for_mod(Mod::Shift, Side::Right).int(), 0b0100_0001);
        assert_eq!(Mods::for_mod(Mod::Alt, Side::Right).int(), 0b1_0000_0100);
    }

    #[test]
    fn key_mods_helpers_match_upstream_shape() {
        let mods = Mods {
            shift: true,
            alt: true,
            caps_lock: true,
            num_lock: true,
            sides: ModSides {
                alt: Side::Right,
                ..ModSides::default()
            },
            ..Mods::new()
        };

        assert!(!mods.empty());
        assert_eq!(mods.keys().int(), 0b0101);
        assert_eq!(
            mods.binding(),
            Mods {
                shift: true,
                alt: true,
                ..Mods::new()
            }
        );
        assert_eq!(
            mods.without_locks(),
            Mods {
                caps_lock: false,
                num_lock: false,
                ..mods
            }
        );
        assert_eq!(
            mods.unset(Mods {
                shift: true,
                ..Mods::new()
            }),
            Mods {
                alt: true,
                caps_lock: true,
                num_lock: true,
                sides: ModSides {
                    alt: Side::Right,
                    ..ModSides::default()
                },
                ..Mods::new()
            }
        );
    }

    #[test]
    fn key_mods_translation_macos_option_as_alt() {
        let left_alt = Mods::for_mod(Mod::Alt, Side::Left);
        let right_alt = Mods::for_mod(Mod::Alt, Side::Right);

        assert_eq!(left_alt.translation(OptionAsAlt::False), left_alt);
        assert!(!left_alt.translation(OptionAsAlt::True).alt);
        assert!(!left_alt.translation(OptionAsAlt::Left).alt);
        assert_eq!(left_alt.translation(OptionAsAlt::Right), left_alt);
        assert_eq!(right_alt.translation(OptionAsAlt::Left), right_alt);
        assert!(!right_alt.translation(OptionAsAlt::Right).alt);

        let shifted_alt = Mods {
            shift: true,
            ..left_alt
        };
        assert_eq!(
            shifted_alt.translation(OptionAsAlt::True),
            Mods {
                shift: true,
                ..Mods::new()
            }
        );
    }

    #[test]
    fn key_mods_ctrl_or_super_is_macos_super() {
        assert!(Mods {
            super_: true,
            ..Mods::new()
        }
        .ctrl_or_super());
        assert!(!Mods {
            ctrl: true,
            ..Mods::new()
        }
        .ctrl_or_super());
        assert_eq!(
            ctrl_or_super(Mods::new()),
            Mods {
                super_: true,
                ..Mods::new()
            }
        );
    }

    fn remap(modifier: Mod, side: Side) -> Mods {
        Mods::for_mod(modifier, side)
    }

    fn remap_with_extra(base: Mods, extra: Mods) -> Mods {
        Mods::from_int(base.int() | extra.int())
    }

    #[test]
    fn key_remap_set_unsided_remap_creates_both_sides() {
        let mut set = RemapSet::default();
        set.parse("ctrl=super").unwrap();
        set.finalize();

        let left_ctrl = remap(Mod::Ctrl, Side::Left);
        let right_ctrl = remap(Mod::Ctrl, Side::Right);
        let left_super = remap(Mod::Super, Side::Left);

        assert_eq!(set.apply(left_ctrl), left_super);
        assert_eq!(set.apply(right_ctrl), left_super);
        assert!(set.is_remapped(left_ctrl));
        assert!(set.is_remapped(right_ctrl));
    }

    #[test]
    fn key_remap_set_sided_from_maps_only_that_side() {
        let mut set = RemapSet::default();
        set.parse("left_ctrl=super").unwrap();
        set.finalize();

        let left_ctrl = remap(Mod::Ctrl, Side::Left);
        let right_ctrl = remap(Mod::Ctrl, Side::Right);
        let left_super = remap(Mod::Super, Side::Left);

        assert_eq!(set.apply(left_ctrl), left_super);
        assert_eq!(set.apply(right_ctrl), right_ctrl);
        assert!(set.is_remapped(left_ctrl));
        assert!(!set.is_remapped(right_ctrl));
    }

    #[test]
    fn key_remap_set_sided_to_preserves_target_side() {
        let mut set = RemapSet::default();
        set.parse("ctrl=right_super").unwrap();
        set.finalize();

        let left_ctrl = remap(Mod::Ctrl, Side::Left);
        let right_super = remap(Mod::Super, Side::Right);

        assert_eq!(set.apply(left_ctrl), right_super);
    }

    #[test]
    fn key_remap_set_multiple_parses_accumulate_and_aliases_match_upstream() {
        let mut set = RemapSet::default();
        set.parse("control=command").unwrap();
        set.parse("opt=cmd").unwrap();
        set.parse("right_option=left_control").unwrap();
        set.finalize();

        assert_eq!(
            set.apply(remap(Mod::Ctrl, Side::Left)),
            remap(Mod::Super, Side::Left)
        );
        assert_eq!(
            set.apply(remap(Mod::Alt, Side::Left)),
            remap(Mod::Super, Side::Left)
        );
        assert_eq!(
            set.apply(remap(Mod::Alt, Side::Right)),
            remap(Mod::Ctrl, Side::Left)
        );
    }

    #[test]
    fn key_remap_set_parse_cli_empty_resets_and_errors_match_upstream_shape() {
        let mut set = RemapSet::default();
        set.parse("ctrl=super").unwrap();
        assert!(set.is_remapped(remap(Mod::Ctrl, Side::Left)));

        set.parse_cli(Some("")).unwrap();
        assert!(!set.is_remapped(remap(Mod::Ctrl, Side::Left)));
        assert_eq!(set.format_entries(), vec![String::new()]);

        assert_eq!(
            set.parse("ctrl"),
            Err(RemapSetParseError::MissingAssignment)
        );
        assert_eq!(set.parse("hyper=ctrl"), Err(RemapSetParseError::InvalidMod));
        assert_eq!(
            set.parse("middle_ctrl=alt"),
            Err(RemapSetParseError::InvalidMod)
        );
    }

    #[test]
    fn key_remap_set_finalize_orders_right_side_mappings_first() {
        let mut set = RemapSet::default();
        set.parse("ctrl=super").unwrap();
        set.parse("right_ctrl=alt").unwrap();

        set.finalize();
        assert_eq!(
            set.apply(remap(Mod::Ctrl, Side::Right)),
            remap(Mod::Alt, Side::Left)
        );
        assert_eq!(
            set.format_entries(),
            vec![
                "right_ctrl=left_alt".to_string(),
                "left_ctrl=left_super".to_string(),
            ]
        );
    }

    #[test]
    fn key_remap_set_apply_is_one_way_non_transitive_and_preserves_other_mods() {
        let mut set = RemapSet::default();
        set.parse("ctrl=super").unwrap();
        set.parse("super=alt").unwrap();
        set.finalize();

        let ctrl_alt = remap_with_extra(remap(Mod::Ctrl, Side::Left), remap(Mod::Alt, Side::Left));
        let expected = remap_with_extra(remap(Mod::Super, Side::Left), remap(Mod::Alt, Side::Left));
        assert_eq!(set.apply(ctrl_alt), expected);
        assert_eq!(
            set.apply(remap(Mod::Ctrl, Side::Left)),
            remap(Mod::Super, Side::Left)
        );
    }

    #[test]
    fn key_remap_set_clone_equality_and_formatter_are_deterministic() {
        let mut set = RemapSet::default();
        set.parse("left_ctrl=left_super").unwrap();
        set.parse("alt=right_ctrl").unwrap();
        set.finalize();

        let cloned = set.clone();
        assert_eq!(cloned, set);
        assert_eq!(
            cloned.format_entries(),
            vec![
                "right_alt=right_ctrl".to_string(),
                "left_ctrl=left_super".to_string(),
                "left_alt=right_ctrl".to_string(),
            ]
        );
    }

    #[test]
    fn key_remap_set_equality_is_order_independent() {
        let mut a = RemapSet::default();
        a.parse("left_ctrl=left_super").unwrap();
        a.parse("left_alt=right_ctrl").unwrap();
        a.finalize();

        let mut b = RemapSet::default();
        b.parse("left_alt=right_ctrl").unwrap();
        b.parse("left_ctrl=left_super").unwrap();
        b.finalize();

        assert_eq!(a, b);

        b.parse("left_shift=left_alt").unwrap();
        b.finalize();

        assert_ne!(a, b);
    }
}
