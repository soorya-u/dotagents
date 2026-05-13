use crossterm::terminal;

/// Detect terminal column count, falling back to 80 on error.
pub(crate) fn terminal_cols() -> usize {
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
