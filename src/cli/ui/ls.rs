use cliclack::outro;
use console::style;
use crossterm::terminal;
use serde_json::Value;

use crate::prelude::*;
use crate::utils::tty::is_tty;

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

/// Build a JSON array of frontmatter values, optionally including body content.
pub(crate) fn to_json_array(items: &[ListItem], content: bool) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            let mut obj = item.frontmatter.clone();
            if content
                && let Some(body) = &item.body
                && !body.trim().is_empty()
                && let Some(map) = obj.as_object_mut()
            {
                map.insert("content".to_string(), Value::String(body.clone()));
            }
            obj
        })
        .collect()
}

/// Apply cyan+bold styling to a name when colors are enabled (console handles detection).
fn styled_name(name: &str) -> String {
    style(name).cyan().bold().to_string()
}

/// Render one section of items with cliclack log output.
fn render_section(items: &[ListItem], content: bool, name_col: usize, cols: usize) {
    for item in items {
        if content && is_tty() {
            let body = item.body.as_deref().unwrap_or("").trim().to_string();
            if !body.is_empty() {
                let header = format!(
                    "{} — {}",
                    style(&item.name).green().bold(),
                    item.description
                );
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

    #[test]
    fn to_json_array_with_content_returns_content_field() {
        // to_json_array with content=true includes body as content key
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body".into()),
        }];
        let result = to_json_array(&items, true);
        assert_eq!(result[0]["content"], "body");
    }

    #[test]
    fn to_json_array_without_content_omits_content() {
        // to_json_array with content=false omits content key
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body".into()),
        }];
        let result = to_json_array(&items, false);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_no_body_ok() {
        // to_json_array works when body is None
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: None,
        }];
        let result = to_json_array(&items, true);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_non_object_frontmatter_handled() {
        // to_json_array does not panic on non-object frontmatter
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: Value::String("plain".into()),
            body: Some("body".into()),
        }];
        let result = to_json_array(&items, true);
        assert_eq!(result[0], Value::String("plain".into()));
    }
}
