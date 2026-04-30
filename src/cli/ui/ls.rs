use cliclack::{intro, outro};
use crossterm::terminal;

use crate::cli::options::LsOptions;
use crate::prelude::*;

/// A name+description pair for display.
pub(crate) struct ListItem {
    pub name: String,
    pub description: String,
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

/// Render one section (Skills or Commands) with cliclack log output.
fn render_section(title: &str, items: &[ListItem], opts: &LsOptions, name_col: usize, cols: usize) {
    info!("{}", title);
    for item in items {
        let desc = if opts.verbose {
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
    }
}

/// Render the full ls output using cliclack.
pub(crate) fn render_ls(skills: Vec<ListItem>, commands: Vec<ListItem>, opts: &LsOptions) {
    let show_skills = !opts.commands || opts.skills;
    let show_commands = !opts.skills || opts.commands;

    let skills_to_show: Vec<&ListItem> = if show_skills {
        skills.iter().collect()
    } else {
        vec![]
    };
    let commands_to_show: Vec<&ListItem> = if show_commands {
        commands.iter().collect()
    } else {
        vec![]
    };

    if skills_to_show.is_empty() && commands_to_show.is_empty() {
        intro("dotagents ls").ok();
        info!("No skills or commands found.");
        outro("").ok();
        return;
    }

    // Compute name column width from the longest name across all shown items.
    let name_col = skills_to_show
        .iter()
        .chain(commands_to_show.iter())
        .map(|i| i.name.len())
        .max()
        .unwrap_or(10)
        .max(10);

    let cols = terminal_cols();

    intro("dotagents ls").ok();

    if !skills_to_show.is_empty() {
        let owned: Vec<ListItem> = skills_to_show
            .into_iter()
            .map(|i| ListItem {
                name: i.name.clone(),
                description: i.description.clone(),
            })
            .collect();
        let header = format!("Skills ({})", owned.len());
        render_section(&header, &owned, opts, name_col, cols);
    }

    if !commands_to_show.is_empty() {
        let owned: Vec<ListItem> = commands_to_show
            .into_iter()
            .map(|i| ListItem {
                name: i.name.clone(),
                description: i.description.clone(),
            })
            .collect();
        let header = format!("Commands ({})", owned.len());
        render_section(&header, &owned, opts, name_col, cols);
    }

    let skill_count = skills.len();
    let cmd_count = commands.len();
    let summary = match (show_skills, show_commands) {
        (true, true) => format!("{} skill(s) · {} command(s)", skill_count, cmd_count),
        (true, false) => format!("{} skill(s)", skill_count),
        (false, true) => format!("{} command(s)", cmd_count),
        (false, false) => String::new(),
    };
    outro(summary).ok();
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
