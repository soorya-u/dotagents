use cliclack::outro;

use crate::cli::ui::common::{
    styled_name, styled_note_name, terminal_cols, truncate_to_width, wrap_at_width,
};
use crate::prelude::*;
use crate::schema::list_item::ListItem;
use crate::utils::tty::is_tty;

/// Render one section of items with cliclack log output.
fn render_section(items: &[ListItem], content: bool, name_col: usize, cols: usize) {
    for item in items {
        if content && is_tty() {
            let body = item.body.as_deref().unwrap_or("").trim().to_string();
            if !body.is_empty() {
                let header = format!("{} — {}", styled_note_name(&item.name), item.description);
                let _ = cliclack::note(header, &body);
            } else {
                let padded = format!("{:<width$}", item.name, width = name_col);
                info!("{} — {}", styled_name(&padded), item.description);
            }
        } else {
            let desc = if content {
                let avail = cols.saturating_sub(name_col + 3);
                let indent = " ".repeat(name_col + 3);
                wrap_at_width(&item.description, avail.max(10), &indent)
            } else {
                let avail = cols.saturating_sub(name_col + 3);
                truncate_to_width(&item.description, avail.max(10))
            };

            let padded = format!("{:<width$}", item.name, width = name_col);
            info!("{} — {}", styled_name(&padded), desc);

            if content
                && let Some(body) = &item.body
                && !body.trim().is_empty()
            {
                let body_indent = " ".repeat(name_col + 3);
                let body_avail = cols.saturating_sub(name_col + 3);
                for line in body.lines() {
                    let wrapped = wrap_at_width(line, body_avail.max(10), " ");
                    for sub_line in wrapped.lines() {
                        info!("{}{}", body_indent, sub_line);
                    }
                }
            }
        }
    }
}

/// Render the commands listing for `dotagents commands ls`.
pub(crate) fn render_commands(items: Vec<ListItem>, content: bool) {
    if items.is_empty() {
        info!("No commands found.");
        outro("").ok();
        return;
    }

    let name_col = items.iter().map(|i| i.name.len()).max().unwrap_or(0);
    let cols = terminal_cols();

    render_section(&items, content, name_col, cols);
    outro(format!("{} command(s)", items.len())).ok();
}

/// Render the skills listing for `dotagents skills ls`.
pub(crate) fn render_skills(items: Vec<ListItem>, content: bool) {
    if items.is_empty() {
        info!("No skills found.");
        outro("").ok();
        return;
    }

    let name_col = items.iter().map(|i| i.name.len()).max().unwrap_or(0);
    let cols = terminal_cols();

    render_section(&items, content, name_col, cols);
    outro(format!("{} skill(s)", items.len())).ok();
}
