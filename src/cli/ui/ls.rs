use cliclack::{intro, outro};
use crossterm::terminal;
use serde_json::Value;

use crate::prelude::*;

/// A name+description pair for display, optionally with full frontmatter + body.
pub(crate) struct ListItem {
    pub name: String,
    pub description: String,
    pub frontmatter: Value,
    pub body: Option<String>,
}

/// Detect terminal column count, falling back to 80 on error.
fn terminal_cols() -> usize {
    terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

/// Truncate `text` to `width` chars, appending `…` when truncated.
pub(crate) fn truncate_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        text.to_string()
    } else {
        let cut = width.saturating_sub(1);
        chars[..cut].iter().collect::<String>() + "…"
    }
}

/// Word-wrap `text` at `width` chars, indenting continuation lines with `indent`.
pub(crate) fn wrap_at_width(text: &str, width: usize, indent: &str) -> String {
    if width == 0 {
        return text.to_string();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = format!("{}{}", indent, word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

/// Render one section of items with cliclack log output.
fn render_section(title: &str, items: &[ListItem], full: bool, name_col: usize, cols: usize) {
    info!("{}", title);
    for item in items {
        let desc = if full {
            let indent_width = name_col + 3; // "  name   " prefix width
            let avail = cols.saturating_sub(indent_width);
            let indent = " ".repeat(indent_width);
            wrap_at_width(&item.description, avail.max(10), &indent)
        } else {
            // available width = cols - indent(2) - name_col - gap(3)
            let avail = cols.saturating_sub(2 + name_col + 3);
            truncate_to_width(&item.description, avail.max(10))
        };

        let padded_name = format!("{:<width$}", item.name, width = name_col);
        info!("  {}   {}", padded_name, desc);

        if full
            && let Some(body) = &item.body
            && !body.is_empty()
        {
            let body_indent = " ".repeat(name_col + 3);
            for line in body.lines() {
                info!("{}{}", body_indent, line);
            }
        }
    }
}

/// Render the commands listing for `dotagents commands ls`.
pub(crate) fn render_commands(items: Vec<ListItem>, full: bool) {
    if items.is_empty() {
        intro("dotagents commands ls").ok();
        info!("No commands found.");
        outro("").ok();
        return;
    }

    let name_col = items
        .iter()
        .map(|i| i.name.len())
        .max()
        .unwrap_or(10)
        .max(10);
    let cols = terminal_cols();

    intro("dotagents commands ls").ok();
    let header = format!("Commands ({})", items.len());
    render_section(&header, &items, full, name_col, cols);
    outro(format!("{} command(s)", items.len())).ok();
}

/// Render the skills listing for `dotagents skills ls`.
pub(crate) fn render_skills(items: Vec<ListItem>, full: bool) {
    if items.is_empty() {
        intro("dotagents skills ls").ok();
        info!("No skills found.");
        outro("").ok();
        return;
    }

    let name_col = items
        .iter()
        .map(|i| i.name.len())
        .max()
        .unwrap_or(10)
        .max(10);
    let cols = terminal_cols();

    intro("dotagents skills ls").ok();
    let header = format!("Skills ({})", items.len());
    render_section(&header, &items, full, name_col, cols);
    outro(format!("{} skill(s)", items.len())).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string_appends_ellipsis() {
        let result = truncate_to_width("hello world", 8);
        assert!(result.ends_with('…'));
        assert!(result.chars().count() <= 8);
    }

    #[test]
    fn truncate_zero_width_returns_empty() {
        assert_eq!(truncate_to_width("hello", 0), "");
    }

    #[test]
    fn wrap_short_text_unchanged() {
        let result = wrap_at_width("hello world", 80, "  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn wrap_long_text_breaks_at_width() {
        let result = wrap_at_width("one two three four five", 12, "  ");
        assert!(result.contains('\n'));
    }
}
