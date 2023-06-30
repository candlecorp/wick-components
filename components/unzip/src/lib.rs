mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}

use wick::*;
use wick_component::packet::Packet;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl UnzipOperation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = unzip::Outputs;
    type Config = unzip::Config;

    async fn unzip(
        input: WickStream<Packet>,
        outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let (_, mut outputs) = handle_new_stream(input, outputs, &_ctx, true).await;
        outputs.filename.done();
        outputs.contents.done();

        Ok(())
    }
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn handle_new_stream(
    mut input_stream: WickStream<Packet>,
    mut outputs: unzip::Outputs,
    _ctx: &Context<unzip::Config>,
    start: bool,
) -> (WickStream<Packet>, unzip::Outputs) {
    let mut zip = stream_unzip::ZipReader::new();
    if start {
        outputs.broadcast_open();
    }

    while let Some(input) = input_stream.next().await {
        let input = propagate_if_error!(input, outputs, continue);
        if input.is_open_bracket() {
            if !start {
                outputs.broadcast_open();
            }
            (input_stream, outputs) = handle_new_stream(input_stream, outputs, _ctx, false).await;
            if !start {
                outputs.broadcast_close();
            }
        } else if input.is_close_bracket() || input.is_done() {
            break;
        } else {
            println!("acting on packet: {:?}", input);
            if !input.has_data() {
                continue;
            }
            let input: Bytes = propagate_if_error!(input.decode(), outputs, continue);

            zip.update(input.into());
            let entries = zip.drain_entries();
            for entry in entries {
                let expanded = entry.inflate().unwrap();

                let filename = expanded.name().to_owned();

                let (_, data) = expanded.into_parts();
                outputs.filename.send(&filename);
                outputs.contents.send(&Bytes::new(data));
            }
        }
    }

    if start {
        outputs.broadcast_close();
    }

    (input_stream, outputs)
}
