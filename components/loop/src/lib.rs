mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl EachOperation for Component {
    type Error = anyhow::Error;
    type Outputs = each::Outputs;
    type Config = each::Config;

    async fn each(
        mut input: WickStream<Value>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = input.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input, outputs, continue);

            match input {
                Value::Array(arr) => {
                    for item in arr {
                        outputs.output.send(&(item));
                    }
                }
                _ => outputs.output.send(&(input)),
            }
        }
        outputs.output.done();
        Ok(())
    }
}
