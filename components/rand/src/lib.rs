
mod wick {
    wick_component::wick_import!();
}
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use wick::*;

#[async_trait::async_trait(?Send)]
impl IntOperation for Component {
type Error=anyhow::Error;
 type Outputs=int::Outputs; type Config=int::Config;
    async fn int(
        mut min: WickStream<u32>,
        mut max: WickStream<u32>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let seed = ctx.inherent.as_ref().expect("no seed available").seed;
        while let (Some(Ok(min)), Some(Ok(max))) = (min.next().await, max.next().await) {
            println!("Received min: {}, max: {}", min, max);
            let mut rng = SmallRng::seed_from_u64(seed);
            let rando = rng.gen_range(min..max);
            outputs.output.send(&(rando));
        }
        outputs.output.done();
        Ok(())
    }
}
