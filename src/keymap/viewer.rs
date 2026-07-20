use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{BindingSet, KeyChord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ViewerAction {
    Quit,
    ScrollDown,
    ScrollUp,
    PageDown,
    PageUp,
    ScrollTop,
    ScrollBottom,
    FocusNextToc,
    FocusPreviousToc,
    ScrollTocDown,
    ScrollTocUp,
    ToggleToc,
    OpenThemePicker,
    OpenEditorPicker,
    OpenHelp,
    ToggleWatch,
    Reload,
    StartSearch,
    NextMatch,
    PreviousMatch,
    CopyRelativePath,
    CopyAbsolutePath,
    StartGotoLine,
    ToggleLineNumbers,
    OpenInEditor,
    OpenFuzzyPicker,
    OpenFileBrowser,
    OpenPathViewer,
    ToggleReverseNavigation,
    Jump1,
    Jump2,
    Jump3,
    Jump4,
    Jump5,
    Jump6,
    Jump7,
    Jump8,
    Jump9,
    EnterCodeSelection,
    CopyVisibleCode,
    ToggleMouseCapture,
}

pub(crate) type ViewerKeymap = BindingSet<ViewerAction>;

pub(crate) struct ActionInfo {
    pub(crate) action: ViewerAction,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) help_visible: bool,
}

pub(crate) const ACTIONS: &[ActionInfo] = &[
    info(ViewerAction::Quit, "quit", "Quit leaf", true),
    info(ViewerAction::ScrollDown, "scroll-down", "Scroll down", true),
    info(ViewerAction::ScrollUp, "scroll-up", "Scroll up", true),
    info(
        ViewerAction::PageDown,
        "page-down",
        "Scroll down one page",
        true,
    ),
    info(ViewerAction::PageUp, "page-up", "Scroll up one page", true),
    info(ViewerAction::ScrollTop, "scroll-top", "Go to the top", true),
    info(
        ViewerAction::ScrollBottom,
        "scroll-bottom",
        "Go to the bottom",
        true,
    ),
    info(
        ViewerAction::FocusNextToc,
        "focus-next-toc",
        "Focus the next TOC entry",
        true,
    ),
    info(
        ViewerAction::FocusPreviousToc,
        "focus-previous-toc",
        "Focus the previous TOC entry",
        true,
    ),
    info(
        ViewerAction::ScrollTocDown,
        "scroll-toc-down",
        "Scroll the TOC down",
        true,
    ),
    info(
        ViewerAction::ScrollTocUp,
        "scroll-toc-up",
        "Scroll the TOC up",
        true,
    ),
    info(
        ViewerAction::ToggleToc,
        "toggle-toc",
        "Toggle the table of contents",
        true,
    ),
    info(
        ViewerAction::OpenThemePicker,
        "open-theme-picker",
        "Open the theme picker",
        true,
    ),
    info(
        ViewerAction::OpenEditorPicker,
        "open-editor-picker",
        "Open the editor picker",
        true,
    ),
    info(
        ViewerAction::OpenHelp,
        "open-help",
        "Open keyboard help",
        true,
    ),
    info(
        ViewerAction::ToggleWatch,
        "toggle-watch",
        "Toggle file watching",
        true,
    ),
    info(ViewerAction::Reload, "reload", "Reload the file", true),
    info(
        ViewerAction::StartSearch,
        "start-search",
        "Start a search",
        true,
    ),
    info(
        ViewerAction::NextMatch,
        "next-match",
        "Go to the next match",
        true,
    ),
    info(
        ViewerAction::PreviousMatch,
        "previous-match",
        "Go to the previous match",
        true,
    ),
    info(
        ViewerAction::CopyRelativePath,
        "copy-relative-path",
        "Copy the relative path",
        true,
    ),
    info(
        ViewerAction::CopyAbsolutePath,
        "copy-absolute-path",
        "Copy the absolute path",
        true,
    ),
    info(
        ViewerAction::StartGotoLine,
        "start-goto-line",
        "Go to a line",
        true,
    ),
    info(
        ViewerAction::ToggleLineNumbers,
        "toggle-line-numbers",
        "Toggle line numbers",
        true,
    ),
    info(
        ViewerAction::OpenInEditor,
        "open-in-editor",
        "Open in the editor",
        true,
    ),
    info(
        ViewerAction::OpenFuzzyPicker,
        "open-fuzzy-picker",
        "Open the fuzzy file picker",
        true,
    ),
    info(
        ViewerAction::OpenFileBrowser,
        "open-file-browser",
        "Open the file browser",
        true,
    ),
    info(
        ViewerAction::OpenPathViewer,
        "open-path-viewer",
        "Open the path viewer",
        true,
    ),
    info(
        ViewerAction::ToggleReverseNavigation,
        "toggle-reverse-navigation",
        "Reverse number-key navigation",
        false,
    ),
    info(
        ViewerAction::Jump1,
        "jump-1",
        "Use number-key jump 1",
        false,
    ),
    info(
        ViewerAction::Jump2,
        "jump-2",
        "Use number-key jump 2",
        false,
    ),
    info(
        ViewerAction::Jump3,
        "jump-3",
        "Use number-key jump 3",
        false,
    ),
    info(
        ViewerAction::Jump4,
        "jump-4",
        "Use number-key jump 4",
        false,
    ),
    info(
        ViewerAction::Jump5,
        "jump-5",
        "Use number-key jump 5",
        false,
    ),
    info(
        ViewerAction::Jump6,
        "jump-6",
        "Use number-key jump 6",
        false,
    ),
    info(
        ViewerAction::Jump7,
        "jump-7",
        "Use number-key jump 7",
        false,
    ),
    info(
        ViewerAction::Jump8,
        "jump-8",
        "Use number-key jump 8",
        false,
    ),
    info(
        ViewerAction::Jump9,
        "jump-9",
        "Use number-key jump 9",
        false,
    ),
    info(
        ViewerAction::EnterCodeSelection,
        "enter-code-selection",
        "Select a code block",
        true,
    ),
    info(
        ViewerAction::CopyVisibleCode,
        "copy-visible-code",
        "Copy the first visible code block",
        true,
    ),
    info(
        ViewerAction::ToggleMouseCapture,
        "toggle-mouse-capture",
        "Toggle mouse capture",
        true,
    ),
];

