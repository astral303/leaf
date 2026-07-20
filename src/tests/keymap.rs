use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keymap::viewer::{self, ViewerAction};
use crate::{app::App, markdown::parse_markdown};
use ratatui::{backend::TestBackend, Terminal};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
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
fn viewer_shortcuts_require_exact_modifiers() {
    let keymap = viewer::default_keymap();

    assert_eq!(
        keymap.action_for(&key(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        )),
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
fn status_and_help_use_effective_viewer_bindings() {
    let overrides = BTreeMap::from([
        ("ctrl+c".to_string(), "none".to_string()),
        ("esc".to_string(), "quit".to_string()),
        ("q".to_string(), "none".to_string()),
        ("shift+q".to_string(), "none".to_string()),
        ("space".to_string(), "page-down".to_string()),
    ]);
    let keymap = viewer::resolve(&overrides).unwrap();
    let (ss, theme) = super::test_assets();
    let (lines, toc, _, _) =
        parse_markdown("body", &ss, &theme, &super::test_md_theme(), false, true).into();
    let mut app = App::new(lines, toc, "test".to_string(), false, false, None, None);
    app.set_viewer_keymap(keymap);

    assert!(crate::render::status_hint_segments(&app)
        .iter()
        .any(|hint| hint == "esc quit"));

    app.open_help();
    let backend = TestBackend::new(90, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| crate::render::ui(frame, &mut app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let mut rendered = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            rendered.push_str(buffer.cell((x, y)).unwrap().symbol());
        }
        rendered.push('\n');
    }
    assert!(rendered.contains("space"));
    assert!(rendered.contains("esc"));
}
