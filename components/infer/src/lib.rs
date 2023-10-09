mod llama;
mod wick {
    wick_component::wick_import!();
}

use anyhow::Result;
use wick::*;
use wick_component::runtime::spawn;

use self::llama::Args;

#[wick_component::operation(unary_with_outputs)]
fn generate(
    prompt: String,
    mut outputs: generate::Outputs,
    ctx: Context<generate::Config>,
) -> Result<()> {
    let model = llama::load_model(&ctx.root_config().model)?;

    let tokenizer = llama::load_tokenizer(&ctx.root_config().tokenizer)?;
    let args = Args {
        prompt,
        temperature: ctx.config.temperature,
        top_p: ctx.config.top_p,
        repeat_penalty: ctx.config.repeat_penalty.map_or(1.1, |v| v as f32),
        repeat_last_n: 64,
        max_seq: ctx.config.max_length.unwrap_or(256) as _,
    };
    let (tx, rx) = FluxChannel::new_parts();
    spawn("llama:generate", async move {
        if let Err(e) = llama::generate(model, tokenizer, args, tx).await {
            println!("error running inference: {}", e);
        };
    });
    spawn("output", async move {
        let rx = rx;
        while let Ok(Some(Ok(packet))) = rx.recv().await {
            outputs.output.send(packet);
        }
    });

    Ok(())
}
