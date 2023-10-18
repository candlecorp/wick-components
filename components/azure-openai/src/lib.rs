mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl parse_completion::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = parse_completion::Inputs;
    type Outputs = parse_completion::Outputs;
    type Config = parse_completion::Config;

    async fn parse_completion(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let mut count = 0;
        println!("parse_completion started");
        while let Some(event) = inputs.event.next().await {
            let event = event.decode()?;
            println!("event: {:?}", event);

            if &event.data == "[DONE]" {
                break;
            }

            let event_data: types::events::EventData =
                wick_component::from_str(&event.data).unwrap();

            let choices = event_data.choices;

            if choices.len() == 0 {
                continue;
            }

            let choices: types::events::Choice = choices[0].clone();
            let delta = choices.delta;
            let content = delta.content;
            if content.is_none() {
                continue;
            }
            let data = content.unwrap();

            let event_obj = types::http::HttpEvent {
                event: "message".to_string(),
                data: data,
                id: "".to_string(),
                retry: None,
            };

            outputs.event.send(&event_obj);
            count += 1;
        }

        let event_obj = types::http::HttpEvent {
            event: "tokens".to_string(),
            data: count.to_string(),
            id: "".to_string(),
            retry: None,
        };
        outputs.event.send(&event_obj);

        let event_obj = types::http::HttpEvent {
            event: "message".to_string(),
            data: "[DONE]".to_string(),
            id: "".to_string(),
            retry: None,
        };

        outputs.event.send(&event_obj);
        outputs.tokens.send(count);
        outputs.tokens.done();
        outputs.event.done();
        Ok(())
    }
}
