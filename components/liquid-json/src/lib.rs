mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl render::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = render::Inputs;
    type Outputs = render::Outputs;
    type Config = render::Config;
    async fn render(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let template = ctx.config.template.clone();
        let template = liquid_json::LiquidJsonValue::from(template);

        while let Some(input) = inputs.input.next().await {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            let _ = match template.render(&input) {
                Ok(output) => outputs.output.send(&output),
                Err(e) => outputs.output.error(&e.to_string()),
            };
        }
        outputs.output.done();
        Ok(())
    }
}
