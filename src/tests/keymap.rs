use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keymap::{
    format_action_help, format_paired_help, format_sequence_help,
    global::{self, GlobalAction},
    viewer::{self, ViewerAction},
    wrap_help_row, HelpLine, KeyChord, Keymaps,
};
use crate::{app::App, markdown::parse_markdown};
use ratatui::{backend::TestBackend, Terminal};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

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

#[test]
fn default_viewer_keymap_preserves_existing_shortcuts() {
    let keymap = viewer::default_keymap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('q'), KeyModifiers::empty())),
        Some(ViewerAction::Quit)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('Q'), KeyModifiers::SHIFT)),
        Some(ViewerAction::Quit)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::PageDown, KeyModifiers::empty())),
        Some(ViewerAction::PageDown)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        Some(ViewerAction::OpenFuzzyPicker)
    );
}

#[test]
fn default_global_keymap_preserves_mouse_capture_shortcuts() {
    let keymap = global::default_keymap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('m'), KeyModifiers::empty())),
        Some(GlobalAction::ToggleMouseCapture)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('M'), KeyModifiers::SHIFT)),
        Some(GlobalAction::ToggleMouseCapture)
    );
}

#[test]
fn viewer_shortcuts_require_exact_modifiers() {
    let keymap = viewer::default_keymap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('j'), KeyModifiers::CONTROL)),
        None
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('q'), KeyModifiers::ALT)),
        None
    );
    assert_eq!(
        keymap.action_for(&key(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        )),
        None
    );
}

#[test]
fn configured_shortcuts_require_and_accept_the_exact_chord() {
    let overrides = BTreeMap::from([("ctrl+space".to_string(), "page-down".to_string())]);
    let keymap = viewer::resolve(&overrides).unwrap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        Some(ViewerAction::PageDown)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char(' '), KeyModifiers::ALT)),
        None
    );
}

#[test]
fn viewer_overrides_replace_and_remove_default_bindings() {
    let overrides = BTreeMap::from([
        ("esc".to_string(), "quit".to_string()),
        ("q".to_string(), "none".to_string()),
        ("space".to_string(), "page-down".to_string()),
    ]);
    let keymap = viewer::resolve(&overrides).unwrap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Esc, KeyModifiers::empty())),
        Some(ViewerAction::Quit)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('q'), KeyModifiers::empty())),
        None
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char(' '), KeyModifiers::empty())),
        Some(ViewerAction::PageDown)
    );
    assert!(keymap.is_configured(ViewerAction::Quit));
    assert!(keymap.is_configured(ViewerAction::PageDown));
}

#[test]
fn global_overrides_replace_default_bindings() {
    let overrides = BTreeMap::from([
        ("esc".to_string(), "toggle-mouse-capture".to_string()),
        ("m".to_string(), "none".to_string()),
    ]);
    let keymap = global::resolve(&overrides).unwrap();

    assert_eq!(
        keymap.action_for(&key(KeyCode::Esc, KeyModifiers::empty())),
        Some(GlobalAction::ToggleMouseCapture)
    );
    assert_eq!(
        keymap.action_for(&key(KeyCode::Char('m'), KeyModifiers::empty())),
        None
    );
    assert!(keymap.is_configured(GlobalAction::ToggleMouseCapture));
}

#[test]
fn removing_an_already_unbound_valid_key_is_idempotent() {
    let overrides = BTreeMap::from([("f24".to_string(), "none".to_string())]);

    assert!(viewer::resolve(&overrides).is_ok());
}

#[test]
fn removing_an_invalid_key_still_fails() {
    let overrides = BTreeMap::from([("not-a-key".to_string(), "none".to_string())]);

    let error = viewer::resolve(&overrides).unwrap_err();
    assert!(error.contains("unknown key"));
}

#[test]
fn equivalent_configured_keys_coalesce_for_the_same_action() {
    let overrides = BTreeMap::from([
        ("E".to_string(), "open-editor-picker".to_string()),
        ("shift+e".to_string(), "open-editor-picker".to_string()),
    ]);

    assert!(viewer::resolve(&overrides).is_ok());
}

