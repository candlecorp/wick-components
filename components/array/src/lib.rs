mod wick {
    wick_component::wick_import!();
}
use glob::Pattern;
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl includes::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = includes::Outputs;
    type Config = includes::Config;

    async fn includes(
        mut value: WickStream<Value>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let array = ctx.config.array.clone();
        while let Some(value) = value.next().await {
            let value = propagate_if_error!(value, outputs, continue);
            let matches = array.contains(&value);
            outputs.result.send(&matches);
        }
        outputs.result.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl includes_glob::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = includes_glob::Outputs;
    type Config = includes_glob::Config;

    async fn includes_glob(
        mut value: WickStream<Value>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let array = ctx.config.array.clone();
        while let Some(value) = value.next().await {
            let value = propagate_if_error!(value, outputs, continue);
            let target_pattern = Pattern::new(&value.to_string());

            if target_pattern.is_err() {
                outputs
                    .result
                    .error(&target_pattern.unwrap_err().to_string());
                continue;
            }

            let target_pattern = target_pattern.unwrap();

            let matches = array.iter().any(|item| {
                let pattern = Pattern::new(item.to_string().as_str());

                if pattern.is_err() {
                    outputs.result.error(&pattern.unwrap_err().to_string());
                    return false;
                }

                let pattern = pattern.unwrap();

                pattern.matches_with(
                    value.to_string().as_str(),
                    glob::MatchOptions {
                        case_sensitive: false,
                        require_literal_separator: true,
                        require_literal_leading_dot: false,
                    },
                ) || target_pattern.matches(pattern.as_str())
            });
            outputs.result.send(&matches);
        }
        outputs.result.done();
        Ok(())
    }
}
