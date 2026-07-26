use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{ActionDefinition, Binding, BindingAction, BindingSet, HelpVisibility, KeyChord};

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
}

pub(crate) type ViewerKeymap = BindingSet<ViewerAction>;

const ACTIONS: &[ActionDefinition<ViewerAction>] = &[
    info(ViewerAction::Quit, "quit", "Quit leaf", true, "quit"),
    paired_info(
        ViewerAction::ScrollDown,
        "scroll-down",
        "Scroll down",
        true,
        "scroll down",
        "scroll",
    ),
    paired_info(
        ViewerAction::ScrollUp,
        "scroll-up",
        "Scroll up",
        true,
        "scroll up",
        "scroll",
    ),
    paired_info(
        ViewerAction::PageDown,
        "page-down",
        "Scroll down one page",
        true,
        "page down",
        "page up/down",
    ),
    paired_info(
        ViewerAction::PageUp,
        "page-up",
        "Scroll up one page",
        true,
        "page up",
        "page up/down",
    ),
    paired_info(
        ViewerAction::ScrollTop,
        "scroll-top",
        "Go to the top",
        true,
        "top",
        "top/bottom",
    ),
    paired_info(
        ViewerAction::ScrollBottom,
        "scroll-bottom",
        "Go to the bottom",
        true,
        "bottom",
        "top/bottom",
    ),
    paired_info(
        ViewerAction::FocusNextToc,
        "focus-next-toc",
        "Focus the next TOC entry",
        true,
        "next toc",
        "navigate toc",
    ),
    paired_info(
        ViewerAction::FocusPreviousToc,
        "focus-previous-toc",
        "Focus the previous TOC entry",
        true,
        "previous toc",
        "navigate toc",
    ),
    paired_info(
        ViewerAction::ScrollTocDown,
        "scroll-toc-down",
        "Scroll the TOC down",
        true,
        "scroll toc down",
        "navigate toc",
    ),
    paired_info(
        ViewerAction::ScrollTocUp,
        "scroll-toc-up",
        "Scroll the TOC up",
        true,
        "scroll toc up",
        "navigate toc",
    ),
    info(
        ViewerAction::ToggleToc,
        "toggle-toc",
        "Toggle the table of contents",
        true,
        "toggle toc",
    ),
    info(
        ViewerAction::OpenThemePicker,
        "open-theme-picker",
        "Open the theme picker",
        true,
        "theme picker",
    ),
    info(
        ViewerAction::OpenEditorPicker,
        "open-editor-picker",
        "Open the editor picker",
        true,
        "editor picker",
    ),
    info(
        ViewerAction::OpenHelp,
        "open-help",
        "Open keyboard help",
        true,
        "help",
    ),
    info(
        ViewerAction::ToggleWatch,
        "toggle-watch",
        "Toggle file watching",
        true,
        "toggle",
    ),
    info(
        ViewerAction::Reload,
        "reload",
        "Reload the file",
        true,
        "reload",
    ),
    info(
        ViewerAction::StartSearch,
        "start-search",
        "Start a search",
        true,
        "find",
    ),
    paired_info(
        ViewerAction::NextMatch,
        "next-match",
        "Go to the next match",
        true,
        "next",
        "next/prev",
    ),
    paired_info(
        ViewerAction::PreviousMatch,
        "previous-match",
        "Go to the previous match",
        true,
        "previous",
        "next/prev",
    ),
    info(
        ViewerAction::CopyRelativePath,
        "copy-relative-path",
        "Copy the relative path",
        true,
        "copy rel path",
    ),
    info(
        ViewerAction::CopyAbsolutePath,
        "copy-absolute-path",
        "Copy the absolute path",
        true,
        "copy abs path",
    ),
    info(
        ViewerAction::StartGotoLine,
        "start-goto-line",
        "Go to a line",
        true,
        "goto",
    ),
    info(
        ViewerAction::ToggleLineNumbers,
        "toggle-line-numbers",
        "Toggle line numbers",
        true,
        "line number",
    ),
    info(
        ViewerAction::OpenInEditor,
        "open-in-editor",
        "Open in the editor",
        true,
        "edit",
    ),
    info(
        ViewerAction::OpenFuzzyPicker,
        "open-fuzzy-picker",
        "Open the fuzzy file picker",
        true,
        "pick",
    ),
    info(
        ViewerAction::OpenFileBrowser,
        "open-file-browser",
        "Open the file browser",
        true,
        "file browser",
    ),
    info(
        ViewerAction::OpenPathViewer,
        "open-path-viewer",
        "Open the path viewer",
        true,
        "path viewer",
    ),
    paired_info(
        ViewerAction::ToggleReverseNavigation,
        "toggle-reverse-navigation",
        "Reverse number-key navigation",
        false,
        "reverse",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump1,
        "jump-1",
        "Use number-key jump 1",
        false,
        "jump 1",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump2,
        "jump-2",
        "Use number-key jump 2",
        false,
        "jump 2",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump3,
        "jump-3",
        "Use number-key jump 3",
        false,
        "jump 3",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump4,
        "jump-4",
        "Use number-key jump 4",
        false,
        "jump 4",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump5,
        "jump-5",
        "Use number-key jump 5",
        false,
        "jump 5",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump6,
        "jump-6",
        "Use number-key jump 6",
        false,
        "jump 6",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump7,
        "jump-7",
        "Use number-key jump 7",
        false,
        "jump 7",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump8,
        "jump-8",
        "Use number-key jump 8",
        false,
        "jump 8",
        "jump/reverse",
    ),
    paired_info(
        ViewerAction::Jump9,
        "jump-9",
        "Use number-key jump 9",
        false,
        "jump 9",
        "jump/reverse",
    ),
    info(
        ViewerAction::EnterCodeSelection,
        "enter-code-selection",
        "Select a code block",
        true,
        "focus code",
    ),
    info(
        ViewerAction::CopyVisibleCode,
        "copy-visible-code",
        "Copy the first visible code block",
        true,
        "copy code",
    ),
];

