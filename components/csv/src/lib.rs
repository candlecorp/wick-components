mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}
use std::ops::Deref;

use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl ParseBytesOperation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = parse_bytes::Outputs;
    type Config = parse_bytes::Config;

    async fn parse_bytes(
        mut input: WickStream<Bytes>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = input.next().await {
            let input = propagate_if_error!(input, outputs, continue);
            let input: &[u8] = input.deref();
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input);

            outputs.output.open_bracket();
            for result in rdr.records() {
                let record = result?;

                let fields: Vec<String> = record.deserialize(None)?;
                outputs.output.send(&fields);
            }
            outputs.output.close_bracket();
        }
        outputs.output.done();
        Ok(())
    }
}
