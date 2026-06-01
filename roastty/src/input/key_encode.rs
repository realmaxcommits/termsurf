use super::key::{Key, KeyAction, KeyEvent};
use super::key_mods::{Mods, OptionAsAlt, Side};
use crate::terminal::kitty::KeyFlags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    pub(crate) cursor_key_application: bool,
    pub(crate) keypad_key_application: bool,
    pub(crate) backarrow_key_mode: bool,
    pub(crate) ignore_keypad_with_numlock: bool,
    pub(crate) alt_esc_prefix: bool,
    pub(crate) modify_other_keys_state_2: bool,
    pub(crate) kitty_flags: KeyFlags,
    pub(crate) macos_option_as_alt: OptionAsAlt,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cursor_key_application: false,
            keypad_key_application: false,
            backarrow_key_mode: false,
            ignore_keypad_with_numlock: false,
            alt_esc_prefix: false,
            modify_other_keys_state_2: false,
            kitty_flags: KeyFlags::DISABLED,
            macos_option_as_alt: OptionAsAlt::False,
        }
    }
}

pub(crate) fn encode(event: &KeyEvent, opts: Options) -> Vec<u8> {
    let mut output = String::new();
    if opts.kitty_flags.int() != 0 {
        kitty(&mut output, event, opts);
    } else {
        legacy(&mut output, event, opts);
    }
    output.into_bytes()
}

fn kitty(output: &mut String, event: &KeyEvent, opts: Options) {
    if opts.kitty_flags.is_disabled() {
        legacy(output, event, opts);
        return;
    }

    if event.action == KeyAction::Release {
        if !opts.kitty_flags.report_events {
            return;
        }
        if !opts.kitty_flags.report_all
            && matches!(event.key, Key::Enter | Key::Backspace | Key::Tab)
        {
            return;
        }
    }

    let all_mods = event.mods;
    let binding_mods = event.effective_mods().binding();
    let entry = kitty_entry(event).or_else(|| {
        if event.unshifted_codepoint > 0 {
            Some(KittyEntry {
                key: event.key,
                code: event.unshifted_codepoint,
                final_byte: 'u',
                modifier: false,
            })
        } else {
            None
        }
    });

    if event.composing {
        if !entry.is_some_and(|entry| entry.modifier) {
            return;
        }
    } else if !opts.kitty_flags.report_all {
        if event.utf8.is_empty() && binding_mods.empty() {
            match event.key {
                Key::Enter => {
                    output.push('\r');
                    return;
                }
                Key::Tab => {
                    output.push('\t');
                    return;
                }
                Key::Backspace => {
                    output.push('\u{7f}');
                    return;
                }
                _ => {}
            }
        }

        if !event.utf8.is_empty() && binding_mods.empty() && event.action != KeyAction::Release {
            if event.utf8.iter().all(|byte| !is_control(*byte as u32)) {
                push_utf8(output, &event.utf8);
                return;
            }
        }
    } else if !event.utf8.is_empty()
        && matches!(event.key, Key::Enter | Key::Backspace)
        && !is_control_utf8(&event.utf8)
    {
        if event.key == Key::Enter {
            push_utf8(output, &event.utf8);
        }
        return;
    }

    let Some(entry) = entry else {
        if !event.utf8.is_empty() {
            push_utf8(output, &event.utf8);
        }
        return;
    };
    if entry.modifier && !opts.kitty_flags.report_all {
        return;
    }

    let mut seq = KittySequence {
        key: entry.code,
        final_byte: entry.final_byte,
        mods: KittyMods::from_input(event.action, event.key, all_mods),
        event: KittyEvent::None,
        alternates: [None, None],
        text: Vec::new(),
    };

    if opts.kitty_flags.report_events {
        seq.event = match event.action {
            KeyAction::Press => KittyEvent::Press,
            KeyAction::Repeat => KittyEvent::Repeat,
            KeyAction::Release => KittyEvent::Release,
        };
    }

    if opts.kitty_flags.report_alternates && !is_control(seq.key) {
        let chars = utf8_chars(&event.utf8);
        if let Some(cp1) = chars.first().copied() {
            if cp1 != seq.key && seq.mods.shift {
                seq.alternates[0] = Some(cp1);
            }
            if chars.len() == 1 {
                if let Some(base) = event.key.codepoint() {
                    if base != seq.key && cp1 != base {
                        seq.alternates[1] = Some(base);
                    }
                }
            }
        } else if let Some(base) = event.key.codepoint() {
            if base != seq.key {
                seq.alternates[1] = Some(base);
            }
        }
    }

    if opts.kitty_flags.report_associated && seq.event != KittyEvent::Release {
        let alt_prevents_text = match opts.macos_option_as_alt {
            OptionAsAlt::False => false,
            OptionAsAlt::True => true,
            OptionAsAlt::Left => all_mods.sides.alt == Side::Left,
            OptionAsAlt::Right => all_mods.sides.alt == Side::Right,
        };
        if !seq.mods.prevents_text(alt_prevents_text) {
            seq.text = event.utf8.clone();
        }
    }

    seq.encode(output);
}

