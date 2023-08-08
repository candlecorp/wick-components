mod wick {
    wick_component::wick_import!();
}

use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl int::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = int::Outputs;
    type Config = int::Config;
    async fn int(
        mut min: WickStream<u32>,
        mut max: WickStream<u32>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let (Some(Ok(min)), Some(Ok(max))) = (min.next().await, max.next().await) {
            println!("Received min: {}, max: {}", min, max);
            let num: u32 = ctx.inherent.rng.gen();
            outputs.output.send(&num);
        }
        outputs.output.done();
        Ok(())
    }
}
