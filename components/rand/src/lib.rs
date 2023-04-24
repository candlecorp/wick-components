use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use wick::*;

#[async_trait::async_trait(?Send)]
impl OpInt for Component {
    async fn int(
        mut min: WickStream<u32>,
        mut max: WickStream<u32>,
        mut seed: WickStream<u32>,
        mut outputs: OpIntOutputs,
    ) -> wick::Result<()> {
        while let (Some(Ok(min)), Some(Ok(max)), Some(Ok(seed))) =
            (min.next().await, max.next().await, seed.next().await)
        {
            println!("Received min: {}, max: {}", min, max);
            let mut rng = SmallRng::seed_from_u64(seed as u64);
            let rando = rng.gen_range(min..max);
            outputs.output.send(&(rando));
        }
        outputs.output.done();
        Ok(())
    }
}
