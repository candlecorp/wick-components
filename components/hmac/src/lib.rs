mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}
use std::ops::Deref;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use wick::*;
use wick_component::wick_packet::{Output, Packet};

type HmacSha256 = Hmac<Sha256>;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl from_bytes::Operation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = from_bytes::Outputs;
    type Config = from_bytes::Config;

    async fn from_bytes(
        input: WickStream<Packet>,
        outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let (_, mut outputs) =
            handle_new_bytes_stream(input, outputs, &ctx.root_config().secret, true).await;
        outputs.output.done();

        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl from_string::Operation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = from_string::Outputs;
    type Config = from_string::Config;

    async fn from_string(
        input: WickStream<Packet>,
        outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let (_, mut outputs) =
            handle_new_string_stream(input, outputs, &ctx.root_config().secret, true).await;
        outputs.output.done();

        Ok(())
    }
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn handle_new_string_stream(
    mut input_stream: WickStream<Packet>,
    mut outputs: from_string::Outputs,
    secret: &Bytes,
    _start: bool,
) -> (WickStream<Packet>, from_string::Outputs) {
    while let Some(input) = input_stream.next().await {
        let input = propagate_if_error!(input, outputs, continue);
        if input.is_open_bracket() {
            outputs.broadcast_open();
            (input_stream, outputs) =
                handle_new_string_stream(input_stream, outputs, secret, false).await;
            outputs.broadcast_close()
        } else if input.is_close_bracket() || input.is_done() {
            break;
        } else {
            println!("acting on packet: {:?}", input);
            if !input.has_data() {
                continue;
            }
            let input: String = propagate_if_error!(input.decode(), outputs, continue);
            if let Err(e) = handle_packet(input.as_bytes(), &mut outputs.output, secret).await {
                outputs.output.error(&e.to_string());
            }
        }
    }

    (input_stream, outputs)
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn handle_new_bytes_stream(
    mut input_stream: WickStream<Packet>,
    mut outputs: from_bytes::Outputs,
    secret: &Bytes,
    _start: bool,
) -> (WickStream<Packet>, from_bytes::Outputs) {
    while let Some(input) = input_stream.next().await {
        let input = propagate_if_error!(input, outputs, continue);
        if input.is_open_bracket() {
            outputs.broadcast_open();
            (input_stream, outputs) =
                handle_new_bytes_stream(input_stream, outputs, secret, false).await;
            outputs.broadcast_close()
        } else if input.is_close_bracket() || input.is_done() {
            break;
        } else {
            println!("acting on packet: {:?}", input);
            if !input.has_data() {
                continue;
            }
            let input: Bytes = propagate_if_error!(input.decode(), outputs, continue);
            if let Err(e) = handle_packet(&input, &mut outputs.output, secret).await {
                outputs.output.error(&e.to_string());
            }
        }
    }

    (input_stream, outputs)
}

async fn handle_packet(
    input: &[u8],
    output: &mut Output<Bytes>,
    secret: &Bytes,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut mac = HmacSha256::new_from_slice(&secret)?;
    mac.update(input);
    let result = mac.finalize().into_bytes();
    let bytes: Bytes = result.deref().to_vec().into();

    output.send(&bytes);

    Ok(())
}
