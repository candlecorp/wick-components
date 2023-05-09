use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpRender for Component {
    async fn render(
        mut input: WickStream<Value>,
        mut outputs: OpRenderOutputs,
        ctx: Context<OpRenderConfig>,
    ) -> wick::Result<()> {
        let template = ctx.config.template.clone();
        println!("Template: {:?}", template);
        let template = liquid_json::LiquidJsonValue::from(template);

        while let Some(Ok(input)) = input.next().await {
            let _ = match template.render(&input) {
                Ok(output) => outputs.output.send(&output),
                Err(e) => outputs.output.error(&e.to_string()),
            };
        }
        outputs.output.done();
        Ok(())
    }
}
