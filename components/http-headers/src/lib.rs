mod wick {
    wick_component::wick_import!();
}
use std::collections::HashMap;

use wick::*;

fn process_headers(
    headers: &mut HashMap<String, Vec<String>>,
    header: &str,
    value: &types::Strings,
) {
    if let Some(values) = headers.get_mut(header) {
        match value {
            types::Strings::String(value) => {
                values.push(value.clone());
            }
            types::Strings::StringList(value_list) => {
                values.extend(value_list.clone());
            }
        }
    } else {
        match value {
            types::Strings::String(value) => {
                headers.insert(header.to_string(), vec![value.clone()]);
            }
            types::Strings::StringList(value_list) => {
                headers.insert(header.to_string(), value_list.clone());
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl add::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = add::Outputs;
    type Config = add::Config;

    async fn add(
        mut input: WickStream<types::http::RequestMiddlewareResponse>,
        mut value: WickStream<types::Strings>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let header = ctx.config.header.clone();
        println!("header: {}", header);
        while let (Some(input), Some(value)) = (input.next().await, value.next().await) {
            let input = propagate_if_error!(input, outputs, continue);
            let value = propagate_if_error!(value, outputs, continue);

            match input {
                types::http::RequestMiddlewareResponse::HttpRequest(mut req) => {
                    process_headers(&mut req.headers, &header, &value);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpRequest(req));
                }
                types::http::RequestMiddlewareResponse::HttpResponse(mut res) => {
                    process_headers(&mut res.headers, &header, &value);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpResponse(res));
                }
            }
        }
        outputs.output.done();
        Ok(())
    }
}