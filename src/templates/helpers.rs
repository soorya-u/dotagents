use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    RenderError, RenderErrorReason, Renderable,
};

#[derive(Clone, Copy)]
pub struct IfEqHelper;

impl HelperDef for IfEqHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param0 = h.param(0).map(|v| v.value().render()).unwrap_or_default();
        let param1 = h.param(1).map(|v| v.value().render()).unwrap_or_default();

        if param0 == param1 {
            if let Some(template) = h.template() {
                template.render(r, ctx, rc, out)?;
            }
        } else if let Some(inverse) = h.inverse() {
            inverse.render(r, ctx, rc, out)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct IfDefinedHelper;

impl HelperDef for IfDefinedHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).map(|v| v.value());
        let is_defined = param.map(|v| !v.is_null()).unwrap_or(false);

        if is_defined {
            if let Some(template) = h.template() {
                template.render(r, ctx, rc, out)?;
            }
        } else if let Some(inverse) = h.inverse() {
            inverse.render(r, ctx, rc, out)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct JsonHelper;

impl HelperDef for JsonHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).ok_or_else(|| {
            RenderError::from(RenderErrorReason::ParamNotFoundForIndex("json", 0))
        })?;

        let json_string = serde_json::to_string(param.value())
            .map_err(|e| RenderError::from(RenderErrorReason::NestedError(Box::new(e))))?;

        out.write(&json_string)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct TomlHelper;

impl HelperDef for TomlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).ok_or_else(|| {
            RenderError::from(RenderErrorReason::ParamNotFoundForIndex("toml", 0))
        })?;

        let value = param.value();
        if !value.is_object() {
            return Err(RenderError::from(RenderErrorReason::NestedError(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "{{toml}} helper only supports objects",
                ),
            ))));
        }

        let toml_string = toml::to_string(value)
            .map_err(|e| RenderError::from(RenderErrorReason::NestedError(Box::new(e))))?;

        out.write(&toml_string)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct TomlInlineHelper;

impl HelperDef for TomlInlineHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).ok_or_else(|| {
            RenderError::from(RenderErrorReason::ParamNotFoundForIndex("toml-inline", 0))
        })?;

        let value = param.value();
        if !value.is_object() {
            return Err(RenderError::from(RenderErrorReason::NestedError(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "{{toml-inline}} helper only supports objects",
                ),
            ))));
        }

        let toml_string = toml::to_string(value)
            .map_err(|e| RenderError::from(RenderErrorReason::NestedError(Box::new(e))))?;

        let inline = format!("{{ {} }}", toml_string.trim());
        out.write(&inline)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct YamlHelper;

impl HelperDef for YamlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).ok_or_else(|| {
            RenderError::from(RenderErrorReason::ParamNotFoundForIndex("yaml", 0))
        })?;

        let yaml_string = serde_yaml::to_string(param.value())
            .map_err(|e| RenderError::from(RenderErrorReason::NestedError(Box::new(e))))?;

        out.write(&yaml_string)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct SnakeCaseHelper;

impl HelperDef for SnakeCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).ok_or_else(|| {
            RenderError::from(RenderErrorReason::ParamNotFoundForIndex("snake-case", 0))
        })?;

        let value = param.value().as_str().ok_or_else(|| {
            RenderError::from(RenderErrorReason::NestedError(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "{{snake-case}} helper only supports strings",
                ),
            )))
        })?;

        out.write(&to_snake_case(value))?;
        Ok(())
    }
}

