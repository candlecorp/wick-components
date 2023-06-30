mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}

use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl UnzipOperation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = unzip::Outputs;
    type Config = unzip::Config;

    async fn unzip(
        mut input: WickStream<Bytes>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let mut zip = stream_unzip::ZipReader::new();
        while let Some(input) = input.next().await {
            let input = propagate_if_error!(input, outputs, continue);
            println!("got {} bytes", input.len());
            zip.update(input.into());
            let entries = zip.drain_entries();
            for entry in entries {
                let expanded = entry.inflate().unwrap();

                let filename = expanded.name().to_owned();
                let (_, data) = expanded.into_parts();
                println!("got entry {}", filename);
                outputs.filename.send(&filename);
                outputs.contents.send(&Bytes::new(data));
            }
        }

        outputs.filename.done();
        outputs.contents.done();
        Ok(())
    }
}
