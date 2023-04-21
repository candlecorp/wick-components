use std::cmp::min;

use regex::Regex;
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpRematch for Component {
    async fn rematch(
        mut input: WickStream<String>,
        mut pattern: WickStream<String>,
        mut outputs: OpRematchOutputs,
    ) -> wick::Result<()> {
        while let (Some(Ok(input)), Some(Ok(pattern_string))) =
            (input.next().await, pattern.next().await)
        {
            println!("Pattern: {}", pattern_string);
            let re = match Regex::new(&pattern_string) {
                Ok(re) => re,
                Err(e) => {
                    return Err(wick_component::anyhow::anyhow!(
                        "Invalid Regex Pattern: {}",
                        e
                    ))
                }
            };
            let mut matches: Vec<Vec<String>> = Vec::new();
            let mut match_found = false;

            outputs.matches.open_bracket();
            //use regex to match the pattern and input and return a vec of matches and captures
            for cap in re.captures_iter(&input) {
                let mut captures: Vec<String> = Vec::new();
                for i in 0..min(cap.len(), 10) {
                    captures.push(cap[i].to_string());
                }
                if !match_found {
                    outputs.result.send(&true);
                    outputs.result.done();
                    match_found = true;
                }
                outputs.matches.send(&captures);
                matches.push(captures);
            }
            outputs.matches.close_bracket();
            if matches.len() == 0 {
                outputs.result.send(&false);
                outputs.result.done();
            }
        }
        outputs.matches.done();
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
