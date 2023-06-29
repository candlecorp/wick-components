mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl GreetOperation for Component {
    type Error = anyhow::Error;
    type Outputs = greet::Outputs;
    type Config = greet::Config;

    async fn greet(
        mut name: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let (Some(Ok(name))) = (name.next().await) {
            outputs.output.send(&format!("Hello, {}", name));
        }
        outputs.output.done();
        Ok(())
    }
}