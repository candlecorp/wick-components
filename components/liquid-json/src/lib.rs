
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl RenderOperation for Component {
type Error=anyhow::Error;
 type Outputs=render::Outputs; type Config=render::Config;
    async fn render(
        mut input: WickStream<Value>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
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
