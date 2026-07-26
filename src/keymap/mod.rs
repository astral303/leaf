use std::collections::BTreeMap;

use crossterm::event::KeyEvent;

pub(crate) mod global;
pub(crate) mod viewer;

mod chord;
pub(crate) use chord::KeyChord;

mod help;
pub(crate) use help::{
    format_action_help, format_paired_help, format_sequence_help, wrap_help_row, HelpLine, HelpRow,
};

pub(crate) struct ActionDefinition<A> {
    pub(crate) action: A,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) help_visible: bool,
    pub(crate) paired_help_label: &'static str,
    pub(crate) singular_help_label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HelpVisibility {
    Primary,
    Synonym,
}

#[derive(Clone, Debug)]
pub(crate) struct Binding<A> {
    key: KeyChord,
    action: A,
    help_visibility: HelpVisibility,
    is_default: bool,
}

impl<A> Binding<A> {
    pub(crate) fn new(key: KeyChord, action: A, help_visibility: HelpVisibility) -> Self {
        Self {
            key,
            action,
            help_visibility,
            is_default: true,
        }
    }

    fn configured(key: KeyChord, action: A) -> Self {
        Self {
            key,
            action,
            help_visibility: HelpVisibility::Primary,
            is_default: false,
        }
    }
}

pub(crate) trait BindingAction: Copy + Eq + 'static {
    fn definitions() -> &'static [ActionDefinition<Self>];

    fn parse(value: &str) -> Option<Self> {
        let name = value.trim().to_ascii_lowercase();
        Self::definitions()
            .iter()
            .find(|definition| definition.name == name)
            .map(|definition| definition.action)
    }

    fn definition(self) -> &'static ActionDefinition<Self> {
        Self::definitions()
            .iter()
            .find(|definition| definition.action == self)
            .expect("every binding action must have a definition")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BindingSet<A> {
    name: String,
    bindings: Vec<Binding<A>>,
    configured_actions: Vec<A>,
}

impl<A: Copy + Eq> BindingSet<A> {
    pub(crate) fn new(name: &str, bindings: impl IntoIterator<Item = Binding<A>>) -> Self {
        Self {
            name: name.to_string(),
            bindings: bindings.into_iter().collect(),
            configured_actions: Vec::new(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn action_for(&self, event: &KeyEvent) -> Option<A> {
        let chord = KeyChord::from_event(event);
        self.bindings
            .iter()
            .find(|binding| binding.key == chord)
            .map(|binding| binding.action)
    }

    pub(crate) fn keys_for(&self, actions: &[A]) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|binding| actions.contains(&binding.action))
            .map(|binding| binding.key.to_string())
            .collect()
    }

    pub(crate) fn is_configured(&self, action: A) -> bool {
        self.configured_actions.contains(&action)
    }
}

impl<A: BindingAction> BindingSet<A> {
    pub(crate) fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        let (parsed, mut errors) = parse_overrides(overrides);
        for override_ in parsed {
            self.apply_override(override_);
        }

        let missing = self.unbound_action_names();
        if !missing.is_empty() {
            errors.push(format!(
                "every {} action needs a key; unbound: {}",
                self.name,
                missing.join(", ")
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format_errors(&self.name, errors))
        }
    }

    fn apply_override(&mut self, override_: ParsedOverride<'_, A>) {
        let ParsedOverride {
            key, replacement, ..
        } = override_;
        if let Some(index) = self.bindings.iter().position(|binding| binding.key == key) {
            let previous = self.bindings[index].action;
            mark_configured(&mut self.configured_actions, previous);
            if replacement == Some(previous) {
                self.bindings[index].help_visibility = HelpVisibility::Primary;
                return;
            }
            self.bindings.remove(index);
        }
        if let Some(action) = replacement {
            self.bindings.push(Binding::configured(key, action));
            mark_configured(&mut self.configured_actions, action);
        }
    }

    fn unbound_action_names(&self) -> Vec<String> {
        A::definitions()
            .iter()
            .filter(|definition| {
                !self
                    .bindings
                    .iter()
                    .any(|binding| binding.action == definition.action)
            })
            .map(|definition| definition.name.to_string())
            .collect()
    }

    fn catalog(&self, include_hidden: bool) -> KeymapCatalog<'_> {
        let rows = A::definitions()
            .iter()
            .filter(|definition| include_hidden || definition.help_visible)
            .map(|definition| CatalogRow {
                keys: self.keys_for(&[definition.action]).join(", "),
                action: definition.name,
                description: definition.description,
                configured: self.is_configured(definition.action),
            })
            .collect();
        KeymapCatalog {
            name: self.name(),
            rows,
        }
    }
}

struct ParsedOverride<'a, A> {
    key: KeyChord,
    replacement: Option<A>,
    key_name: &'a str,
}

