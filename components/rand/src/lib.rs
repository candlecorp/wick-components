mod wick {
    wick_component::wick_import!();
}

use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl int::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = int::Inputs;
    type Outputs = int::Outputs;
    type Config = int::Config;
    async fn int(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let (Some(min), Some(max)) = (inputs.min.next().await, inputs.max.next().await) {
            let num: u32 = ctx.inherent.rng.range(min.decode()?, max.decode()?);
            outputs.output.send(&num);
        }
        outputs.output.done();
        Ok(())
    }
}
