mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl parse_completion::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = parse_completion::Outputs;
    type Config = parse_completion::Config;

    async fn parse_completion(
        mut event: WickStream<types::http::HttpEvent>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let mut count = 0;
        while let Some(event) = event.next().await {
            let event = propagate_if_error!(event, outputs, continue);
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
                event: Some("message".to_string()),
                data: data,
                id: None,
                retry: None,
            };

            outputs.event.send(&event_obj);
            count += 1;
        }

        let event_obj = types::http::HttpEvent {
            event: Some("tokens".to_string()),
            data: count.to_string(),
            id: None,
            retry: None,
        };
        outputs.event.send(&event_obj);

        let event_obj = types::http::HttpEvent {
            event: Some("message".to_string()),
            data: "[DONE]".to_string(),
            id: None,
            retry: None,
        };

        outputs.event.send(&event_obj);
        outputs.event.done();
        Ok(())
    }
}