#[test]
fn equivalent_configured_keys_reject_conflicting_actions() {
    let overrides = BTreeMap::from([
        ("E".to_string(), "open-help".to_string()),
        ("shift+e".to_string(), "open-editor-picker".to_string()),
    ]);

    let error = viewer::resolve(&overrides).unwrap_err();
    assert!(error.contains("conflicts with equivalent key"));
}

#[test]
fn viewer_keymap_reports_all_semantic_errors_together() {
    let overrides = BTreeMap::from([
        ("super+x".to_string(), "quit".to_string()),
        ("ctrl+x".to_string(), "does-not-exist".to_string()),
    ]);

    let error = viewer::resolve(&overrides).unwrap_err();
    assert!(error.contains("unsupported modifier"));
    assert!(error.contains("unknown action"));
}

#[test]
fn viewer_keymap_rejects_an_action_with_no_remaining_binding() {
    let overrides = BTreeMap::from([
        ("q".to_string(), "none".to_string()),
        ("shift+q".to_string(), "none".to_string()),
        ("ctrl+c".to_string(), "none".to_string()),
    ]);

    let error = viewer::resolve(&overrides).unwrap_err();
    assert!(error.contains("unbound: quit"));
}

#[test]
fn keymaps_reject_cross_scope_binding_collisions() {
    let global_overrides =
        BTreeMap::from([("esc".to_string(), "toggle-mouse-capture".to_string())]);
    let viewer_overrides = BTreeMap::from([("esc".to_string(), "quit".to_string())]);

    let error = Keymaps::resolve(&global_overrides, &viewer_overrides).unwrap_err();
    for expected in ["'esc'", "global", "toggle-mouse-capture", "viewer", "quit"] {
        assert!(
            error.contains(expected),
            "missing {expected:?} in {error:?}"
        );
    }
}

#[test]
fn catalogs_are_selected_by_their_binding_set_names() {
    let global_overrides =
        BTreeMap::from([("esc".to_string(), "toggle-mouse-capture".to_string())]);
    let keymaps = Keymaps::resolve(&global_overrides, &BTreeMap::new()).unwrap();

    let mut global_output = Vec::new();
    keymaps
        .catalog("global", false)
        .unwrap()
        .write_to(&mut global_output)
        .unwrap();
    let global_output = String::from_utf8(global_output).unwrap();
    assert!(global_output.contains("toggle-mouse-capture"));
    assert!(global_output.contains("yes"));
    assert!(!global_output.contains("quit"));

    let mut viewer_output = Vec::new();
    keymaps
        .catalog("viewer", false)
        .unwrap()
        .write_to(&mut viewer_output)
        .unwrap();
    let viewer_output = String::from_utf8(viewer_output).unwrap();
    assert!(viewer_output.contains("quit"));
    assert!(!viewer_output.contains("toggle-mouse-capture"));
}

#[test]
fn catalog_lookup_reports_registered_names() {
    let keymaps = Keymaps::defaults();

    let error = keymaps
        .catalog("browser", false)
        .err()
        .expect("unknown catalog should fail");
    assert!(error.contains("Unknown keymap: 'browser'"));
    assert!(error.contains("global, viewer"));
}

const JUMP_ACTIONS: &[ViewerAction] = &[
    ViewerAction::Jump1,
    ViewerAction::Jump2,
    ViewerAction::Jump3,
    ViewerAction::Jump4,
    ViewerAction::Jump5,
    ViewerAction::Jump6,
    ViewerAction::Jump7,
    ViewerAction::Jump8,
    ViewerAction::Jump9,
];

fn remapped_jump_overrides() -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    for number in 1..=9 {
        overrides.insert(number.to_string(), "none".to_string());
        overrides.insert(format!("ctrl+{number}"), format!("jump-{number}"));
    }
    overrides
}

