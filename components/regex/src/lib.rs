use regex::Regex;

mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl match_::Operation for Component {
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

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl capture::Operation for Component {
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
