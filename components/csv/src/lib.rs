mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}

use wick::*;
use wick_component::Bytes;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl parse_bytes::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = parse_bytes::Inputs;
    type Outputs = parse_bytes::Outputs;
    type Config = parse_bytes::Config;

    async fn parse_bytes(
        inputs: Self::Inputs,
        outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let (_, mut outputs) = handle_new_stream(inputs.input, outputs, &_ctx, true).await;
        outputs.output.done();

        Ok(())
    }
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn handle_new_stream<T: Stream<Item = VPacket<Bytes>> + Unpin>(
    mut input_stream: T,
    mut outputs: parse_bytes::Outputs,
    _ctx: &Context<parse_bytes::Config>,
    _start: bool,
) -> (T, parse_bytes::Outputs) {
    while let Some(input) = input_stream.next().await {
        if input.is_open_bracket() {
            outputs.broadcast_open();
            (input_stream, outputs) = handle_new_stream(input_stream, outputs, _ctx, false).await;
            outputs.broadcast_close();
        } else if input.is_close_bracket() || input.is_done() {
            break;
        } else {
            println!("acting on packet: {:?}", input);
            if !input.has_data() {
                continue;
            }
            let input = propagate_if_error!(input.decode(), outputs, continue);
            if let Err(e) = handle_packet(&input, &mut outputs).await {
                outputs.output.error(&e.to_string());
            }
        }
    }

    (input_stream, outputs)
}

async fn handle_packet(
    input: &Bytes,
    outputs: &mut parse_bytes::Outputs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(input.as_ref());

    outputs.output.open_bracket();
    for result in rdr.records() {
        let record = result.unwrap();

        let fields: Vec<String> = record.deserialize(None).unwrap();
        outputs.output.send(&fields);
    }
    outputs.output.close_bracket();

    Ok(())
}