const fn info(
    action: ViewerAction,
    name: &'static str,
    description: &'static str,
    help_visible: bool,
) -> ActionInfo {
    ActionInfo {
        action,
        name,
        description,
        help_visible,
    }
}

pub(crate) fn default_keymap() -> ViewerKeymap {
    use ViewerAction::*;
    let mut bindings = vec![
        binding('q', Quit),
        binding('Q', Quit),
        modified('c', KeyModifiers::CONTROL, Quit),
        binding('j', ScrollDown),
        named(KeyCode::Down, ScrollDown),
        binding('k', ScrollUp),
        named(KeyCode::Up, ScrollUp),
        binding('d', PageDown),
        named(KeyCode::PageDown, PageDown),
        binding('u', PageUp),
        named(KeyCode::PageUp, PageUp),
        binding('g', ScrollTop),
        named(KeyCode::Home, ScrollTop),
        binding('G', ScrollBottom),
        named(KeyCode::End, ScrollBottom),
        binding('J', FocusNextToc),
        binding('K', FocusPreviousToc),
        binding('D', ScrollTocDown),
        binding('U', ScrollTocUp),
        binding('t', ToggleToc),
        binding('T', OpenThemePicker),
        binding('E', OpenEditorPicker),
        binding('?', OpenHelp),
        binding('w', ToggleWatch),
        modified('w', KeyModifiers::CONTROL, ToggleWatch),
        binding('r', Reload),
        modified('r', KeyModifiers::CONTROL, Reload),
        binding('/', StartSearch),
        modified('f', KeyModifiers::CONTROL, StartSearch),
        binding('n', NextMatch),
        binding('N', PreviousMatch),
        binding('R', CopyRelativePath),
        binding('A', CopyAbsolutePath),
        modified('l', KeyModifiers::CONTROL, StartGotoLine),
        binding('l', ToggleLineNumbers),
        binding('L', ToggleLineNumbers),
        modified('e', KeyModifiers::CONTROL, OpenInEditor),
        modified('p', KeyModifiers::CONTROL, OpenFuzzyPicker),
        modified('q', KeyModifiers::CONTROL, OpenFuzzyPicker),
        binding('P', OpenFileBrowser),
        binding('p', OpenPathViewer),
        binding('0', ToggleReverseNavigation),
        binding('c', EnterCodeSelection),
        binding('C', EnterCodeSelection),
        binding('y', EnterCodeSelection),
        binding('Y', EnterCodeSelection),
        modified('y', KeyModifiers::CONTROL, CopyVisibleCode),
        binding('m', ToggleMouseCapture),
        binding('M', ToggleMouseCapture),
    ];
    for (number, action) in [
        Jump1, Jump2, Jump3, Jump4, Jump5, Jump6, Jump7, Jump8, Jump9,
    ]
    .into_iter()
    .enumerate()
    {
        bindings.push(binding(
            char::from_digit((number + 1) as u32, 10).unwrap(),
            action,
        ));
    }
    ViewerKeymap::new(bindings)
}