const fn info(
    action: ViewerAction,
    name: &'static str,
    description: &'static str,
    help_visible: bool,
    help_label: &'static str,
) -> ActionDefinition<ViewerAction> {
    paired_info(
        action,
        name,
        description,
        help_visible,
        help_label,
        help_label,
    )
}

const fn paired_info(
    action: ViewerAction,
    name: &'static str,
    description: &'static str,
    help_visible: bool,
    singular_help_label: &'static str,
    paired_help_label: &'static str,
) -> ActionDefinition<ViewerAction> {
    ActionDefinition {
        action,
        name,
        description,
        help_visible,
        paired_help_label,
        singular_help_label,
    }
}

impl BindingAction for ViewerAction {
    fn definitions() -> &'static [ActionDefinition<Self>] {
        ACTIONS
    }
}

pub(crate) fn default_keymap() -> ViewerKeymap {
    use HelpVisibility::{Primary, Synonym};
    use ViewerAction::*;
    let mut bindings = vec![
        binding('j', ScrollDown, Primary),
        binding('k', ScrollUp, Primary),
        named(KeyCode::Up, ScrollUp, Primary),
        named(KeyCode::Down, ScrollDown, Primary),
        binding('u', PageUp, Primary),
        binding('d', PageDown, Primary),
        named(KeyCode::PageUp, PageUp, Synonym),
        named(KeyCode::PageDown, PageDown, Synonym),
        binding('g', ScrollTop, Primary),
        binding('G', ScrollBottom, Primary),
        named(KeyCode::Home, ScrollTop, Synonym),
        named(KeyCode::End, ScrollBottom, Synonym),
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
            Primary,
        ));
    }
    bindings.extend([
        binding('0', ToggleReverseNavigation, Primary),
        binding('y', EnterCodeSelection, Primary),
        binding('Y', EnterCodeSelection, Primary),
        binding('c', EnterCodeSelection, Primary),
        binding('C', EnterCodeSelection, Primary),
        binding('J', FocusNextToc, Primary),
        binding('K', FocusPreviousToc, Primary),
        binding('U', ScrollTocUp, Primary),
        binding('D', ScrollTocDown, Primary),
        modified('f', KeyModifiers::CONTROL, StartSearch, Primary),
        binding('/', StartSearch, Synonym),
        binding('n', NextMatch, Primary),
        binding('N', PreviousMatch, Primary),
        modified('w', KeyModifiers::CONTROL, ToggleWatch, Primary),
        binding('w', ToggleWatch, Primary),
        modified('r', KeyModifiers::CONTROL, Reload, Primary),
        binding('r', Reload, Primary),
        binding('E', OpenEditorPicker, Primary),
        modified('e', KeyModifiers::CONTROL, OpenInEditor, Primary),
        binding('L', ToggleLineNumbers, Primary),
        binding('l', ToggleLineNumbers, Synonym),
        modified('l', KeyModifiers::CONTROL, StartGotoLine, Primary),
        binding('P', OpenFileBrowser, Primary),
        modified('p', KeyModifiers::CONTROL, OpenFuzzyPicker, Primary),
        modified('q', KeyModifiers::CONTROL, OpenFuzzyPicker, Synonym),
        binding('T', OpenThemePicker, Primary),
        binding('?', OpenHelp, Primary),
        binding('p', OpenPathViewer, Primary),
        binding('q', Quit, Primary),
        binding('Q', Quit, Synonym),
        modified('c', KeyModifiers::CONTROL, Quit, Synonym),
        binding('t', ToggleToc, Primary),
        binding('R', CopyRelativePath, Primary),
        binding('A', CopyAbsolutePath, Primary),
        modified('y', KeyModifiers::CONTROL, CopyVisibleCode, Primary),
    ]);
    ViewerKeymap::new("viewer", bindings)
}

pub(crate) fn resolve(overrides: &BTreeMap<String, String>) -> Result<ViewerKeymap, String> {
    let mut keymap = default_keymap();
    keymap.apply_overrides(overrides)?;
    Ok(keymap)
}

fn binding(
    key: char,
    action: ViewerAction,
    help_visibility: HelpVisibility,
) -> Binding<ViewerAction> {
    Binding::new(
        KeyChord::parse(&key.to_string()).unwrap(),
        action,
        help_visibility,
    )
}

fn named(
    code: KeyCode,
    action: ViewerAction,
    help_visibility: HelpVisibility,
) -> Binding<ViewerAction> {
    Binding::new(
        KeyChord::new(code, KeyModifiers::empty()),
        action,
        help_visibility,
    )
}

fn modified(
    key: char,
    modifiers: KeyModifiers,
    action: ViewerAction,
    help_visibility: HelpVisibility,
) -> Binding<ViewerAction> {
    Binding::new(
        KeyChord::new(KeyCode::Char(key), modifiers),
        action,
        help_visibility,
    )
}
