use regex::Regex;

mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl MatchOperation for Component {
    type Error = anyhow::Error;
    type Outputs = match_::Outputs;
    type Config = match_::Config;
    async fn match_(
        mut input: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let pattern = ctx.config.pattern.clone();
        while let Some(Ok(input)) = input.next().await {
            println!("Pattern: {}", pattern);
            let re = match Regex::new(&pattern) {
                Ok(re) => re,
                Err(e) => {
                    return Err(anyhow::anyhow!("Invalid Regex Pattern: {}", e));
                }
            };
            outputs.result.send(&re.is_match(&input));

            outputs.result.done();
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl CaptureOperation for Component {
    type Error = anyhow::Error;
    type Outputs = capture::Outputs;
    type Config = capture::Config;
    async fn capture(
        mut input: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let pattern = ctx.config.pattern.clone();
        while let Some(Ok(input)) = input.next().await {
            println!("Pattern: {}", pattern);
            let re = match Regex::new(&pattern) {
                Ok(re) => re,
                Err(e) => {
                    return Err(anyhow::anyhow!("Invalid Regex Pattern: {}", e));
                }
            };

            if let Some(captures) = re.captures(&input) {
                let mut captures_vec = Vec::new();
                for capture in captures.iter() {
                    if let Some(capture) = capture {
                        captures_vec.push(capture.as_str().to_string());
                    }
                }
                outputs.result.send(&captures_vec);
            }
        }
        outputs.result.done();
        Ok(())
    }
}

// #[async_trait::async_trait(?Send)]
// impl CapturesOperation for Component {
//     type Error = anyhow::Error;
//     type Outputs = captures::Outputs;
//     type Config = captures::Config;
//     async fn captures(
//         mut input: WickStream<String>,
//         mut outputs: Self::Outputs,
//         ctx: Context<Self::Config>,
//     ) -> anyhow::Result<()> {
//         let pattern = ctx.config.pattern.clone();
//         while let Some(Ok(input)) = input.next().await {
//             println!("Pattern: {}", pattern);
//             let re = match Regex::new(&pattern) {
//                 Ok(re) => re,
//                 Err(e) => {
//                     return Err(anyhow::anyhow!("Invalid Regex Pattern: {}", e));
//                 }
//             };

//             let mut all_captures_vec: Vec<Vec<String>> = Vec::new(); // Store captures for all matches

//             for captures in re.captures_iter(&input) {
//                 let mut captures_vec = Vec::new();
//                 for capture in captures.iter() {
//                     if let Some(capture) = capture {
//                         captures_vec.push(capture.as_str().to_string());
//                     }
//                 }

//                 all_captures_vec.push(captures_vec);
//             }

//             outputs.captures.send(&all_captures_vec);

//             outputs.captures.done();
//         }
//         Ok(())
//     }
// }