pub(crate) fn resolve(overrides: &BTreeMap<String, String>) -> Result<ViewerKeymap, String> {
    let mut keymap = default_keymap();
    let actions = ACTIONS.iter().map(|info| info.action).collect::<Vec<_>>();
    keymap.apply_overrides(overrides, &actions, parse_action, action_name)?;
    Ok(keymap)
}

pub(crate) fn action_info(action: ViewerAction) -> &'static ActionInfo {
    ACTIONS.iter().find(|info| info.action == action).unwrap()
}

pub(crate) fn print_catalog(keymap: &ViewerKeymap, include_hidden: bool) {
    let rows = ACTIONS
        .iter()
        .filter(|info| include_hidden || info.help_visible)
        .map(|info| {
            (
                keymap.keys_for(&[info.action]).join(", "),
                info.name,
                info.description,
                if keymap.is_configured(info.action) {
                    "yes"
                } else {
                    "no"
                },
            )
        })
        .collect::<Vec<_>>();
    let key_width = rows
        .iter()
        .map(|row| row.0.len())
        .max()
        .unwrap_or(4)
        .max("KEYS".len());
    let action_width = rows
        .iter()
        .map(|row| row.1.len())
        .max()
        .unwrap_or(6)
        .max("ACTION".len());
    let description_width = rows
        .iter()
        .map(|row| row.2.len())
        .max()
        .unwrap_or(11)
        .max("DESCRIPTION".len());

    println!(
        "{:<key_width$}  {:<action_width$}  {:<description_width$}  CONFIGURED",
        "KEYS", "ACTION", "DESCRIPTION"
    );
    for (keys, action, description, configured) in rows {
        println!(
            "{keys:<key_width$}  {action:<action_width$}  {description:<description_width$}  {configured}"
        );
    }
}

pub(crate) fn key_label(
    keymap: &ViewerKeymap,
    actions: &[ViewerAction],
    max_chars: usize,
) -> String {
    let label = keymap.keys_for(actions).join("/");
    if label.chars().count() <= max_chars {
        return label;
    }
    let keep = max_chars.saturating_sub(1);
    let mut shortened = label.chars().take(keep).collect::<String>();
    shortened.push('…');
    shortened
}

fn parse_action(value: &str) -> Option<ViewerAction> {
    ACTIONS
        .iter()
        .find(|info| info.name == value.trim().to_ascii_lowercase())
        .map(|info| info.action)
}

fn action_name(action: ViewerAction) -> &'static str {
    action_info(action).name
}

fn binding(key: char, action: ViewerAction) -> (KeyChord, ViewerAction) {
    (KeyChord::parse(&key.to_string()).unwrap(), action)
}

fn named(code: KeyCode, action: ViewerAction) -> (KeyChord, ViewerAction) {
    let event = crossterm::event::KeyEvent::new(code, KeyModifiers::empty());
    (KeyChord::from_event(&event), action)
}

fn modified(key: char, modifiers: KeyModifiers, action: ViewerAction) -> (KeyChord, ViewerAction) {
    let event = crossterm::event::KeyEvent::new(KeyCode::Char(key), modifiers);
    (KeyChord::from_event(&event), action)
}
