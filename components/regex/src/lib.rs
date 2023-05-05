use std::cmp::min;

use regex::Regex;
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpMatch for Component {
    async fn match_(
        mut input: WickStream<String>,
        mut outputs: OpMatchOutputs,
        ctx: Context<OpMatchConfig>,
    ) -> wick::Result<()> {
        let pattern = ctx.config.pattern.clone();
        while let Some(Ok(input)) = input.next().await {
            println!("Pattern: {}", pattern);
            let re = match Regex::new(&pattern) {
                Ok(re) => re,
                Err(e) => {
                    return Err(wick_component::anyhow::anyhow!(
                        "Invalid Regex Pattern: {}",
                        e
                    ));
                }
            };

            outputs.result.send(&re.is_match(&input));
            outputs.result.done();
        }
        Ok(())
    }
}

// here is the component definition
// - name: match
// inputs:
//   - name: input
//     type: string
//   - name: pattern
//     type: string
// outputs:
//   - name: result
//     type: bool
//   - name: output
//     type: string
//   - name: captures
//     type: { list: { type: string } }
//     optional: true