fn help_line(keys: &str, description: &'static str) -> HelpLine {
    HelpLine {
        keys: keys.to_string(),
        description,
    }
}

fn app_with_viewer_overrides(source: &str, viewer_overrides: &BTreeMap<String, String>) -> App {
    let keymaps = Keymaps::resolve(&BTreeMap::new(), viewer_overrides).unwrap();
    let (ss, theme) = super::test_assets();
    let (lines, toc, _, _) =
        parse_markdown(source, &ss, &theme, &super::test_md_theme(), false, true).into();
    let mut app = App::new(lines, toc, "test".to_string(), false, false, None, None);
    app.set_keymaps(keymaps);
    app
}

fn render_help_popup(viewer_overrides: &BTreeMap<String, String>) -> Vec<String> {
    let mut app = app_with_viewer_overrides("body", viewer_overrides);
    app.open_help();

    let backend = TestBackend::new(90, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| crate::render::ui(frame, &mut app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let (left, top) = super::find_symbol(buffer, "┌").expect("help popup top-left corner");
    let right = left + 52;
    let bottom = (top..buffer.area.height)
        .find(|y| {
            buffer
                .cell((right, *y))
                .is_some_and(|cell| cell.symbol() == "┘")
        })
        .expect("help popup bottom-right corner");

    (top..=bottom)
        .map(|y| {
            (left..=right)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect()
        })
        .collect()
}

fn default_help_popup() -> Vec<String> {
    let mut expected = r#"┌─ Help ────────────────────────────────────────────┐
│ VERSION                                           │
│ Keyboard shortcuts                                │
│                                                   │
│ Navigation                  Mouse                 │
│ j/k, ↑/↓   scroll           shift+m     capture   │
│ u/d        page up/down     dbl-click   copy code │
│ g/G        top/bottom       dbl-click   copy link │
│ 1-9/0»1-9  jump/reverse     ctrl+click  open link │
│ y/Y, c/C   focus code       shift+drag  slct text │
│ J/K, U/D   navigate toc                           │
│                                                   │
│ Search                      Watch                 │
│ ctrl+f     find             ctrl+w, w   toggle    │
│ n/N        next/prev        ctrl+r, r   reload    │
│                                                   │
│ Actions                                           │
│ shift+e    editor picker    ctrl+e      edit      │
│ shift+l    line number      ctrl+l      goto      │
│ shift+p    file browser     ctrl+p      pick      │
│ shift+t    theme picker     ?           help      │
│ p          path viewer      q           quit      │
│ t          toggle toc                             │
│                                                   │
│ esc close · ? close                               │
└───────────────────────────────────────────────────┘"#
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    expected[1] = format!("│ {:<49} │", crate::cli::version_text());
    expected
}

fn configured_help_popup() -> Vec<String> {
    let mut expected = r#"┌─ Help ────────────────────────────────────────────┐
│ VERSION                                           │
│ Keyboard shortcuts                                │
│                                                   │
│ Navigation                  Mouse                 │
│ j/k, ↑/↓   scroll           shift+m     capture   │
│ u/d,       page up/down     dbl-click   copy code │
│   bsp/spc                   dbl-click   copy link │
│ g/G        top/bottom       ctrl+click  open link │
│ 1-9/0»1-9  jump/reverse     shift+drag  slct text │
│ y/Y, c/C   focus code                             │
│ J/K, U/D   navigate toc                           │
│                                                   │
│ Search                      Watch                 │
│ ctrl+f     find             ctrl+w, w   toggle    │
│ n/N        next/prev        ctrl+r, r   reload    │
│                                                   │
│ Actions                                           │
│ shift+e    editor picker    ctrl+e      edit      │
│ shift+l    line number      ctrl+l      goto      │
│ shift+p    file browser     ctrl+p      pick      │
│ shift+t    theme picker     ?           help      │
│ p          path viewer      q, esc      quit      │
│ t          toggle toc                             │
│                                                   │
│ esc close · ? close                               │
└───────────────────────────────────────────────────┘"#
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    expected[1] = format!("│ {:<49} │", crate::cli::version_text());
    expected
}

#[test]
fn default_help_matches_the_upstream_popup() {
    assert_eq!(render_help_popup(&BTreeMap::new()), default_help_popup());
}

#[test]
fn restating_primary_defaults_does_not_change_help() {
    let overrides = BTreeMap::from([
        ("j".to_string(), "scroll-down".to_string()),
        ("q".to_string(), "quit".to_string()),
    ]);

    assert_eq!(
        render_help_popup(&overrides),
        render_help_popup(&BTreeMap::new())
    );
}

#[test]
fn personal_overrides_match_the_configured_help_golden() {
    let overrides = BTreeMap::from([
        ("backspace".to_string(), "page-up".to_string()),
        ("esc".to_string(), "quit".to_string()),
        ("space".to_string(), "page-down".to_string()),
    ]);
    assert_eq!(render_help_popup(&overrides), configured_help_popup());
}

#[test]
fn jump_range_wraps_after_the_alternative_separator() {
    let keymap = viewer::resolve(&remapped_jump_overrides()).unwrap();

    let row = format_sequence_help(&keymap, ViewerAction::ToggleReverseNavigation, JUMP_ACTIONS);

    assert_eq!(
        wrap_help_row(row, 9, 25, 2),
        vec![
            help_line("ctrl+1-9/", "jump/reverse"),
            help_line("  0»ctrl+1-9", ""),
        ]
    );
}

#[test]
fn jump_range_prefers_a_prefix_with_matching_modifiers() {
    let mut overrides = remapped_jump_overrides();
    overrides.insert(
        "ctrl+0".to_string(),
        "toggle-reverse-navigation".to_string(),
    );
    let keymap = viewer::resolve(&overrides).unwrap();

    let row = format_sequence_help(&keymap, ViewerAction::ToggleReverseNavigation, JUMP_ACTIONS);

    assert_eq!(
        wrap_help_row(row, 9, 25, 2),
        vec![
            help_line("ctrl+1-9/", "jump/reverse"),
            help_line("  ctrl+0»ctrl+1-9", ""),
        ]
    );
}

#[test]
fn explicitly_configured_synonym_surfaces_in_help_and_status() {
    let overrides = BTreeMap::from([("/".to_string(), "start-search".to_string())]);
    let keymap = viewer::resolve(&overrides).unwrap();

    assert_eq!(
        format_action_help(&keymap, ViewerAction::StartSearch).key_label(),
        "ctrl+f, /"
    );
    assert_eq!(status_hints(&overrides)[1], "ctrl+f, / find");
}

#[test]
fn unbinding_search_primary_promotes_its_synonym() {
    let overrides = BTreeMap::from([("ctrl+f".to_string(), "none".to_string())]);
    let keymap = viewer::resolve(&overrides).unwrap();

    assert_eq!(
        format_action_help(&keymap, ViewerAction::StartSearch).key_label(),
        "/"
    );
}

#[test]
fn promoted_shifted_letter_uses_the_standalone_atom() {
    let overrides = BTreeMap::from([("q".to_string(), "none".to_string())]);
    let keymap = viewer::resolve(&overrides).unwrap();

    assert_eq!(
        format_action_help(&keymap, ViewerAction::Quit).key_label(),
        "shift+q"
    );
}

#[test]
fn unmatched_page_binding_gets_its_singular_help_row() {
    let overrides = BTreeMap::from([("space".to_string(), "page-down".to_string())]);
    let keymap = viewer::resolve(&overrides).unwrap();

    let lines = format_paired_help(&keymap, &[(ViewerAction::PageUp, ViewerAction::PageDown)])
        .into_iter()
        .flat_map(|row| wrap_help_row(row, 9, 25, 2))
        .collect::<Vec<_>>();

    assert_eq!(
        lines,
        vec![
            help_line("u/d", "page up/down"),
            help_line("spc", "page down"),
        ]
    );
}

#[test]
fn wrapped_help_keeps_the_separator_without_adding_a_blank_line() {
    let overrides = BTreeMap::from([
        ("backspace".to_string(), "page-up".to_string()),
        ("space".to_string(), "page-down".to_string()),
    ]);
    let keymap = viewer::resolve(&overrides).unwrap();
    let row =
        format_paired_help(&keymap, &[(ViewerAction::PageUp, ViewerAction::PageDown)]).remove(0);

    assert_eq!(
        wrap_help_row(row, 9, 25, 2),
        vec![
            help_line("u/d,", "page up/down"),
            help_line("  bsp/spc", ""),
        ]
    );
}

fn status_hints(viewer_overrides: &BTreeMap<String, String>) -> Vec<String> {
    let app = app_with_viewer_overrides("body", viewer_overrides);
    crate::render::status_hint_segments(&app)
}

fn active_search_status_hints(viewer_overrides: &BTreeMap<String, String>) -> Vec<String> {
    let mut app = app_with_viewer_overrides("body", viewer_overrides);
    app.set_search_query("body");
    crate::render::status_hint_segments(&app)
}

#[test]
fn default_status_hints_match_upstream() {
    assert_eq!(
        status_hints(&BTreeMap::new()),
        vec!["ctrl+e edit", "ctrl+f find", "t toc", "? help", "q quit"]
    );
}

#[test]
fn configured_status_hints_use_effective_viewer_bindings() {
    let overrides = BTreeMap::from([
        ("backspace".to_string(), "page-up".to_string()),
        ("ctrl+c".to_string(), "none".to_string()),
        ("esc".to_string(), "quit".to_string()),
        ("q".to_string(), "none".to_string()),
        ("shift+q".to_string(), "none".to_string()),
        ("space".to_string(), "page-down".to_string()),
    ]);

    assert_eq!(
        status_hints(&overrides),
        vec!["ctrl+e edit", "ctrl+f find", "t toc", "? help", "esc quit",]
    );
}

#[test]
fn hostile_status_key_label_is_capped() {
    let overrides = BTreeMap::from([
        ("/".to_string(), "none".to_string()),
        ("ctrl+f".to_string(), "none".to_string()),
        ("ctrl+shift+f".to_string(), "start-search".to_string()),
    ]);

    assert_eq!(
        status_hints(&overrides),
        vec!["ctrl+e edit", "ctrl+shi… find", "t toc", "? help", "q quit"]
    );
}

#[test]
fn status_key_label_cap_has_pinned_boundaries() {
    for (configured_key, expected_hint) in [
        ("shift+esc", "shift+esc find"),
        ("shift+home", "shift+ho… find"),
    ] {
        let overrides = BTreeMap::from([
            ("/".to_string(), "none".to_string()),
            ("ctrl+f".to_string(), "none".to_string()),
            (configured_key.to_string(), "start-search".to_string()),
        ]);

        assert_eq!(status_hints(&overrides)[1], expected_hint);
    }
}

#[test]
fn active_search_status_includes_unpaired_next_and_previous_keys() {
    let overrides = BTreeMap::from([
        ("ctrl+f2".to_string(), "previous-match".to_string()),
        ("f1".to_string(), "next-match".to_string()),
        ("n".to_string(), "none".to_string()),
        ("shift+n".to_string(), "none".to_string()),
    ]);

    assert_eq!(
        active_search_status_hints(&overrides),
        vec!["f1/ctrl+f2 next/prev", "esc cancel"]
    );
}

#[test]
fn active_search_status_preserves_its_twelve_character_budget() {
    let overrides = BTreeMap::from([
        ("ctrl+shift+f1".to_string(), "next-match".to_string()),
        ("f2".to_string(), "previous-match".to_string()),
        ("n".to_string(), "none".to_string()),
        ("shift+n".to_string(), "none".to_string()),
    ]);

    assert_eq!(
        active_search_status_hints(&overrides),
        vec!["ctrl+shift+… next/prev", "esc cancel"]
    );
}
