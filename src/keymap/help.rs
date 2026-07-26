use crossterm::event::{KeyCode, KeyModifiers};

use super::{
    chord::{grouped_atom, standalone_atom},
    Binding, BindingAction, BindingSet, HelpVisibility, KeyChord,
};

#[derive(Clone, Copy, Debug)]
struct IndexedHelpBinding {
    index: usize,
    key: KeyChord,
    is_default: bool,
}

impl IndexedHelpBinding {
    fn new<A>(index: usize, binding: &Binding<A>) -> Self {
        Self {
            index,
            key: binding.key,
            is_default: binding.is_default,
        }
    }
}

impl<A: Copy + Eq> BindingSet<A> {
    fn help_bindings_for(&self, action: A) -> Vec<IndexedHelpBinding> {
        let primary = self
            .bindings
            .iter()
            .enumerate()
            .filter(|(_, binding)| {
                binding.action == action && binding.help_visibility == HelpVisibility::Primary
            })
            .map(|(index, binding)| IndexedHelpBinding::new(index, binding))
            .collect::<Vec<_>>();
        if !primary.is_empty() {
            return primary;
        }

        self.bindings
            .iter()
            .enumerate()
            .find(|(_, binding)| {
                binding.action == action && binding.help_visibility == HelpVisibility::Synonym
            })
            .map(|(index, binding)| vec![IndexedHelpBinding::new(index, binding)])
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HelpPart {
    text: String,
    separator_after: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpRow {
    parts: Vec<HelpPart>,
    description: &'static str,
}

impl HelpRow {
    fn from_groups(
        groups: Vec<String>,
        separator: &'static str,
        description: &'static str,
    ) -> Self {
        let last = groups.len().saturating_sub(1);
        let parts = groups
            .into_iter()
            .enumerate()
            .map(|(index, text)| HelpPart {
                text,
                separator_after: if index == last { "" } else { separator },
            })
            .collect();
        Self { parts, description }
    }

    pub(crate) fn key_label(&self) -> String {
        self.parts
            .iter()
            .map(|part| format!("{}{}", part.text, part.separator_after))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HelpLine {
    pub(crate) keys: String,
    pub(crate) description: &'static str,
}

pub(crate) fn format_action_help<A: BindingAction>(keymap: &BindingSet<A>, action: A) -> HelpRow {
    HelpRow::from_groups(
        group_action_bindings(&keymap.help_bindings_for(action)),
        ", ",
        action.definition().singular_help_label,
    )
}

pub(crate) fn format_paired_help<A: BindingAction>(
    keymap: &BindingSet<A>,
    action_pairs: &[(A, A)],
) -> Vec<HelpRow> {
    let Some((first_action, _)) = action_pairs.first() else {
        return Vec::new();
    };

    let mut paired_groups = Vec::new();
    let mut leftover_rows = Vec::new();
    for &(first_action, second_action) in action_pairs {
        let first = keymap.help_bindings_for(first_action);
        let second = keymap.help_bindings_for(second_action);
        let (groups, first_leftovers, second_leftovers) = pair_bindings(&first, &second);
        paired_groups.extend(groups);
        push_leftover_row(&mut leftover_rows, first_action, first_leftovers);
        push_leftover_row(&mut leftover_rows, second_action, second_leftovers);
    }

    let mut rows = Vec::new();
    if !paired_groups.is_empty() {
        rows.push(HelpRow::from_groups(
            paired_groups,
            ", ",
            first_action.definition().paired_help_label,
        ));
    }
    rows.extend(leftover_rows);
    rows
}

pub(crate) fn format_sequence_help<A: BindingAction>(
    keymap: &BindingSet<A>,
    prefix_action: A,
    target_actions: &[A],
) -> HelpRow {
    let prefixes = keymap.help_bindings_for(prefix_action);
    let target_groups = sequence_target_groups(keymap, target_actions);
    let mut parts = Vec::new();

    for (index, target_group) in target_groups.iter().enumerate() {
        let prefix = prefixes
            .iter()
            .find(|prefix| prefix.key.modifiers() == target_group.modifiers)
            .or_else(|| prefixes.first());
        let Some(prefix) = prefix else {
            continue;
        };
        parts.push(HelpPart {
            text: target_group.text.clone(),
            separator_after: "/",
        });
        parts.push(HelpPart {
            text: format!("{}»{}", grouped_atom(&prefix.key), target_group.text),
            separator_after: if index + 1 == target_groups.len() {
                ""
            } else {
                ", "
            },
        });
    }

    HelpRow {
        parts,
        description: prefix_action.definition().paired_help_label,
    }
}

pub(crate) fn wrap_help_row(
    row: HelpRow,
    first_line_width: usize,
    continuation_line_width: usize,
    continuation_indent: usize,
) -> Vec<HelpLine> {
    if row.key_label().chars().count() <= first_line_width {
        return vec![HelpLine {
            keys: row.key_label(),
            description: row.description,
        }];
    }

    let mut key_lines = Vec::new();
    let mut current = String::new();
    let mut width = first_line_width;
    let continuation_content_width = continuation_line_width.saturating_sub(continuation_indent);
    for part in row.parts {
        let text = format!("{}{}", part.text, part.separator_after);
        if should_wrap_after_part(&current, &text, width) {
            current.push_str(text.trim_end());
            key_lines.push(std::mem::take(&mut current));
            width = continuation_content_width;
            continue;
        }
        if !current.is_empty() && current.chars().count() + text.chars().count() > width {
            key_lines.push(current.trim_end().to_string());
            current.clear();
            width = continuation_content_width;
        }
        if current.is_empty() && key_lines.is_empty() && text.chars().count() > width {
            key_lines.push(String::new());
            width = continuation_content_width;
        }

        let mut fragments = split_overlong_help_part(&text, width);
        if fragments.len() == 1 {
            current.push_str(&text);
            continue;
        }

        let last = fragments
            .pop()
            .expect("split help part has a last fragment");
        key_lines.extend(
            fragments
                .into_iter()
                .map(|fragment| fragment.trim_end().to_string()),
        );
        current = last;
    }
    if !current.is_empty() {
        key_lines.push(current.trim_end().to_string());
    }

    key_lines
        .into_iter()
        .enumerate()
        .map(|(index, keys)| HelpLine {
            keys: if index == 0 {
                keys
            } else {
                format!("{}{keys}", " ".repeat(continuation_indent))
            },
            description: if index == 0 { row.description } else { "" },
        })
        .collect()
}

fn should_wrap_after_part(current: &str, part: &str, width: usize) -> bool {
    let part_at_line_end = part.trim_end();
    if part_at_line_end == part {
        return false;
    }

    let current_width = current.chars().count();
    let inline_width = current_width + part.chars().count();
    let line_end_width = current_width + part_at_line_end.chars().count();
    inline_width > width && line_end_width <= width
}

fn split_overlong_help_part(text: &str, width: usize) -> Vec<String> {
    if text.chars().count() <= width || !text.contains('+') {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for fragment in text.split_inclusive('+') {
        if !current.is_empty() && current.chars().count() + fragment.chars().count() > width {
            lines.push(current);
            current = String::new();
        }
        current.push_str(fragment);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn push_leftover_row<A: BindingAction>(
    rows: &mut Vec<HelpRow>,
    action: A,
    bindings: Vec<IndexedHelpBinding>,
) {
    if bindings.is_empty() {
        return;
    }
    rows.push(HelpRow::from_groups(
        group_action_bindings(&bindings),
        ", ",
        action.definition().singular_help_label,
    ));
}

fn pair_bindings(
    first: &[IndexedHelpBinding],
    second: &[IndexedHelpBinding],
) -> (
    Vec<String>,
    Vec<IndexedHelpBinding>,
    Vec<IndexedHelpBinding>,
) {
    let mut first_used = vec![false; first.len()];
    let mut second_used = vec![false; second.len()];
    let mut groups = Vec::new();

    while let Some((first_index, second_index)) =
        earliest_default_pair(first, second, &first_used, &second_used)
    {
        first_used[first_index] = true;
        second_used[second_index] = true;
        let first_binding = first[first_index];
        let second_binding = second[second_index];
        groups.push(if first_binding.index <= second_binding.index {
            pair_label(&first_binding.key, &second_binding.key)
        } else {
            pair_label(&second_binding.key, &first_binding.key)
        });
    }

    for (first_index, first_binding) in first.iter().enumerate() {
        if first_used[first_index] {
            continue;
        }
        let Some(second_index) =
            second
                .iter()
                .enumerate()
                .position(|(second_index, second_binding)| {
                    !second_used[second_index]
                        && same_pair_family(&first_binding.key, &second_binding.key)
                })
        else {
            continue;
        };
        first_used[first_index] = true;
        second_used[second_index] = true;
        groups.push(pair_label(&first_binding.key, &second[second_index].key));
    }

    let first_leftovers = unpaired_bindings(first, &first_used);
    let second_leftovers = unpaired_bindings(second, &second_used);
    (groups, first_leftovers, second_leftovers)
}

fn unpaired_bindings(
    bindings: &[IndexedHelpBinding],
    is_paired: &[bool],
) -> Vec<IndexedHelpBinding> {
    debug_assert_eq!(bindings.len(), is_paired.len());

    bindings
        .iter()
        .zip(is_paired)
        .filter_map(|(binding, &is_paired)| (!is_paired).then_some(*binding))
        .collect()
}

fn earliest_default_pair(
    first: &[IndexedHelpBinding],
    second: &[IndexedHelpBinding],
    first_used: &[bool],
    second_used: &[bool],
) -> Option<(usize, usize)> {
    let mut earliest = None;
    for (first_index, first_binding) in first.iter().enumerate() {
        if first_used[first_index] || !first_binding.is_default {
            continue;
        }
        for (second_index, second_binding) in second.iter().enumerate() {
            if second_used[second_index]
                || !second_binding.is_default
                || !same_pair_family(&first_binding.key, &second_binding.key)
            {
                continue;
            }
            let declaration_index = first_binding.index.min(second_binding.index);
            if earliest.is_none_or(|(_, _, earliest_index)| declaration_index < earliest_index) {
                earliest = Some((first_index, second_index, declaration_index));
            }
        }
    }
    earliest.map(|(first_index, second_index, _)| (first_index, second_index))
}

fn pair_label(first: &KeyChord, second: &KeyChord) -> String {
    let separator = if matches!(first.code(), KeyCode::Char('/'))
        || matches!(second.code(), KeyCode::Char('/'))
    {
        ", "
    } else {
        "/"
    };
    let first = grouped_atom(first);
    let second = grouped_atom(second);
    format!("{first}{separator}{second}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairFamily {
    Letter,
    Arrow,
    Special,
}

fn same_pair_family(first: &KeyChord, second: &KeyChord) -> bool {
    first.modifiers() == second.modifiers()
        && pair_family(first.code()) == pair_family(second.code())
}

fn pair_family(code: KeyCode) -> PairFamily {
    match code {
        KeyCode::Char(character) if character.is_ascii_alphabetic() => PairFamily::Letter,
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => PairFamily::Arrow,
        _ => PairFamily::Special,
    }
}

fn group_action_bindings(bindings: &[IndexedHelpBinding]) -> Vec<String> {
    let collapsed = collapse_ranges(bindings);
    let mut used = vec![false; collapsed.len()];
    let mut groups = Vec::new();

    for (index, atom) in collapsed.iter().enumerate() {
        if used[index] {
            continue;
        }
        if let Some(key) = atom.key {
            if let Some(pair_index) = collapsed
                .iter()
                .enumerate()
                .skip(index + 1)
                .find(|(pair_index, candidate)| {
                    !used[*pair_index]
                        && candidate
                            .key
                            .is_some_and(|candidate| is_shift_pair(&key, &candidate))
                })
                .map(|(pair_index, _)| pair_index)
            {
                used[index] = true;
                used[pair_index] = true;
                groups.push(pair_label(
                    &key,
                    &collapsed[pair_index]
                        .key
                        .expect("a paired atom must contain a key"),
                ));
                continue;
            }
            groups.push(standalone_atom(&key));
        } else {
            groups.push(atom.text.clone());
        }
        used[index] = true;
    }
    groups
}

#[derive(Clone, Debug)]
struct CollapsedAtom {
    text: String,
    key: Option<KeyChord>,
}

fn collapse_ranges(bindings: &[IndexedHelpBinding]) -> Vec<CollapsedAtom> {
    let mut collapsed = Vec::new();
    let mut start = 0;
    while start < bindings.len() {
        let mut end = start + 1;
        while end < bindings.len()
            && consecutive_range_chords(&bindings[end - 1].key, &bindings[end].key)
        {
            end += 1;
        }

        if end - start >= 3 {
            collapsed.push(CollapsedAtom {
                text: range_label(&bindings[start].key, &bindings[end - 1].key),
                key: None,
            });
            start = end;
        } else {
            collapsed.push(CollapsedAtom {
                text: grouped_atom(&bindings[start].key),
                key: Some(bindings[start].key),
            });
            start += 1;
        }
    }
    collapsed
}

fn consecutive_range_chords(first: &KeyChord, second: &KeyChord) -> bool {
    if first.modifiers() != second.modifiers() {
        return false;
    }
    let (KeyCode::Char(first), KeyCode::Char(second)) = (first.code(), second.code()) else {
        return false;
    };
    same_character_class(first, second) && second as u32 == first as u32 + 1
}

fn same_character_class(first: char, second: char) -> bool {
    (first.is_ascii_digit() && second.is_ascii_digit())
        || (first.is_ascii_lowercase() && second.is_ascii_lowercase())
        || (first.is_ascii_uppercase() && second.is_ascii_uppercase())
}

fn range_label(first: &KeyChord, last: &KeyChord) -> String {
    let KeyCode::Char(last_character) = last.code() else {
        unreachable!("range endpoints are characters")
    };
    format!("{}-{last_character}", grouped_atom(first))
}

fn is_shift_pair(first: &KeyChord, second: &KeyChord) -> bool {
    if first.modifiers() != second.modifiers() {
        return false;
    }
    let (KeyCode::Char(first), KeyCode::Char(second)) = (first.code(), second.code()) else {
        return false;
    };
    first.is_ascii_alphabetic()
        && second.is_ascii_alphabetic()
        && first.eq_ignore_ascii_case(&second)
        && first.is_ascii_lowercase() != second.is_ascii_lowercase()
}

#[derive(Clone, Debug)]
struct SequenceTargetGroup {
    text: String,
    modifiers: KeyModifiers,
}

fn sequence_target_groups<A: BindingAction>(
    keymap: &BindingSet<A>,
    actions: &[A],
) -> Vec<SequenceTargetGroup> {
    let bindings_by_action = actions
        .iter()
        .map(|action| keymap.help_bindings_for(*action))
        .collect::<Vec<_>>();
    let mut modifier_sets = Vec::<(KeyModifiers, usize)>::new();
    for bindings in &bindings_by_action {
        for binding in bindings {
            if let Some((_, first_index)) = modifier_sets
                .iter_mut()
                .find(|(modifiers, _)| *modifiers == binding.key.modifiers())
            {
                *first_index = (*first_index).min(binding.index);
            } else {
                modifier_sets.push((binding.key.modifiers(), binding.index));
            }
        }
    }
    modifier_sets.sort_by_key(|(_, first_index)| *first_index);

    modifier_sets
        .into_iter()
        .flat_map(|(modifiers, _)| {
            let mut groups = Vec::new();
            let mut run = Vec::new();
            for bindings in &bindings_by_action {
                if let Some(binding) = bindings
                    .iter()
                    .find(|binding| binding.key.modifiers() == modifiers)
                {
                    run.push(*binding);
                } else {
                    push_sequence_run(&mut groups, &run, modifiers);
                    run.clear();
                }
            }
            push_sequence_run(&mut groups, &run, modifiers);
            groups
        })
        .collect()
}

fn push_sequence_run(
    groups: &mut Vec<SequenceTargetGroup>,
    run: &[IndexedHelpBinding],
    modifiers: KeyModifiers,
) {
    groups.extend(collapse_ranges(run).into_iter().map(|atom| {
        let text = atom.key.map(|key| grouped_atom(&key)).unwrap_or(atom.text);
        SequenceTargetGroup { text, modifiers }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuation_indent_reduces_the_available_text_width() {
        let row = HelpRow::from_groups(
            vec!["a".to_string(), "bb".to_string(), "ccc".to_string()],
            ", ",
            "label",
        );

        assert_eq!(
            wrap_help_row(row, 4, 8, 2),
            vec![
                HelpLine {
                    keys: "a,".to_string(),
                    description: "label",
                },
                HelpLine {
                    keys: "  bb,".to_string(),
                    description: "",
                },
                HelpLine {
                    keys: "  ccc".to_string(),
                    description: "",
                },
            ]
        );
    }
}
