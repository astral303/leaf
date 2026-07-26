use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyChord {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyChord {
    pub(crate) fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self::normalized(code, modifiers)
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        let value = value.trim();
        if value.is_empty() {
            return Err("key cannot be empty".to_string());
        }

        let (modifier_parts, key) = if value == "+" {
            (Vec::new(), "+")
        } else if let Some(prefix) = value.strip_suffix("++") {
            (prefix.split('+').collect(), "+")
        } else {
            let mut parts: Vec<&str> = value.split('+').collect();
            let key = parts.pop().unwrap_or_default();
            (parts, key)
        };
        let mut modifiers = KeyModifiers::empty();

        for modifier in &modifier_parts {
            let flag = match modifier.trim().to_ascii_lowercase().as_str() {
                "ctrl" | "control" => KeyModifiers::CONTROL,
                "alt" | "option" => KeyModifiers::ALT,
                "shift" => KeyModifiers::SHIFT,
                "meta" | "super" | "hyper" | "cmd" | "command" => {
                    return Err(format!("unsupported modifier '{modifier}'"));
                }
                "" => return Err(format!("invalid key '{value}'")),
                _ => return Err(format!("unknown modifier '{modifier}'")),
            };
            if modifiers.contains(flag) {
                return Err(format!("duplicate modifier '{modifier}'"));
            }
            modifiers.insert(flag);
        }

        let code = parse_key_code(key)?;
        Ok(Self::normalized(code, modifiers))
    }

    pub(crate) fn from_event(event: &KeyEvent) -> Self {
        Self::new(event.code, event.modifiers)
    }

    pub(super) fn code(&self) -> KeyCode {
        self.code
    }

    pub(super) fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    fn normalized(code: KeyCode, mut modifiers: KeyModifiers) -> Self {
        let code = match code {
            KeyCode::Char(c) if modifiers.contains(KeyModifiers::SHIFT) => {
                modifiers.remove(KeyModifiers::SHIFT);
                KeyCode::Char(c.to_ascii_uppercase())
            }
            KeyCode::BackTab => {
                modifiers.insert(KeyModifiers::SHIFT);
                KeyCode::Tab
            }
            other => other,
        };
        Self { code, modifiers }
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "ctrl+")?;
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            write!(f, "alt+")?;
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            write!(f, "shift+")?;
        }
        match self.code {
            KeyCode::Char(c) if c.is_ascii_uppercase() => {
                write!(f, "shift+{}", c.to_ascii_lowercase())
            }
            KeyCode::Char(' ') => write!(f, "space"),
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Esc => write!(f, "esc"),
            KeyCode::Enter => write!(f, "enter"),
            KeyCode::Backspace => write!(f, "backspace"),
            KeyCode::Left => write!(f, "left"),
            KeyCode::Right => write!(f, "right"),
            KeyCode::Up => write!(f, "up"),
            KeyCode::Down => write!(f, "down"),
            KeyCode::Home => write!(f, "home"),
            KeyCode::End => write!(f, "end"),
            KeyCode::PageUp => write!(f, "page-up"),
            KeyCode::PageDown => write!(f, "page-down"),
            KeyCode::Tab => write!(f, "tab"),
            KeyCode::Delete => write!(f, "delete"),
            KeyCode::Insert => write!(f, "insert"),
            KeyCode::F(number) => write!(f, "f{number}"),
            _ => write!(f, "{:?}", self.code),
        }
    }
}

fn parse_key_code(value: &str) -> Result<KeyCode, String> {
    let normalized = value.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    let named = match normalized.as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" => Some(KeyCode::Enter),
        "backspace" | "bsp" => Some(KeyCode::Backspace),
        "space" => Some(KeyCode::Char(' ')),
        "left" | "arrow-left" => Some(KeyCode::Left),
        "right" | "arrow-right" => Some(KeyCode::Right),
        "up" | "arrow-up" => Some(KeyCode::Up),
        "down" | "arrow-down" => Some(KeyCode::Down),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "page-up" | "pageup" | "pgup" => Some(KeyCode::PageUp),
        "page-down" | "pagedown" | "pgdown" | "pgdn" => Some(KeyCode::PageDown),
        "tab" => Some(KeyCode::Tab),
        "backtab" | "back-tab" => Some(KeyCode::BackTab),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        _ => None,
    };
    if let Some(code) = named {
        return Ok(code);
    }

    if let Some(number) = normalized
        .strip_prefix('f')
        .and_then(|n| n.parse::<u8>().ok())
    {
        if (1..=24).contains(&number) {
            return Ok(KeyCode::F(number));
        }
    }

    let mut chars = value.chars();
    let Some(c) = chars.next() else {
        return Err("key cannot be empty".to_string());
    };
    if chars.next().is_none() && !c.is_control() {
        return Ok(KeyCode::Char(c));
    }
    Err(format!("unknown key '{value}'"))
}

pub(super) fn standalone_atom(chord: &KeyChord) -> String {
    help_atom(chord, ShiftedLetterFormat::SpelledOut)
}

pub(super) fn grouped_atom(chord: &KeyChord) -> String {
    help_atom(chord, ShiftedLetterFormat::Uppercase)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShiftedLetterFormat {
    SpelledOut,
    Uppercase,
}

fn help_atom(chord: &KeyChord, shifted_letter_format: ShiftedLetterFormat) -> String {
    let mut label = String::new();
    if chord.modifiers.contains(KeyModifiers::CONTROL) {
        label.push_str("ctrl+");
    }
    if chord.modifiers.contains(KeyModifiers::ALT) {
        label.push_str("alt+");
    }
    if chord.modifiers.contains(KeyModifiers::SHIFT) {
        label.push_str("shift+");
    }
    match chord.code {
        KeyCode::Char(character)
            if character.is_ascii_uppercase()
                && shifted_letter_format == ShiftedLetterFormat::Uppercase =>
        {
            label.push(character);
        }
        KeyCode::Char(character) if character.is_ascii_uppercase() => {
            label.push_str("shift+");
            label.push(character.to_ascii_lowercase());
        }
        KeyCode::Char(' ') => label.push_str("spc"),
        KeyCode::Char(character) => label.push(character),
        KeyCode::Esc => label.push_str("esc"),
        KeyCode::Enter => label.push_str("enter"),
        KeyCode::Backspace => label.push_str("bsp"),
        KeyCode::Left => label.push('←'),
        KeyCode::Right => label.push('→'),
        KeyCode::Up => label.push('↑'),
        KeyCode::Down => label.push('↓'),
        KeyCode::Home => label.push_str("home"),
        KeyCode::End => label.push_str("end"),
        KeyCode::PageUp => label.push_str("pgup"),
        KeyCode::PageDown => label.push_str("pgdn"),
        KeyCode::Tab => label.push_str("tab"),
        KeyCode::Delete => label.push_str("del"),
        KeyCode::Insert => label.push_str("insert"),
        KeyCode::F(number) => label.push_str(&format!("f{number}")),
        _ => label.push_str(&format!("{:?}", chord.code)),
    }
    label
}
