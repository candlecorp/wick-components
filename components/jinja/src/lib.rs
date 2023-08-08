mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl render::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = render::Outputs;
    type Config = render::Config;

    async fn render(
        mut data: WickStream<Value>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let tpl = ctx.config.template.clone();
        let mut env = minijinja::Environment::new();
        env.add_template("root", &tpl).unwrap();
        let template = env.get_template("root").unwrap();
        while let Some(Ok(data)) = data.next().await {
            let rendered = template.render(data).unwrap();
            outputs.output.send(&rendered);
        }
        outputs.output.done();
        Ok(())
    }
}
