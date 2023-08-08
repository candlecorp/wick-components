mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl get::Operation for Component {
    type Error = anyhow::Error;
    type Config = get::Config;
    type Outputs = get::Outputs;

    async fn get(
        mut input: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let cookie_name = ctx.config.name.clone();
        while let Some(cookie) = input.next().await {
            match cookie {
                Ok(cookie) => {
                    let cookies = basic_cookies::Cookie::parse(&cookie)?;
                    let value = cookies
                        .iter()
                        .find(|c| c.get_name() == cookie_name)
                        .ok_or(anyhow::anyhow!("cookie {} not found", cookie_name))?;
                    outputs.output.send(&value.get_value().to_owned());
                }
                Err(e) => {
                    anyhow::bail!(e);
                }
            }
        }
        outputs.output.done();
        Ok(())
    }
}
