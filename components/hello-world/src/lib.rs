mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl greet::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = greet::Inputs;
    type Outputs = greet::Outputs;
    type Config = greet::Config;

    async fn greet(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(name) = inputs.name.next().await {
            let name = propagate_if_error!(name.decode(), outputs, continue);
            outputs.output.send(&format!("Hello, {}", name));
        }
        outputs.output.done();
        Ok(())
    }
}
