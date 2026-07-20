use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) mod viewer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyChord {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyChord {
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
        Self::normalized(event.code, event.modifiers)
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

#[derive(Clone, Debug)]
pub(crate) struct BindingSet<A> {
    bindings: Vec<(KeyChord, A)>,
    configured_actions: Vec<A>,
}

impl<A: Copy + Eq> BindingSet<A> {
    pub(crate) fn new(bindings: impl IntoIterator<Item = (KeyChord, A)>) -> Self {
        Self {
            bindings: bindings.into_iter().collect(),
            configured_actions: Vec::new(),
        }
    }

    pub(crate) fn action_for(&self, event: &KeyEvent) -> Option<A> {
        let chord = KeyChord::from_event(event);
        self.bindings
            .iter()
            .find(|(candidate, _)| *candidate == chord)
            .map(|(_, action)| *action)
    }

    pub(crate) fn keys_for(&self, actions: &[A]) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|(_, action)| actions.contains(action))
            .map(|(key, _)| key.to_string())
            .collect()
    }

    pub(crate) fn is_configured(&self, action: A) -> bool {
        self.configured_actions.contains(&action)
    }

    pub(crate) fn apply_overrides(
        &mut self,
        overrides: &std::collections::BTreeMap<String, String>,
        all_actions: &[A],
        parse_action: impl Fn(&str) -> Option<A>,
        action_name: impl Fn(A) -> &'static str,
    ) -> Result<(), String> {
        let mut parsed: Vec<(KeyChord, Option<A>, &str)> = Vec::new();
        let mut errors = Vec::new();

        for (key_name, action_name_value) in overrides {
            let key = match KeyChord::parse(key_name) {
                Ok(key) => key,
                Err(message) => {
                    errors.push(format!("'{key_name}': {message}"));
                    continue;
                }
            };
            let action = if action_name_value.eq_ignore_ascii_case("none") {
                None
            } else {
                match parse_action(action_name_value) {
                    Some(action) => Some(action),
                    None => {
                        errors.push(format!(
                            "'{key_name}': unknown action '{action_name_value}'"
                        ));
                        continue;
                    }
                }
            };

            if let Some((_, previous, previous_name)) =
                parsed.iter().find(|(candidate, _, _)| *candidate == key)
            {
                if *previous != action {
                    errors.push(format!(
                        "'{key_name}' conflicts with equivalent key '{previous_name}'"
                    ));
                }
                continue;
            }
            parsed.push((key, action, key_name));
        }

        for (key, replacement, _) in parsed {
            if let Some(index) = self
                .bindings
                .iter()
                .position(|(candidate, _)| *candidate == key)
            {
                let previous = self.bindings[index].1;
                mark_configured(&mut self.configured_actions, previous);
                match replacement {
                    Some(action) => {
                        self.bindings.remove(index);
                        self.bindings.insert(0, (key, action));
                        mark_configured(&mut self.configured_actions, action);
                    }
                    None => {
                        self.bindings.remove(index);
                    }
                }
            } else if let Some(action) = replacement {
                self.bindings.insert(0, (key, action));
                mark_configured(&mut self.configured_actions, action);
            }
        }

        let missing: Vec<String> = all_actions
            .iter()
            .copied()
            .filter(|action| !self.bindings.iter().any(|(_, bound)| bound == action))
            .map(|action| action_name(action).to_string())
            .collect();
        if !missing.is_empty() {
            errors.push(format!(
                "every viewer action needs a key; unbound: {}",
                missing.join(", ")
            ));
        }
        if !errors.is_empty() {
            return Err(format_errors(errors));
        }
        Ok(())
    }
}

fn mark_configured<A: Copy + Eq>(configured: &mut Vec<A>, action: A) {
    if !configured.contains(&action) {
        configured.push(action);
    }
}

fn format_errors(errors: Vec<String>) -> String {
    let details = errors
        .into_iter()
        .map(|error| format!("  - {error}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Invalid viewer keymap:\n{details}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_equivalent_key_spellings() {
        assert_eq!(KeyChord::parse("shift+e"), KeyChord::parse("E"));
        assert_eq!(KeyChord::parse("page down"), KeyChord::parse("pgdn"));
        assert_eq!(KeyChord::parse("option+x"), KeyChord::parse("alt+x"));
    }

    #[test]
    fn rejects_unsupported_modifiers() {
        assert!(KeyChord::parse("super+x")
            .unwrap_err()
            .contains("unsupported modifier"));
    }

    #[test]
    fn accepts_function_keys_and_literal_plus() {
        assert_eq!(KeyChord::parse("f24").unwrap().to_string(), "f24");
        assert_eq!(KeyChord::parse("+").unwrap().to_string(), "+");
        assert_eq!(KeyChord::parse("ctrl++").unwrap().to_string(), "ctrl++");
    }
}
