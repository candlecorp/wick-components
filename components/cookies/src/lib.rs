mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl get::Operation for Component {
    type Error = anyhow::Error;
    type Config = get::Config;
    type Inputs = get::Inputs;
    type Outputs = get::Outputs;

    async fn get(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let cookie_name = ctx.config.name.clone();
        while let Some(cookie) = inputs.input.next().await {
            let cookie = propagate_if_error!(cookie.decode(), outputs, continue);
            let cookies = basic_cookies::Cookie::parse(&cookie)?;
            let value = cookies
                .iter()
                .find(|c| c.get_name() == cookie_name)
                .ok_or(anyhow::anyhow!("cookie {} not found", cookie_name))?;
            outputs.output.send(&value.get_value().to_owned());
        }
        outputs.output.done();
        Ok(())
    }
}
