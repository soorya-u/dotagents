use serde_json::Value;

/// A name+description pair for display, optionally with full frontmatter + body.
pub(crate) struct ListItem {
    pub name: String,
    pub description: String,
    pub frontmatter: Value,
    pub body: Option<String>,
}

impl ListItem {
    /// Build a JSON array of frontmatter values, optionally including body content.
    pub(crate) fn to_json_array(items: &[Self], content: bool) -> Vec<Value> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_array_with_content_returns_content_field() {
        // includes body as content key when content=true
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body".into()),
        }];
        let result = ListItem::to_json_array(&items, true);
        assert_eq!(result[0]["content"], "body");
    }

    #[test]
    fn to_json_array_without_content_omits_content() {
        // omits content key when content=false
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("body".into()),
        }];
        let result = ListItem::to_json_array(&items, false);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_no_body_ok() {
        // does not insert content key when body is None
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: None,
        }];
        let result = ListItem::to_json_array(&items, true);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_empty_body_omits_content() {
        // does not insert content key when body is whitespace-only
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: serde_json::json!({"name": "test"}),
            body: Some("   ".into()),
        }];
        let result = ListItem::to_json_array(&items, true);
        assert!(result[0].get("content").is_none());
    }

    #[test]
    fn to_json_array_non_object_frontmatter_handled() {
        // does not panic on non-object frontmatter
        let items = vec![ListItem {
            name: "test".into(),
            description: "desc".into(),
            frontmatter: Value::String("plain".into()),
            body: Some("body".into()),
        }];
        let result = ListItem::to_json_array(&items, true);
        assert_eq!(result[0], Value::String("plain".into()));
    }
}
