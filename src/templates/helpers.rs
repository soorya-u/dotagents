use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    RenderError, RenderErrorReason, Renderable,
};
use serde_json;

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
        let param0 = h
            .param(0)
            .and_then(|v| Some(v.value().render()))
            .unwrap_or_default();
        let param1 = h
            .param(1)
            .and_then(|v| Some(v.value().render()))
            .unwrap_or_default();

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