fn to_snake_case(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut result = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch == '-' || ch == ' ' {
            if !result.ends_with('_') && !result.is_empty() {
                result.push('_');
            }
            continue;
        }

        if ch == '_' {
            if !result.ends_with('_') {
                result.push('_');
            }
            continue;
        }

        if ch.is_uppercase() {
            let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            let next = chars.get(index + 1).copied();
            let needs_separator = previous.is_some_and(|prev| {
                (prev.is_lowercase() || prev.is_ascii_digit())
                    || (prev.is_uppercase() && next.is_some_and(|next| next.is_lowercase()))
            });

            if needs_separator && !result.ends_with('_') {
                result.push('_');
            }

            for lower in ch.to_lowercase() {
                result.push(lower);
            }
        } else {
            result.push(ch);
        }
    }

    result.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;
    use serde_json::json;

    #[test]
    fn test_if_eq_helper_equal_strings() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifEq", Box::new(IfEqHelper));

        let template = "{{#ifEq name \"Alice\"}}Hello Alice{{else}}Hello stranger{{/ifEq}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "Alice"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "Hello Alice");
    }

    #[test]
    fn test_if_eq_helper_not_equal() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifEq", Box::new(IfEqHelper));

        let template = "{{#ifEq name \"Alice\"}}Hello Alice{{else}}Hello stranger{{/ifEq}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "Bob"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "Hello stranger");
    }

    #[test]
    fn test_if_eq_helper_equal_numbers() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifEq", Box::new(IfEqHelper));

        let template = "{{#ifEq age 30}}Age is 30{{else}}Age is not 30{{/ifEq}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"age": 30});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "Age is 30");
    }

    #[test]
    fn test_if_eq_helper_no_else_branch() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifEq", Box::new(IfEqHelper));

        let template = "{{#ifEq name \"Alice\"}}Hello Alice{{/ifEq}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "Bob"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_json_helper_object() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("json", Box::new(JsonHelper));

        let template = "{{json user}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"user": {"name": "Alice", "age": 30}});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, r#"{"age":30,"name":"Alice"}"#);
    }

    #[test]
    fn test_json_helper_array() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("json", Box::new(JsonHelper));

        let template = "{{json items}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"items": [1, 2, 3]});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_json_helper_string() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("json", Box::new(JsonHelper));

        let template = "{{json name}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "Alice"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, r#""Alice""#);
    }

    #[test]
    fn test_json_helper_number() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("json", Box::new(JsonHelper));

        let template = "{{json age}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"age": 42});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_toml_helper_object() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("toml", Box::new(TomlHelper));

        let template = "{{toml env}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"env": {"KEY": "val", "FOO": "bar"}});
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("KEY = \"val\""));
        assert!(result.contains("FOO = \"bar\""));
    }

    #[test]
    fn test_toml_helper_errors_on_string() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("toml", Box::new(TomlHelper));

        let template = "{{toml name}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "hello"});
        let result = handlebars.render("test", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_helper_errors_on_array() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("toml", Box::new(TomlHelper));

        let template = "{{toml items}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"items": [1, 2, 3]});
        let result = handlebars.render("test", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_inline_helper_object() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("toml-inline", Box::new(TomlInlineHelper));

        let template = "env = {{toml-inline env}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"env": {"KEY": "val", "FOO": "bar"}});
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.starts_with("env = {"));
        assert!(result.ends_with("}"));
        assert!(result.contains("KEY = \"val\""));
    }

    #[test]
    fn test_toml_inline_helper_errors_on_null() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("toml-inline", Box::new(TomlInlineHelper));

        let template = "{{toml-inline value}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"value": null});
        let result = handlebars.render("test", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_yaml_helper_object() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("yaml", Box::new(YamlHelper));

        let template = "{{yaml env}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"env": {"KEY": "val", "FOO": "bar"}});
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("KEY: val"));
        assert!(result.contains("FOO: bar"));
    }

    #[test]
    fn test_yaml_helper_array() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("yaml", Box::new(YamlHelper));

        let template = "{{yaml items}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"items": [1, 2, 3]});
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("- 1"));
        assert!(result.contains("- 2"));
        assert!(result.contains("- 3"));
    }

    #[test]
    fn test_yaml_helper_string() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("yaml", Box::new(YamlHelper));

        let template = "{{yaml name}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": "hello"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn test_if_defined_helper_true_value() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined flag}}yes{{else}}no{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"flag": true});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_if_defined_helper_false_value() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined flag}}flag: {{flag}}{{else}}omitted{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"flag": false});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "flag: false");
    }

    #[test]
    fn test_if_defined_helper_undefined() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined missing}}yes{{else}}no{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_if_defined_helper_null() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined value}}yes{{else}}no{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"value": null});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_if_defined_helper_zero() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined count}}count: {{count}}{{else}}omitted{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"count": 0});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "count: 0");
    }

    #[test]
    fn test_if_defined_helper_empty_string() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("ifDefined", Box::new(IfDefinedHelper));

        let template = "{{#ifDefined name}}name: {{name}}{{else}}omitted{{/ifDefined}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"name": ""});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "name: ");
    }

    #[test]
    fn test_snake_case_helper_camel_case() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake-case", Box::new(SnakeCaseHelper));

        let template = "{{snake-case field}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"field": "startupTimeoutSec"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "startup_timeout_sec");
    }

    #[test]
    fn test_snake_case_helper_pascal_case() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake-case", Box::new(SnakeCaseHelper));

        let template = "{{snake-case field}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"field": "BearerTokenEnvVar"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "bearer_token_env_var");
    }

    #[test]
    fn test_snake_case_helper_preserves_snake_case() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake-case", Box::new(SnakeCaseHelper));

        let template = "{{snake-case field}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"field": "tool_timeout_sec"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "tool_timeout_sec");
    }

    #[test]
    fn test_snake_case_helper_errors_on_non_string() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake-case", Box::new(SnakeCaseHelper));

        let template = "{{snake-case value}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({"value": 42});
        let result = handlebars.render("test", &data);
        assert!(result.is_err());
    }
}
