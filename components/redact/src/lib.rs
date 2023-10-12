mod wick {
    wick_component::wick_import!();
}

use ::regex::{Captures, Regex};

use anyhow::Result;
use wick::*;

#[wick_component::operation(generic_raw)]
async fn regex(
    mut inputs: regex::Inputs,
    mut outputs: regex::Outputs,
    ctx: Context<regex::Config>,
) -> Result<()> {
    let mut patterns = Vec::new();
    for pattern in &ctx.config.patterns {
        patterns.push(Regex::new(&pattern)?);
    }

    while let Some(line) = inputs.input.next().await {
        if line.is_open_bracket() {
            outputs.broadcast_open();
            continue;
        }
        if line.is_close_bracket() {
            outputs.broadcast_close();
            continue;
        }
        let mut line = propagate_if_error!(line.decode(), outputs, continue);

        for pattern in &patterns {
            line = pattern
                .replace_all(&line, |caps: &Captures| -> String {
                    // replace the string with XXXX where X is the length of the word
                    "X".repeat(caps[0].len())
                })
                .into();
        }
        outputs.output.send(&line);
    }
    outputs.output.done();

    Ok(())
}
