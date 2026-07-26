use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{ActionDefinition, Binding, BindingAction, BindingSet, HelpVisibility, KeyChord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlobalAction {
    ToggleMouseCapture,
}

pub(crate) type GlobalKeymap = BindingSet<GlobalAction>;

const ACTIONS: &[ActionDefinition<GlobalAction>] = &[ActionDefinition {
    action: GlobalAction::ToggleMouseCapture,
    name: "toggle-mouse-capture",
    description: "Toggle mouse capture",
    help_visible: true,
    paired_help_label: "capture",
    singular_help_label: "capture",
}];

impl BindingAction for GlobalAction {
    fn definitions() -> &'static [ActionDefinition<Self>] {
        ACTIONS
    }
}

pub(crate) fn default_keymap() -> GlobalKeymap {
    use GlobalAction::ToggleMouseCapture;
    use HelpVisibility::{Primary, Synonym};

    GlobalKeymap::new(
        "global",
        [
            binding('M', ToggleMouseCapture, Primary),
            binding('m', ToggleMouseCapture, Synonym),
        ],
    )
}

pub(crate) fn resolve(overrides: &BTreeMap<String, String>) -> Result<GlobalKeymap, String> {
    let mut keymap = default_keymap();
    keymap.apply_overrides(overrides)?;
    Ok(keymap)
}

fn binding(
    key: char,
    action: GlobalAction,
    help_visibility: HelpVisibility,
) -> Binding<GlobalAction> {
    Binding::new(
        KeyChord::new(KeyCode::Char(key), KeyModifiers::empty()),
        action,
        help_visibility,
    )
}