fn parse_overrides<'a, A: BindingAction>(
    overrides: &'a BTreeMap<String, String>,
) -> (Vec<ParsedOverride<'a, A>>, Vec<String>) {
    let mut parsed = Vec::<ParsedOverride<'a, A>>::new();
    let mut errors = Vec::new();

    for (key_name, action_name) in overrides {
        let key = match KeyChord::parse(key_name) {
            Ok(key) => key,
            Err(message) => {
                errors.push(format!("'{key_name}': {message}"));
                continue;
            }
        };
        let replacement = if action_name.eq_ignore_ascii_case("none") {
            None
        } else {
            match A::parse(action_name) {
                Some(action) => Some(action),
                None => {
                    errors.push(format!("'{key_name}': unknown action '{action_name}'"));
                    continue;
                }
            }
        };

        if let Some(previous) = parsed.iter().find(|override_| override_.key == key) {
            if previous.replacement != replacement {
                errors.push(format!(
                    "'{key_name}' conflicts with equivalent key '{}'",
                    previous.key_name
                ));
            }
            continue;
        }
        parsed.push(ParsedOverride {
            key,
            replacement,
            key_name,
        });
    }

    (parsed, errors)
}

fn mark_configured<A: Copy + Eq>(configured: &mut Vec<A>, action: A) {
    if !configured.contains(&action) {
        configured.push(action);
    }
}

fn format_errors(name: &str, errors: Vec<String>) -> String {
    let details = errors
        .into_iter()
        .map(|error| format!("  - {error}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Invalid {name} keymap:\n{details}")
}

pub(crate) struct KeymapCatalog<'a> {
    name: &'a str,
    rows: Vec<CatalogRow>,
}

struct CatalogRow {
    keys: String,
    action: &'static str,
    description: &'static str,
    configured: bool,
}

impl<'a> KeymapCatalog<'a> {
    pub(crate) fn name(&self) -> &'a str {
        self.name
    }

    pub(crate) fn write_to(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let key_width = self
            .rows
            .iter()
            .fold("KEYS".len(), |width, row| width.max(row.keys.len()));
        let action_width = self
            .rows
            .iter()
            .fold("ACTION".len(), |width, row| width.max(row.action.len()));
        let description_width = self.rows.iter().fold("DESCRIPTION".len(), |width, row| {
            width.max(row.description.len())
        });

        writeln!(
            writer,
            "{:<key_width$}  {:<action_width$}  {:<description_width$}  CONFIGURED",
            "KEYS", "ACTION", "DESCRIPTION"
        )?;
        for row in &self.rows {
            let configured = if row.configured { "yes" } else { "no" };
            writeln!(
                writer,
                "{:<key_width$}  {:<action_width$}  {:<description_width$}  {configured}",
                row.keys, row.action, row.description
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Keymaps {
    global: global::GlobalKeymap,
    viewer: viewer::ViewerKeymap,
}

impl Keymaps {
    pub(crate) fn defaults() -> Self {
        Self::new(global::default_keymap(), viewer::default_keymap())
            .expect("default keymaps must not conflict")
    }

    pub(crate) fn resolve(
        global_overrides: &BTreeMap<String, String>,
        viewer_overrides: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        Self::new(
            global::resolve(global_overrides)?,
            viewer::resolve(viewer_overrides)?,
        )
    }

    fn new(global: global::GlobalKeymap, viewer: viewer::ViewerKeymap) -> Result<Self, String> {
        let mut collisions = Vec::new();
        for global_binding in &global.bindings {
            let Some(viewer_binding) = viewer
                .bindings
                .iter()
                .find(|viewer_binding| viewer_binding.key == global_binding.key)
            else {
                continue;
            };
            collisions.push(format!(
                "'{}' is bound in {} to '{}' and in {} to '{}'",
                global_binding.key,
                global.name(),
                global_binding.action.definition().name,
                viewer.name(),
                viewer_binding.action.definition().name
            ));
        }
        if !collisions.is_empty() {
            let details = collisions
                .into_iter()
                .map(|collision| format!("  - {collision}"))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("Invalid keymaps:\n{details}"));
        }
        Ok(Self { global, viewer })
    }

    pub(crate) fn global(&self) -> &global::GlobalKeymap {
        &self.global
    }

    pub(crate) fn viewer(&self) -> &viewer::ViewerKeymap {
        &self.viewer
    }

    #[cfg(test)]
    pub(crate) fn registered_names(&self) -> impl Iterator<Item = &str> {
        self.catalogs(false)
            .into_iter()
            .map(|catalog| catalog.name())
    }

    pub(crate) fn catalog(
        &self,
        name: &str,
        include_hidden: bool,
    ) -> Result<KeymapCatalog<'_>, String> {
        let catalogs = self.catalogs(include_hidden);
        let expected = catalogs
            .iter()
            .map(KeymapCatalog::name)
            .collect::<Vec<_>>()
            .join(", ");
        catalogs
            .into_iter()
            .find(|catalog| catalog.name() == name)
            .ok_or_else(|| format!("Unknown keymap: '{name}'. Expected: {expected}"))
    }

    fn catalogs(&self, include_hidden: bool) -> [KeymapCatalog<'_>; 2] {
        [
            self.global.catalog(include_hidden),
            self.viewer.catalog(include_hidden),
        ]
    }
}
