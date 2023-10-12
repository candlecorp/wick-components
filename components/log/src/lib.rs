mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl string::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = string::Inputs;
    type Outputs = string::Outputs;
    type Config = string::Config;

    async fn string(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = inputs.input.next().await {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            println!("{}", input);
            outputs.output.send(&true);
        }
        outputs.output.done();
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl object::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = object::Inputs;
    type Outputs = object::Outputs;
    type Config = object::Config;

    async fn object(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = inputs.input.next().await {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            println!("{}", input);
            outputs.output.send(&true);
        }
        outputs.output.done();
        Ok(())
    }
}