fn legacy(output: &mut String, event: &KeyEvent, opts: Options) {
    if event.action != KeyAction::Press && event.action != KeyAction::Repeat {
        return;
    }
    if event.composing {
        return;
    }

    let all_mods = event.mods;
    let binding_mods = event.effective_mods().binding();

    if let Some(sequence) = pc_style_function_key(event.key, all_mods, opts) {
        if !event.utf8.is_empty()
            && matches!(event.key, Key::Backspace | Key::Enter | Key::Escape)
            && !is_control_utf8(&event.utf8)
        {
            if event.key == Key::Backspace {
                return;
            }
        } else {
            output.push_str(&sequence);
            return;
        }
    }

    if let Some(byte) = ctrl_seq(event.key, &event.utf8, event.unshifted_codepoint, all_mods) {
        if binding_mods.alt {
            output.push('\x1b');
        }
        output.push(byte as char);
        return;
    }

    if event.utf8.is_empty() {
        if let Some(byte) = legacy_alt_prefix(event, binding_mods, all_mods, opts) {
            output.push('\x1b');
            output.push(byte as char);
        }
        return;
    }

    if opts.modify_other_keys_state_2 {
        let chars = utf8_chars(&event.utf8);
        if chars.len() == 1 {
            let codepoint = chars[0];
            let mut mods = event.mods.binding();
            match opts.macos_option_as_alt {
                OptionAsAlt::False => mods.alt = false,
                OptionAsAlt::True => {}
                OptionAsAlt::Left if event.mods.sides.alt == Side::Right => mods.alt = false,
                OptionAsAlt::Right if event.mods.sides.alt == Side::Left => mods.alt = false,
                OptionAsAlt::Left | OptionAsAlt::Right => {}
            }
            let mut mods_no_shift = mods;
            mods_no_shift.shift = false;
            let should_modify = (0x40..=0x7f).contains(&codepoint)
                || !mods_no_shift.empty()
                || codepoint == ' ' as u32;
            if should_modify {
                if let Some(code) = modifier_code(mods) {
                    output.push_str(&format!("\x1b[27;{code};{codepoint}~"));
                    return;
                }
            }
        }
    }

    if event.mods.ctrl {
        if let Some((mods, cp)) = csi_u_for_ctrl(event) {
            output.push_str(&format!("\x1b[{cp};{}u", csi_u_seq(mods)));
            return;
        }
    }

    if let Some(byte) = legacy_alt_prefix(event, binding_mods, all_mods, opts) {
        output.push('\x1b');
        output.push(byte as char);
        return;
    }

    if all_mods.super_ {
        return;
    }

    push_utf8(output, &event.utf8);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KittyEntry {
    key: Key,
    code: u32,
    final_byte: char,
    modifier: bool,
}

fn kitty_entry(event: &KeyEvent) -> Option<KittyEntry> {
    kitty_entry_for_key(event.key)
}

fn kitty_entry_for_key(key: Key) -> Option<KittyEntry> {
    KITTY_ENTRIES.iter().copied().find(|entry| entry.key == key)
}

const KITTY_ENTRIES: &[KittyEntry] = &[
    KittyEntry {
        key: Key::Escape,
        code: 27,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Enter,
        code: 13,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Tab,
        code: 9,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Backspace,
        code: 127,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Insert,
        code: 2,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::Delete,
        code: 3,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::ArrowLeft,
        code: 1,
        final_byte: 'D',
        modifier: false,
    },
    KittyEntry {
        key: Key::ArrowRight,
        code: 1,
        final_byte: 'C',
        modifier: false,
    },
    KittyEntry {
        key: Key::ArrowUp,
        code: 1,
        final_byte: 'A',
        modifier: false,
    },
    KittyEntry {
        key: Key::ArrowDown,
        code: 1,
        final_byte: 'B',
        modifier: false,
    },
    KittyEntry {
        key: Key::PageUp,
        code: 5,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::PageDown,
        code: 6,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::Home,
        code: 1,
        final_byte: 'H',
        modifier: false,
    },
    KittyEntry {
        key: Key::End,
        code: 1,
        final_byte: 'F',
        modifier: false,
    },
    KittyEntry {
        key: Key::CapsLock,
        code: 57358,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::ScrollLock,
        code: 57359,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumLock,
        code: 57360,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::PrintScreen,
        code: 57361,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Pause,
        code: 57362,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F1,
        code: 1,
        final_byte: 'P',
        modifier: false,
    },
    KittyEntry {
        key: Key::F2,
        code: 1,
        final_byte: 'Q',
        modifier: false,
    },
    KittyEntry {
        key: Key::F3,
        code: 13,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F4,
        code: 1,
        final_byte: 'S',
        modifier: false,
    },
    KittyEntry {
        key: Key::F5,
        code: 15,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F6,
        code: 17,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F7,
        code: 18,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F8,
        code: 19,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F9,
        code: 20,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F10,
        code: 21,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F11,
        code: 23,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F12,
        code: 24,
        final_byte: '~',
        modifier: false,
    },
    KittyEntry {
        key: Key::F13,
        code: 57376,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F14,
        code: 57377,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F15,
        code: 57378,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F16,
        code: 57379,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F17,
        code: 57380,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F18,
        code: 57381,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F19,
        code: 57382,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F20,
        code: 57383,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F21,
        code: 57384,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F22,
        code: 57385,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F23,
        code: 57386,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F24,
        code: 57387,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::F25,
        code: 57388,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad0,
        code: 57399,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad1,
        code: 57400,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad2,
        code: 57401,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad3,
        code: 57402,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad4,
        code: 57403,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad5,
        code: 57404,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad6,
        code: 57405,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad7,
        code: 57406,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad8,
        code: 57407,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::Numpad9,
        code: 57408,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadDecimal,
        code: 57409,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadDivide,
        code: 57410,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadMultiply,
        code: 57411,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadSubtract,
        code: 57412,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadAdd,
        code: 57413,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadEnter,
        code: 57414,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadEqual,
        code: 57415,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadSeparator,
        code: 57416,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadLeft,
        code: 57417,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadRight,
        code: 57418,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadUp,
        code: 57419,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadDown,
        code: 57420,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadPageUp,
        code: 57421,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadPageDown,
        code: 57422,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadHome,
        code: 57423,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadEnd,
        code: 57424,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadInsert,
        code: 57425,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadDelete,
        code: 57426,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::NumpadBegin,
        code: 57427,
        final_byte: 'u',
        modifier: false,
    },
    KittyEntry {
        key: Key::ShiftLeft,
        code: 57441,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::ShiftRight,
        code: 57447,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::ControlLeft,
        code: 57442,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::ControlRight,
        code: 57448,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::MetaLeft,
        code: 57444,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::MetaRight,
        code: 57450,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::AltLeft,
        code: 57443,
        final_byte: 'u',
        modifier: true,
    },
    KittyEntry {
        key: Key::AltRight,
        code: 57449,
        final_byte: 'u',
        modifier: true,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KittyEvent {
    None,
    Press,
    Repeat,
    Release,
}

impl KittyEvent {
    fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Press => 1,
            Self::Repeat => 2,
            Self::Release => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KittyMods {
    shift: bool,
    alt: bool,
    ctrl: bool,
    super_: bool,
    caps_lock: bool,
    num_lock: bool,
}

impl KittyMods {
    fn from_input(_action: KeyAction, _key: Key, mods: Mods) -> Self {
        Self {
            shift: mods.shift,
            alt: mods.alt,
            ctrl: mods.ctrl,
            super_: mods.super_,
            caps_lock: mods.caps_lock,
            num_lock: mods.num_lock,
        }
    }

    fn int(self) -> u8 {
        self.shift as u8
            | ((self.alt as u8) << 1)
            | ((self.ctrl as u8) << 2)
            | ((self.super_ as u8) << 3)
            | ((self.caps_lock as u8) << 6)
            | ((self.num_lock as u8) << 7)
    }

    fn seq_int(self) -> u16 {
        self.int() as u16 + 1
    }

    fn prevents_text(self, alt_prevents_text: bool) -> bool {
        (self.alt && alt_prevents_text) || self.ctrl || self.super_
    }
}

struct KittySequence {
    key: u32,
    final_byte: char,
    mods: KittyMods,
    event: KittyEvent,
    alternates: [Option<u32>; 2],
    text: Vec<u8>,
}

impl KittySequence {
    fn encode(self, output: &mut String) {
        if self.final_byte == 'u' || self.final_byte == '~' {
            self.encode_full(output);
        } else {
            self.encode_special(output);
        }
    }

    fn encode_full(self, output: &mut String) {
        output.push_str(&format!("\x1b[{}", self.key));
        if let Some(shifted) = self.alternates[0] {
            output.push_str(&format!(":{shifted}"));
        }
        if let Some(base) = self.alternates[1] {
            if self.alternates[0].is_none() {
                output.push_str("::");
            } else {
                output.push(':');
            }
            output.push_str(&base.to_string());
        }

        let mods = self.mods.seq_int();
        let mut emitted_prior = false;
        if self.event != KittyEvent::None && self.event != KittyEvent::Press {
            output.push_str(&format!(";{}:{}", mods, self.event.code()));
            emitted_prior = true;
        } else if mods > 1 {
            output.push_str(&format!(";{mods}"));
            emitted_prior = true;
        }

        let text_chars = utf8_chars(&self.text);
        let printable: Vec<u32> = text_chars
            .into_iter()
            .filter(|cp| !is_control(*cp))
            .collect();
        if !printable.is_empty() {
            if !emitted_prior {
                output.push(';');
            }
            output.push(';');
            for (idx, cp) in printable.into_iter().enumerate() {
                if idx > 0 {
                    output.push(':');
                }
                output.push_str(&cp.to_string());
            }
        }

        output.push(self.final_byte);
    }

    fn encode_special(self, output: &mut String) {
        let mods = self.mods.seq_int();
        match self.event {
            KittyEvent::None if mods == 1 => {
                output.push_str("\x1b[");
                output.push(self.final_byte);
            }
            KittyEvent::None => {
                output.push_str(&format!("\x1b[1;{mods}{}", self.final_byte));
            }
            event => {
                output.push_str(&format!(
                    "\x1b[1;{}:{}{}",
                    mods,
                    event.code(),
                    self.final_byte
                ));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CursorMode {
    Normal,
    Application,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PcKeyKind {
    Cursor {
        normal: &'static str,
        application: &'static str,
        final_byte: char,
    },
    Tilde {
        number: u8,
        normal: &'static str,
    },
    Function {
        normal: &'static str,
        modifier_number: u8,
        final_byte: char,
    },
    Keypad {
        suffix: char,
        fallback: Option<&'static str>,
    },
    Special,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PcKeySpec {
    key: Key,
    kind: PcKeyKind,
}

fn pc_style_function_key(key: Key, mods: Mods, opts: Options) -> Option<String> {
    let mods = mods.binding();
    let spec = pc_key_spec(key)?;

    match spec.kind {
        PcKeyKind::Special => pc_special_key(key, mods, opts),
        PcKeyKind::Cursor {
            normal,
            application,
            final_byte,
        } => pc_modified_csi(mods, 1, final_byte).or_else(|| {
            Some(match cursor_mode(opts) {
                CursorMode::Normal => normal.to_string(),
                CursorMode::Application => application.to_string(),
            })
        }),
        PcKeyKind::Tilde { number, normal } => pc_modified_tilde(mods, number)
            .or_else(|| (!mods.empty()).then(|| None).flatten())
            .or_else(|| Some(normal.to_string())),
        PcKeyKind::Function {
            normal,
            modifier_number,
            final_byte,
        } => pc_modified_function(mods, modifier_number, final_byte)
            .or_else(|| Some(normal.to_string())),
        PcKeyKind::Keypad { suffix, fallback } => pc_keypad_key(mods, opts, suffix, fallback),
    }
}

fn pc_key_spec(key: Key) -> Option<PcKeySpec> {
    PC_KEY_SPECS.iter().copied().find(|spec| spec.key == key)
}

const PC_KEY_SPECS: &[PcKeySpec] = &[
    PcKeySpec {
        key: Key::ArrowUp,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[A",
            application: "\x1bOA",
            final_byte: 'A',
        },
    },
    PcKeySpec {
        key: Key::ArrowDown,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[B",
            application: "\x1bOB",
            final_byte: 'B',
        },
    },
    PcKeySpec {
        key: Key::ArrowRight,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[C",
            application: "\x1bOC",
            final_byte: 'C',
        },
    },
    PcKeySpec {
        key: Key::ArrowLeft,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[D",
            application: "\x1bOD",
            final_byte: 'D',
        },
    },
    PcKeySpec {
        key: Key::Home,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[H",
            application: "\x1bOH",
            final_byte: 'H',
        },
    },
    PcKeySpec {
        key: Key::End,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[F",
            application: "\x1bOF",
            final_byte: 'F',
        },
    },
    PcKeySpec {
        key: Key::Insert,
        kind: PcKeyKind::Tilde {
            number: 2,
            normal: "\x1b[2~",
        },
    },
    PcKeySpec {
        key: Key::Delete,
        kind: PcKeyKind::Tilde {
            number: 3,
            normal: "\x1b[3~",
        },
    },
    PcKeySpec {
        key: Key::PageUp,
        kind: PcKeyKind::Tilde {
            number: 5,
            normal: "\x1b[5~",
        },
    },
    PcKeySpec {
        key: Key::PageDown,
        kind: PcKeyKind::Tilde {
            number: 6,
            normal: "\x1b[6~",
        },
    },
    PcKeySpec {
        key: Key::F1,
        kind: PcKeyKind::Function {
            normal: "\x1bOP",
            modifier_number: 1,
            final_byte: 'P',
        },
    },
    PcKeySpec {
        key: Key::F2,
        kind: PcKeyKind::Function {
            normal: "\x1bOQ",
            modifier_number: 1,
            final_byte: 'Q',
        },
    },
    PcKeySpec {
        key: Key::F3,
        kind: PcKeyKind::Function {
            normal: "\x1bOR",
            modifier_number: 13,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F4,
        kind: PcKeyKind::Function {
            normal: "\x1bOS",
            modifier_number: 1,
            final_byte: 'S',
        },
    },
    PcKeySpec {
        key: Key::F5,
        kind: PcKeyKind::Function {
            normal: "\x1b[15~",
            modifier_number: 15,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F6,
        kind: PcKeyKind::Function {
            normal: "\x1b[17~",
            modifier_number: 17,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F7,
        kind: PcKeyKind::Function {
            normal: "\x1b[18~",
            modifier_number: 18,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F8,
        kind: PcKeyKind::Function {
            normal: "\x1b[19~",
            modifier_number: 19,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F9,
        kind: PcKeyKind::Function {
            normal: "\x1b[20~",
            modifier_number: 20,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F10,
        kind: PcKeyKind::Function {
            normal: "\x1b[21~",
            modifier_number: 21,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F11,
        kind: PcKeyKind::Function {
            normal: "\x1b[23~",
            modifier_number: 23,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::F12,
        kind: PcKeyKind::Function {
            normal: "\x1b[24~",
            modifier_number: 24,
            final_byte: '~',
        },
    },
    PcKeySpec {
        key: Key::Numpad0,
        kind: PcKeyKind::Keypad {
            suffix: 'p',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad1,
        kind: PcKeyKind::Keypad {
            suffix: 'q',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad2,
        kind: PcKeyKind::Keypad {
            suffix: 'r',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad3,
        kind: PcKeyKind::Keypad {
            suffix: 's',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad4,
        kind: PcKeyKind::Keypad {
            suffix: 't',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad5,
        kind: PcKeyKind::Keypad {
            suffix: 'u',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad6,
        kind: PcKeyKind::Keypad {
            suffix: 'v',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad7,
        kind: PcKeyKind::Keypad {
            suffix: 'w',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad8,
        kind: PcKeyKind::Keypad {
            suffix: 'x',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::Numpad9,
        kind: PcKeyKind::Keypad {
            suffix: 'y',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadDecimal,
        kind: PcKeyKind::Keypad {
            suffix: 'n',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadDivide,
        kind: PcKeyKind::Keypad {
            suffix: 'o',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadMultiply,
        kind: PcKeyKind::Keypad {
            suffix: 'j',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadSubtract,
        kind: PcKeyKind::Keypad {
            suffix: 'm',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadAdd,
        kind: PcKeyKind::Keypad {
            suffix: 'k',
            fallback: None,
        },
    },
    PcKeySpec {
        key: Key::NumpadEnter,
        kind: PcKeyKind::Keypad {
            suffix: 'M',
            fallback: Some("\r"),
        },
    },
    PcKeySpec {
        key: Key::NumpadUp,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[A",
            application: "\x1bOA",
            final_byte: 'A',
        },
    },
    PcKeySpec {
        key: Key::NumpadDown,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[B",
            application: "\x1bOB",
            final_byte: 'B',
        },
    },
    PcKeySpec {
        key: Key::NumpadRight,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[C",
            application: "\x1bOC",
            final_byte: 'C',
        },
    },
    PcKeySpec {
        key: Key::NumpadLeft,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[D",
            application: "\x1bOD",
            final_byte: 'D',
        },
    },
    PcKeySpec {
        key: Key::NumpadBegin,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[E",
            application: "\x1bOE",
            final_byte: 'E',
        },
    },
    PcKeySpec {
        key: Key::NumpadHome,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[H",
            application: "\x1bOH",
            final_byte: 'H',
        },
    },
    PcKeySpec {
        key: Key::NumpadEnd,
        kind: PcKeyKind::Cursor {
            normal: "\x1b[F",
            application: "\x1bOF",
            final_byte: 'F',
        },
    },
    PcKeySpec {
        key: Key::NumpadInsert,
        kind: PcKeyKind::Tilde {
            number: 2,
            normal: "\x1b[2~",
        },
    },
    PcKeySpec {
        key: Key::NumpadDelete,
        kind: PcKeyKind::Tilde {
            number: 3,
            normal: "\x1b[3~",
        },
    },
    PcKeySpec {
        key: Key::NumpadPageUp,
        kind: PcKeyKind::Tilde {
            number: 5,
            normal: "\x1b[5~",
        },
    },
    PcKeySpec {
        key: Key::NumpadPageDown,
        kind: PcKeyKind::Tilde {
            number: 6,
            normal: "\x1b[6~",
        },
    },
    PcKeySpec {
        key: Key::Backspace,
        kind: PcKeyKind::Special,
    },
    PcKeySpec {
        key: Key::Tab,
        kind: PcKeyKind::Special,
    },
    PcKeySpec {
        key: Key::Enter,
        kind: PcKeyKind::Special,
    },
    PcKeySpec {
        key: Key::Escape,
        kind: PcKeyKind::Special,
    },
];

fn cursor_mode(opts: Options) -> CursorMode {
    if opts.cursor_key_application {
        CursorMode::Application
    } else {
        CursorMode::Normal
    }
}

fn pc_modified_csi(mods: Mods, number: u8, final_byte: char) -> Option<String> {
    if mods.empty() {
        return None;
    }
    modifier_code(mods).map(|code| format!("\x1b[{number};{code}{final_byte}"))
}

fn pc_modified_tilde(mods: Mods, number: u8) -> Option<String> {
    if mods.empty() {
        return None;
    }
    modifier_code(mods).map(|code| format!("\x1b[{number};{code}~"))
}

fn pc_modified_function(mods: Mods, number: u8, final_byte: char) -> Option<String> {
    if final_byte == '~' {
        pc_modified_tilde(mods, number)
    } else {
        pc_modified_csi(mods, number, final_byte)
    }
}

fn pc_keypad_key(
    mods: Mods,
    opts: Options,
    suffix: char,
    fallback: Option<&'static str>,
) -> Option<String> {
    let keypad_application = opts.keypad_key_application && !opts.ignore_keypad_with_numlock;
    if keypad_application {
        if mods.empty() {
            return Some(format!("\x1bO{suffix}"));
        }
        return modifier_code(mods).map(|code| format!("\x1bO{code}{suffix}"));
    }

    fallback.map(str::to_string)
}

fn pc_special_key(key: Key, mods: Mods, opts: Options) -> Option<String> {
    match key {
        Key::Backspace => pc_backspace(mods, opts),
        Key::Tab => pc_tab(mods, opts),
        Key::Enter => pc_enter(mods, opts),
        Key::Escape => pc_escape(mods),
        _ => None,
    }
}

fn pc_backspace(mods: Mods, opts: Options) -> Option<String> {
    if opts.modify_other_keys_state_2 {
        if let Some(code) = modifier_code(mods) {
            if code != 5 {
                return Some(format!("\x1b[27;{code};127~"));
            }
        }
    } else {
        match (mods.shift, mods.alt, mods.ctrl, mods.super_) {
            (true, false, false, false) => return Some("\x7f".to_string()),
            (false, true, false, false) | (true, true, false, false) => {
                return Some("\x1b\x7f".to_string());
            }
            (true, false, true, false) => return Some("\x08".to_string()),
            (false, true, true, false) => return Some("\x1b\x08".to_string()),
            (false, false, false, true) | (true, false, false, true) => {
                return Some("\x7f".to_string());
            }
            (false, true, false, true) | (true, true, false, true) => {
                return Some("\x1b\x7f".to_string());
            }
            (false, false, true, true) | (true, false, true, true) => {
                return Some("\x08".to_string());
            }
            (false, true, true, true) | (true, true, true, true) => {
                return Some("\x1b\x08".to_string());
            }
            _ => {}
        }
    }

    match (opts.backarrow_key_mode, mods.ctrl) {
        (false, false) => Some("\x7f".to_string()),
        (false, true) => Some("\x08".to_string()),
        (true, false) => Some("\x08".to_string()),
        (true, true) => Some("\x7f".to_string()),
    }
}

fn pc_tab(mods: Mods, opts: Options) -> Option<String> {
    if opts.modify_other_keys_state_2 {
        if let Some(code) = modifier_code(mods) {
            return Some(format!("\x1b[27;{code};9~"));
        }
    } else {
        match (mods.shift, mods.alt, mods.ctrl, mods.super_) {
            (true, false, false, false) => return Some("\x1b[Z".to_string()),
            (false, true, false, false) => return Some("\x1b\t".to_string()),
            _ if !mods.empty() => {
                if let Some(code) = modifier_code(mods) {
                    return Some(format!("\x1b[27;{code};9~"));
                }
            }
            _ => {}
        }
    }
    Some("\t".to_string())
}

fn pc_enter(mods: Mods, opts: Options) -> Option<String> {
    if opts.modify_other_keys_state_2 {
        if let Some(code) = modifier_code(mods) {
            return Some(format!("\x1b[27;{code};13~"));
        }
    } else {
        match (mods.shift, mods.alt, mods.ctrl, mods.super_) {
            (true, false, false, false) => return Some("\x1b[27;2;13~".to_string()),
            (false, true, false, false) => return Some("\x1b\r".to_string()),
            _ if !mods.empty() => {
                if let Some(code) = modifier_code(mods) {
                    return Some(format!("\x1b[27;{code};13~"));
                }
            }
            _ => {}
        }
    }
    Some("\r".to_string())
}

fn pc_escape(mods: Mods) -> Option<String> {
    match (mods.shift, mods.alt, mods.ctrl, mods.super_) {
        (false, false, false, false) => Some("\x1b".to_string()),
        (false, true, false, false) => Some("\x1b\x1b".to_string()),
        _ => modifier_code(mods).map(|code| format!("\x1b[27;{code};27~")),
    }
}

fn ctrl_seq(key: Key, utf8: &[u8], unshifted_codepoint: u32, mods: Mods) -> Option<u8> {
    if !mods.ctrl {
        return None;
    }
    let mut unset_mods = mods.binding();
    unset_mods.alt = false;
    let ctrl_only = Mods {
        ctrl: true,
        ..Mods::new()
    }
    .int();

    let mut char_byte = if utf8.len() == 1 {
        utf8[0]
    } else if let Some(cp) = key.codepoint() {
        if cp <= u8::MAX as u32 {
            if unset_mods.int() == ctrl_only {
                cp as u8
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    if unset_mods.shift && !(b'A'..=b'Z').contains(&char_byte) {
        if char_byte != b'@' {
            unset_mods.shift = false;
        }
    }

    if (b'A'..=b'Z').contains(&char_byte) && unshifted_codepoint > 0 {
        if unshifted_codepoint <= u8::MAX as u32 {
            char_byte = unshifted_codepoint as u8;
        }
    }

    if unset_mods.int() != ctrl_only {
        return None;
    }

    match char_byte {
        b' ' => Some(0),
        b'/' => Some(31),
        b'0' => Some(48),
        b'1' => Some(49),
        b'2' => Some(0),
        b'3' => Some(27),
        b'4' => Some(28),
        b'5' => Some(29),
        b'6' => Some(30),
        b'7' => Some(31),
        b'8' => Some(127),
        b'9' => Some(57),
        b'?' => Some(127),
        b'@' => Some(0),
        b'\\' => Some(28),
        b']' => Some(29),
        b'^' => Some(30),
        b'_' => Some(31),
        b'a'..=b'h' => Some(char_byte - b'a' + 1),
        b'j'..=b'l' => Some(char_byte - b'a' + 1),
        b'n'..=b'z' => Some(char_byte - b'a' + 1),
        b'~' => Some(30),
        _ => None,
    }
}

fn csi_u_for_ctrl(event: &KeyEvent) -> Option<(Mods, u32)> {
    let chars = utf8_chars(&event.utf8);
    if chars.len() != 1 {
        return None;
    }
    let mut char = chars[0];
    let mut mods = event.mods;
    if (b'A' as u32..=b'Z' as u32).contains(&char) && mods.shift {
        char = char.to_ascii_lowercase();
    }
    if event.unshifted_codepoint != 0 && event.unshifted_codepoint != char {
        mods.shift = false;
    }
    Some((mods, char))
}

fn csi_u_seq(mods: Mods) -> u8 {
    1 + mods.shift as u8 + ((mods.alt as u8) << 1) + ((mods.ctrl as u8) << 2)
}

fn modifier_code(mods: Mods) -> Option<u8> {
    let mods = mods.binding();
    Some(match (mods.shift, mods.alt, mods.ctrl, mods.super_) {
        (true, false, false, false) => 2,
        (false, true, false, false) => 3,
        (true, true, false, false) => 4,
        (false, false, true, false) => 5,
        (true, false, true, false) => 6,
        (false, true, true, false) => 7,
        (true, true, true, false) => 8,
        (false, false, false, true) => 9,
        (true, false, false, true) => 10,
        (false, true, false, true) => 11,
        (true, true, false, true) => 12,
        (false, false, true, true) => 13,
        (true, false, true, true) => 14,
        (false, true, true, true) => 15,
        (true, true, true, true) => 16,
        _ => return None,
    })
}

fn legacy_alt_prefix(
    event: &KeyEvent,
    binding_mods: Mods,
    mods: Mods,
    opts: Options,
) -> Option<u8> {
    if !binding_mods.alt || !opts.alt_esc_prefix {
        return None;
    }
    match opts.macos_option_as_alt {
        OptionAsAlt::False => return None,
        OptionAsAlt::Left if mods.sides.alt == Side::Right => return None,
        OptionAsAlt::Right if mods.sides.alt == Side::Left => return None,
        OptionAsAlt::True | OptionAsAlt::Left | OptionAsAlt::Right => {}
    }

    if event.utf8.len() == 1 {
        return Some(event.utf8[0]);
    }
    if event.unshifted_codepoint > 0 && event.unshifted_codepoint <= u8::MAX as u32 {
        return Some(event.unshifted_codepoint as u8);
    }
    None
}

fn utf8_chars(bytes: &[u8]) -> Vec<u32> {
    std::str::from_utf8(bytes)
        .map(|text| text.chars().map(|ch| ch as u32).collect())
        .unwrap_or_default()
}

fn push_utf8(output: &mut String, bytes: &[u8]) {
    output.push_str(std::str::from_utf8(bytes).unwrap_or_default());
}

fn is_control_utf8(bytes: &[u8]) -> bool {
    let chars = utf8_chars(bytes);
    !chars.is_empty() && chars.iter().all(|cp| is_control(*cp))
}

fn is_control(cp: u32) -> bool {
    cp < 0x20 || cp == 0x7f
}

trait AsciiLower {
    fn to_ascii_lowercase(self) -> Self;
}

impl AsciiLower for u32 {
    fn to_ascii_lowercase(self) -> Self {
        if (b'A' as u32..=b'Z' as u32).contains(&self) {
            self + 32
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::key_mods::ModSides;

    fn event(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            ..KeyEvent::default()
        }
    }

    fn text_event(key: Key, text: &str) -> KeyEvent {
        KeyEvent {
            key,
            utf8: text.as_bytes().to_vec(),
            ..KeyEvent::default()
        }
    }

    fn encoded(event: KeyEvent, opts: Options) -> String {
        String::from_utf8(encode(&event, opts)).unwrap()
    }

    fn kitty_flags() -> KeyFlags {
        KeyFlags {
            disambiguate: true,
            ..KeyFlags::DISABLED
        }
    }

    fn kitty_all() -> KeyFlags {
        KeyFlags {
            disambiguate: true,
            report_events: true,
            report_alternates: true,
            report_all: true,
            report_associated: true,
        }
    }

    #[test]
    fn key_encode_kitty_table_covers_upstream_supported_entries() {
        let expected = [
            KittyEntry {
                key: Key::Escape,
                code: 27,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Enter,
                code: 13,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Tab,
                code: 9,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Backspace,
                code: 127,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Insert,
                code: 2,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::Delete,
                code: 3,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::ArrowLeft,
                code: 1,
                final_byte: 'D',
                modifier: false,
            },
            KittyEntry {
                key: Key::ArrowRight,
                code: 1,
                final_byte: 'C',
                modifier: false,
            },
            KittyEntry {
                key: Key::ArrowUp,
                code: 1,
                final_byte: 'A',
                modifier: false,
            },
            KittyEntry {
                key: Key::ArrowDown,
                code: 1,
                final_byte: 'B',
                modifier: false,
            },
            KittyEntry {
                key: Key::PageUp,
                code: 5,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::PageDown,
                code: 6,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::Home,
                code: 1,
                final_byte: 'H',
                modifier: false,
            },
            KittyEntry {
                key: Key::End,
                code: 1,
                final_byte: 'F',
                modifier: false,
            },
            KittyEntry {
                key: Key::CapsLock,
                code: 57358,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::ScrollLock,
                code: 57359,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumLock,
                code: 57360,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::PrintScreen,
                code: 57361,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Pause,
                code: 57362,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F1,
                code: 1,
                final_byte: 'P',
                modifier: false,
            },
            KittyEntry {
                key: Key::F2,
                code: 1,
                final_byte: 'Q',
                modifier: false,
            },
            KittyEntry {
                key: Key::F3,
                code: 13,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F4,
                code: 1,
                final_byte: 'S',
                modifier: false,
            },
            KittyEntry {
                key: Key::F5,
                code: 15,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F6,
                code: 17,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F7,
                code: 18,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F8,
                code: 19,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F9,
                code: 20,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F10,
                code: 21,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F11,
                code: 23,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F12,
                code: 24,
                final_byte: '~',
                modifier: false,
            },
            KittyEntry {
                key: Key::F13,
                code: 57376,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F14,
                code: 57377,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F15,
                code: 57378,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F16,
                code: 57379,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F17,
                code: 57380,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F18,
                code: 57381,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F19,
                code: 57382,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F20,
                code: 57383,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F21,
                code: 57384,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F22,
                code: 57385,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F23,
                code: 57386,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F24,
                code: 57387,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::F25,
                code: 57388,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad0,
                code: 57399,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad1,
                code: 57400,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad2,
                code: 57401,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad3,
                code: 57402,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad4,
                code: 57403,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad5,
                code: 57404,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad6,
                code: 57405,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad7,
                code: 57406,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad8,
                code: 57407,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::Numpad9,
                code: 57408,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadDecimal,
                code: 57409,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadDivide,
                code: 57410,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadMultiply,
                code: 57411,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadSubtract,
                code: 57412,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadAdd,
                code: 57413,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadEnter,
                code: 57414,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadEqual,
                code: 57415,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadSeparator,
                code: 57416,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadLeft,
                code: 57417,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadRight,
                code: 57418,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadUp,
                code: 57419,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadDown,
                code: 57420,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadPageUp,
                code: 57421,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadPageDown,
                code: 57422,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadHome,
                code: 57423,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadEnd,
                code: 57424,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadInsert,
                code: 57425,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadDelete,
                code: 57426,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::NumpadBegin,
                code: 57427,
                final_byte: 'u',
                modifier: false,
            },
            KittyEntry {
                key: Key::ShiftLeft,
                code: 57441,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::ShiftRight,
                code: 57447,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::ControlLeft,
                code: 57442,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::ControlRight,
                code: 57448,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::MetaLeft,
                code: 57444,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::MetaRight,
                code: 57450,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::AltLeft,
                code: 57443,
                final_byte: 'u',
                modifier: true,
            },
            KittyEntry {
                key: Key::AltRight,
                code: 57449,
                final_byte: 'u',
                modifier: true,
            },
        ];

        assert_eq!(KITTY_ENTRIES.len(), expected.len());
        for entry in expected {
            assert_eq!(kitty_entry_for_key(entry.key), Some(entry));
        }
    }

    #[test]
    fn key_encode_legacy_pc_table_covers_supported_groups() {
        let expected_keys = [
            Key::ArrowUp,
            Key::ArrowDown,
            Key::ArrowRight,
            Key::ArrowLeft,
            Key::Home,
            Key::End,
            Key::Insert,
            Key::Delete,
            Key::PageUp,
            Key::PageDown,
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
            Key::F9,
            Key::F10,
            Key::F11,
            Key::F12,
            Key::Numpad0,
            Key::Numpad1,
            Key::Numpad2,
            Key::Numpad3,
            Key::Numpad4,
            Key::Numpad5,
            Key::Numpad6,
            Key::Numpad7,
            Key::Numpad8,
            Key::Numpad9,
            Key::NumpadDecimal,
            Key::NumpadDivide,
            Key::NumpadMultiply,
            Key::NumpadSubtract,
            Key::NumpadAdd,
            Key::NumpadEnter,
            Key::NumpadUp,
            Key::NumpadDown,
            Key::NumpadRight,
            Key::NumpadLeft,
            Key::NumpadBegin,
            Key::NumpadHome,
            Key::NumpadEnd,
            Key::NumpadInsert,
            Key::NumpadDelete,
            Key::NumpadPageUp,
            Key::NumpadPageDown,
            Key::Backspace,
            Key::Tab,
            Key::Enter,
            Key::Escape,
        ];

        assert_eq!(PC_KEY_SPECS.len(), expected_keys.len());
        for key in expected_keys {
            assert!(
                pc_key_spec(key).is_some(),
                "missing PC key spec for {key:?}"
            );
        }
    }

    #[test]
    fn key_encode_options_default_to_upstream_values() {
        assert_eq!(Options::default().kitty_flags, KeyFlags::DISABLED);
        assert!(!Options::default().cursor_key_application);
        assert!(!Options::default().keypad_key_application);
        assert!(!Options::default().backarrow_key_mode);
        assert!(!Options::default().ignore_keypad_with_numlock);
        assert!(!Options::default().alt_esc_prefix);
        assert!(!Options::default().modify_other_keys_state_2);
        assert_eq!(Options::default().macos_option_as_alt, OptionAsAlt::False);
    }

    #[test]
    fn key_encode_kitty_plain_text_and_repeat_with_disambiguate() {
        let opts = Options {
            kitty_flags: kitty_flags(),
            ..Options::default()
        };
        assert_eq!(encoded(text_event(Key::KeyA, "abcd"), opts), "abcd");

        let repeat = KeyEvent {
            action: KeyAction::Repeat,
            key: Key::KeyA,
            utf8: b"a".to_vec(),
            ..KeyEvent::default()
        };
        assert_eq!(encoded(repeat, opts), "a");
    }

    #[test]
    fn key_encode_kitty_enter_backspace_tab_report_all_off_and_on() {
        let opts = Options {
            kitty_flags: kitty_flags(),
            ..Options::default()
        };
        assert_eq!(encoded(event(Key::Enter), opts), "\r");
        assert_eq!(encoded(event(Key::Backspace), opts), "\u{7f}");
        assert_eq!(encoded(event(Key::Tab), opts), "\t");

        let release_enter = KeyEvent {
            action: KeyAction::Release,
            key: Key::Enter,
            ..KeyEvent::default()
        };
        let release_backspace = KeyEvent {
            action: KeyAction::Release,
            key: Key::Backspace,
            ..KeyEvent::default()
        };
        let release_tab = KeyEvent {
            action: KeyAction::Release,
            key: Key::Tab,
            ..KeyEvent::default()
        };
        assert_eq!(
            encoded(
                release_enter.clone(),
                Options {
                    kitty_flags: KeyFlags {
                        disambiguate: true,
                        report_events: true,
                        ..KeyFlags::DISABLED
                    },
                    ..Options::default()
                }
            ),
            ""
        );
        assert_eq!(
            encoded(
                release_enter,
                Options {
                    kitty_flags: kitty_all(),
                    ..Options::default()
                }
            ),
            "\x1b[13;1:3u"
        );
        assert_eq!(
            encoded(
                release_backspace,
                Options {
                    kitty_flags: kitty_all(),
                    ..Options::default()
                }
            ),
            "\x1b[127;1:3u"
        );
        assert_eq!(
            encoded(
                release_tab,
                Options {
                    kitty_flags: kitty_all(),
                    ..Options::default()
                }
            ),
            "\x1b[9;1:3u"
        );
    }

    #[test]
    fn key_encode_kitty_shift_specials_delete_arrow_composing_and_keypad() {
        let opts = Options {
            kitty_flags: kitty_flags(),
            ..Options::default()
        };
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Backspace,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                opts
            ),
            "\x1b[127;2u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Enter,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                opts
            ),
            "\x1b[13;2u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Tab,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                opts
            ),
            "\x1b[9;2u"
        );
        assert_eq!(encoded(text_event(Key::Delete, "\u{7f}"), opts), "\x1b[3~");
        assert_eq!(encoded(event(Key::ArrowUp), opts), "\x1b[A");
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyA,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    composing: true,
                    ..KeyEvent::default()
                },
                opts
            ),
            ""
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::ShiftLeft,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    composing: true,
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: KeyFlags {
                        disambiguate: true,
                        report_all: true,
                        ..KeyFlags::DISABLED
                    },
                    ..Options::default()
                }
            ),
            "\x1b[57441;2u"
        );
        assert_eq!(
            encoded(
                text_event(Key::Numpad1, "1"),
                Options {
                    kitty_flags: kitty_all(),
                    ..Options::default()
                }
            ),
            "\x1b[57400;;49u"
        );
    }

    #[test]
    fn key_encode_kitty_completed_table_representatives() {
        let opts = Options {
            kitty_flags: kitty_flags(),
            ..Options::default()
        };
        assert_eq!(encoded(event(Key::Insert), opts), "\x1b[2~");
        assert_eq!(encoded(event(Key::PageUp), opts), "\x1b[5~");
        assert_eq!(encoded(event(Key::Home), opts), "\x1b[H");
        assert_eq!(encoded(event(Key::End), opts), "\x1b[F");
        assert_eq!(encoded(event(Key::F1), opts), "\x1b[P");
        assert_eq!(encoded(event(Key::F3), opts), "\x1b[13~");
        assert_eq!(encoded(event(Key::F5), opts), "\x1b[15~");
        assert_eq!(encoded(event(Key::F12), opts), "\x1b[24~");
        assert_eq!(encoded(event(Key::F13), opts), "\x1b[57376u");
        assert_eq!(encoded(event(Key::F25), opts), "\x1b[57388u");
        assert_eq!(encoded(event(Key::PrintScreen), opts), "\x1b[57361u");
        assert_eq!(encoded(event(Key::Pause), opts), "\x1b[57362u");
        assert_eq!(encoded(event(Key::NumpadAdd), opts), "\x1b[57413u");
        assert_eq!(encoded(event(Key::NumpadEqual), opts), "\x1b[57415u");
        assert_eq!(encoded(event(Key::NumpadBegin), opts), "\x1b[57427u");
        assert_eq!(encoded(event(Key::CapsLock), opts), "");
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::CapsLock,
                    mods: Mods {
                        caps_lock: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: KeyFlags {
                        disambiguate: true,
                        report_all: true,
                        ..KeyFlags::DISABLED
                    },
                    ..Options::default()
                }
            ),
            "\x1b[57358;65u"
        );
    }

    #[test]
    fn key_encode_kitty_alternates_and_associated_text() {
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyA,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"A".to_vec(),
                    unshifted_codepoint: 'a' as u32,
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: KeyFlags {
                        disambiguate: true,
                        report_alternates: true,
                        ..KeyFlags::DISABLED
                    },
                    ..Options::default()
                }
            ),
            "\x1b[97:65;2u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Semicolon,
                    utf8: "ч".as_bytes().to_vec(),
                    unshifted_codepoint: 1095,
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: kitty_all(),
                    ..Options::default()
                }
            ),
            "\x1b[1095::59;;1095u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyW,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: "∑".as_bytes().to_vec(),
                    unshifted_codepoint: 'w' as u32,
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: kitty_all(),
                    macos_option_as_alt: OptionAsAlt::False,
                    ..Options::default()
                }
            ),
            "\x1b[119;3;8721u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyW,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: "∑".as_bytes().to_vec(),
                    unshifted_codepoint: 'w' as u32,
                    ..KeyEvent::default()
                },
                Options {
                    kitty_flags: kitty_all(),
                    macos_option_as_alt: OptionAsAlt::True,
                    ..Options::default()
                }
            ),
            "\x1b[119;3u"
        );
    }

    #[test]
    fn key_encode_legacy_control_alt_and_option_as_alt() {
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyC,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: b"c".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\u{3}"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyC,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: b"c".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::True,
                    ..Options::default()
                }
            ),
            "\x1bc"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Digit8,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    consumed_mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: b"[".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::False,
                    ..Options::default()
                }
            ),
            "["
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyC,
                    mods: Mods {
                        alt: true,
                        sides: ModSides {
                            alt: Side::Right,
                            ..ModSides::default()
                        },
                        ..Mods::new()
                    },
                    utf8: b"c".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::Left,
                    ..Options::default()
                }
            ),
            "c"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyC,
                    mods: Mods {
                        alt: true,
                        sides: ModSides {
                            alt: Side::Left,
                            ..ModSides::default()
                        },
                        ..Mods::new()
                    },
                    utf8: b"c".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::Left,
                    ..Options::default()
                }
            ),
            "\x1bc"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Space,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: b" ".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\0"
        );
    }

    #[test]
    fn key_encode_legacy_ctrl_seq_full_c0_table() {
        let ctrl = Mods {
            ctrl: true,
            ..Mods::new()
        };
        let cases = [
            (Key::Space, " ", ' ' as u32, 0),
            (Key::Slash, "/", '/' as u32, 31),
            (Key::Digit0, "0", '0' as u32, 48),
            (Key::Digit1, "1", '1' as u32, 49),
            (Key::Digit2, "2", '2' as u32, 0),
            (Key::Digit3, "3", '3' as u32, 27),
            (Key::Digit4, "4", '4' as u32, 28),
            (Key::Digit5, "5", '5' as u32, 29),
            (Key::Digit6, "6", '6' as u32, 30),
            (Key::Digit7, "7", '7' as u32, 31),
            (Key::Digit8, "8", '8' as u32, 127),
            (Key::Digit9, "9", '9' as u32, 57),
            (Key::Slash, "?", '?' as u32, 127),
            (Key::Digit2, "@", '2' as u32, 0),
            (Key::Backslash, "\\", '\\' as u32, 28),
            (Key::BracketRight, "]", ']' as u32, 29),
            (Key::Digit6, "^", '6' as u32, 30),
            (Key::Minus, "_", '-' as u32, 31),
            (Key::Backquote, "~", '`' as u32, 30),
        ];

        for (key, text, unshifted, expected) in cases {
            assert_eq!(
                ctrl_seq(key, text.as_bytes(), unshifted, ctrl),
                Some(expected),
                "failed ctrl sequence for {key:?}"
            );
        }

        for byte in b'a'..=b'z' {
            let key = Key::from_ascii(byte).expect("ascii letter must map to a key");
            let text = [byte];
            let expected = match byte {
                b'i' | b'm' => None,
                _ => Some(byte - b'a' + 1),
            };
            assert_eq!(ctrl_seq(key, &text, byte as u32, ctrl), expected);
        }

        assert_eq!(ctrl_seq(Key::BracketLeft, b"[", '[' as u32, ctrl), None);
    }

    #[test]
    fn key_encode_legacy_ctrl_seq_modifiers_layouts_and_csiu_fallthrough() {
        assert_eq!(
            ctrl_seq(
                Key::KeyC,
                b"c",
                'c' as u32,
                Mods {
                    ctrl: true,
                    sides: ModSides {
                        ctrl: Side::Right,
                        ..ModSides::default()
                    },
                    ..Mods::new()
                }
            ),
            Some(3)
        );
        assert_eq!(
            ctrl_seq(
                Key::KeyC,
                b"C",
                'c' as u32,
                Mods {
                    ctrl: true,
                    caps_lock: true,
                    ..Mods::new()
                }
            ),
            Some(3)
        );
        assert_eq!(
            ctrl_seq(
                Key::KeyC,
                b"C",
                'c' as u32,
                Mods {
                    ctrl: true,
                    shift: true,
                    ..Mods::new()
                }
            ),
            None
        );
        assert_eq!(
            ctrl_seq(
                Key::KeyC,
                "с".as_bytes(),
                0x0441,
                Mods {
                    ctrl: true,
                    ..Mods::new()
                }
            ),
            Some(3)
        );
        assert_eq!(
            ctrl_seq(
                Key::KeyC,
                "с".as_bytes(),
                0x0441,
                Mods {
                    ctrl: true,
                    alt: true,
                    ..Mods::new()
                }
            ),
            Some(3)
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyI,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: b"i".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[105;5u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyM,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: b"m".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[109;5u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::BracketLeft,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: b"[".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[91;5u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyM,
                    mods: Mods {
                        ctrl: true,
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"M".to_vec(),
                    unshifted_codepoint: 'm' as u32,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[109;6u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Digit2,
                    mods: Mods {
                        ctrl: true,
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"@".to_vec(),
                    unshifted_codepoint: '2' as u32,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[64;5u"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::BracketLeft,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    utf8: "ő".as_bytes().to_vec(),
                    unshifted_codepoint: 337,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[337;5u"
        );
    }

    #[test]
    fn key_encode_legacy_dead_key_and_alt_prefix_edge_cases() {
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Backspace,
                    utf8: b"A".to_vec(),
                    unshifted_codepoint: 0x0d,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            ""
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Enter,
                    utf8: b"A".to_vec(),
                    unshifted_codepoint: 0x0d,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "A"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Escape,
                    utf8: b"A".to_vec(),
                    unshifted_codepoint: 0x0d,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "A"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Backspace,
                    utf8: b"\x7f".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    backarrow_key_mode: false,
                    ..Options::default()
                }
            ),
            "\x7f"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Backspace,
                    utf8: b"\x7f".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    backarrow_key_mode: true,
                    ..Options::default()
                }
            ),
            "\x08"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyE,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    unshifted_codepoint: 'e' as u32,
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::True,
                    ..Options::default()
                }
            ),
            "\x1be"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Period,
                    mods: Mods {
                        shift: true,
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: b">".to_vec(),
                    unshifted_codepoint: '.' as u32,
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::True,
                    ..Options::default()
                }
            ),
            "\x1b>"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyF,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    utf8: "ф".as_bytes().to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    alt_esc_prefix: true,
                    macos_option_as_alt: OptionAsAlt::True,
                    ..Options::default()
                }
            ),
            "ф"
        );
    }

    #[test]
    fn key_encode_legacy_backspace_modify_other_and_function_keys() {
        assert_eq!(encoded(event(Key::Backspace), Options::default()), "\u{7f}");
        assert_eq!(
            encoded(
                event(Key::Backspace),
                Options {
                    backarrow_key_mode: true,
                    ..Options::default()
                }
            ),
            "\u{8}"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyH,
                    mods: Mods {
                        ctrl: true,
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"H".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    modify_other_keys_state_2: true,
                    ..Options::default()
                }
            ),
            "\x1b[27;6;72~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyH,
                    mods: Mods {
                        ctrl: true,
                        shift: true,
                        ..Mods::new()
                    },
                    consumed_mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"H".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    modify_other_keys_state_2: true,
                    ..Options::default()
                }
            ),
            "\x1b[27;6;72~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::F1,
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1bOP"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::F1,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[1;5P"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::F1,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    consumed_mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[1;2P"
        );
        assert_eq!(encoded(event(Key::ArrowUp), Options::default()), "\x1b[A");
        assert_eq!(
            encoded(
                event(Key::ArrowUp),
                Options {
                    cursor_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOA"
        );
    }

    #[test]
    fn key_encode_legacy_completed_cursor_edit_and_function_tables() {
        assert_eq!(encoded(event(Key::ArrowDown), Options::default()), "\x1b[B");
        assert_eq!(
            encoded(
                event(Key::ArrowDown),
                Options {
                    cursor_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOB"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Home,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[1;2H"
        );
        assert_eq!(encoded(event(Key::Insert), Options::default()), "\x1b[2~");
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Delete,
                    mods: Mods {
                        ctrl: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[3;5~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::PageDown,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b[6;3~"
        );

        let plain = [
            (Key::F1, "\x1bOP"),
            (Key::F2, "\x1bOQ"),
            (Key::F3, "\x1bOR"),
            (Key::F4, "\x1bOS"),
            (Key::F5, "\x1b[15~"),
            (Key::F6, "\x1b[17~"),
            (Key::F7, "\x1b[18~"),
            (Key::F8, "\x1b[19~"),
            (Key::F9, "\x1b[20~"),
            (Key::F10, "\x1b[21~"),
            (Key::F11, "\x1b[23~"),
            (Key::F12, "\x1b[24~"),
        ];
        for (key, expected) in plain {
            assert_eq!(encoded(event(key), Options::default()), expected);
        }

        let ctrl = [
            (Key::F1, "\x1b[1;5P"),
            (Key::F2, "\x1b[1;5Q"),
            (Key::F3, "\x1b[13;5~"),
            (Key::F4, "\x1b[1;5S"),
            (Key::F5, "\x1b[15;5~"),
            (Key::F6, "\x1b[17;5~"),
            (Key::F7, "\x1b[18;5~"),
            (Key::F8, "\x1b[19;5~"),
            (Key::F9, "\x1b[20;5~"),
            (Key::F10, "\x1b[21;5~"),
            (Key::F11, "\x1b[23;5~"),
            (Key::F12, "\x1b[24;5~"),
        ];
        for (key, expected) in ctrl {
            assert_eq!(
                encoded(
                    KeyEvent {
                        key,
                        mods: Mods {
                            ctrl: true,
                            ..Mods::new()
                        },
                        ..KeyEvent::default()
                    },
                    Options::default()
                ),
                expected
            );
        }
    }

    #[test]
    fn key_encode_legacy_completed_keypad_and_special_tables() {
        assert_eq!(
            encoded(
                text_event(Key::NumpadAdd, "+"),
                Options {
                    keypad_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOk"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::NumpadAdd,
                    mods: Mods {
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"+".to_vec(),
                    ..KeyEvent::default()
                },
                Options {
                    keypad_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bO2k"
        );
        assert_eq!(
            encoded(
                event(Key::NumpadEnter),
                Options {
                    keypad_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOM"
        );
        assert_eq!(
            encoded(
                event(Key::NumpadBegin),
                Options {
                    cursor_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOE"
        );
        assert_eq!(
            encoded(
                text_event(Key::NumpadDecimal, "."),
                Options {
                    keypad_key_application: true,
                    ignore_keypad_with_numlock: true,
                    ..Options::default()
                }
            ),
            "."
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Backspace,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options {
                    modify_other_keys_state_2: true,
                    ..Options::default()
                }
            ),
            "\x1b[27;3;127~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Tab,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b\t"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Tab,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options {
                    modify_other_keys_state_2: true,
                    ..Options::default()
                }
            ),
            "\x1b[27;3;9~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Enter,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b\r"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Enter,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options {
                    modify_other_keys_state_2: true,
                    ..Options::default()
                }
            ),
            "\x1b[27;3;13~"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::Escape,
                    mods: Mods {
                        alt: true,
                        ..Mods::new()
                    },
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            "\x1b\x1b"
        );
    }

    #[test]
    fn key_encode_legacy_keypad_and_super_text() {
        assert_eq!(encoded(event(Key::NumpadEnter), Options::default()), "\r");
        assert_eq!(
            encoded(text_event(Key::Numpad1, "1"), Options::default()),
            "1"
        );
        assert_eq!(
            encoded(
                text_event(Key::Numpad1, "1"),
                Options {
                    keypad_key_application: true,
                    ..Options::default()
                }
            ),
            "\x1bOq"
        );
        assert_eq!(
            encoded(
                text_event(Key::Numpad1, "1"),
                Options {
                    keypad_key_application: true,
                    ignore_keypad_with_numlock: true,
                    ..Options::default()
                }
            ),
            "1"
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyB,
                    mods: Mods {
                        super_: true,
                        ..Mods::new()
                    },
                    utf8: b"b".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            ""
        );
        assert_eq!(
            encoded(
                KeyEvent {
                    key: Key::KeyB,
                    mods: Mods {
                        super_: true,
                        shift: true,
                        ..Mods::new()
                    },
                    utf8: b"B".to_vec(),
                    ..KeyEvent::default()
                },
                Options::default()
            ),
            ""
        );
    }
}
