mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl ConcatenateOperation for Component {
    type Error = anyhow::Error;
    type Outputs = concatenate::Outputs;
    type Config = concatenate::Config;

    async fn concatenate(
        mut left: WickStream<String>,
        mut right: WickStream<String>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let (Some(left), Some(right)) = (left.next().await, right.next().await) {
            let left = propagate_if_error!(left, outputs, continue);
            let right = propagate_if_error!(right, outputs, continue);
            outputs.output.send(&(left + &right));
        }
        outputs.output.done();
        Ok(())
    }
}
