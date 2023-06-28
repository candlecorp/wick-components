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

type HmacSha256 = Hmac<Sha256>;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl FromBytesOperation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = from_bytes::Outputs;
    type Config = from_bytes::Config;

    async fn from_bytes(
        mut input: WickStream<Bytes>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = input.next().await {
            let input = propagate_if_error!(input, outputs, continue);

            let mut mac = HmacSha256::new_from_slice(&ctx.root_config().secret)?;
            mac.update(&input);
            let result = mac.finalize().into_bytes();
            let bytes: Bytes = result.deref().to_vec().into();

            outputs.output.send(&bytes);
        }

        outputs.output.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl FromStringOperation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = from_string::Outputs;
    type Config = from_string::Config;

    async fn from_string(
        mut input: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = input.next().await {
            let input = propagate_if_error!(input, outputs, continue);

            let mut mac = HmacSha256::new_from_slice(&ctx.root_config().secret)?;
            mac.update(input.as_bytes());
            let result = mac.finalize().into_bytes();
            let bytes: Bytes = result.deref().to_vec().into();

            outputs.output.send(&bytes);
        }

        outputs.output.done();
        Ok(())
    }
}
