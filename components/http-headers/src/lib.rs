mod wick {
    wick_component::wick_import!();
}
use std::collections::HashMap;

use wick::*;

fn process_headers(
    headers: &mut HashMap<String, Vec<String>>,
    header: &str,
    value: &types::Strings,
    replace: bool,
) {
    if let Some(values) = headers.get_mut(header) {
        if replace {
            values.clear();
        }
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
    type Inputs = add::Inputs;
    type Outputs = add::Outputs;
    type Config = add::Config;

    async fn add(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let header = ctx.config.header.clone();
        println!("header: {}", header);
        while let (Some(input), Some(value)) =
            (inputs.input.next().await, inputs.value.next().await)
        {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            let value = propagate_if_error!(value.decode(), outputs, continue);

            match input {
                types::http::RequestMiddlewareResponse::HttpRequest(mut req) => {
                    process_headers(&mut req.headers, &header, &value, false);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpRequest(req));
                }
                types::http::RequestMiddlewareResponse::HttpResponse(mut res) => {
                    process_headers(&mut res.headers, &header, &value, false);
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

#[async_trait::async_trait(?Send)]
impl update::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = update::Inputs;
    type Outputs = update::Outputs;
    type Config = update::Config;

    async fn update(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let header = ctx.config.header.clone();
        println!("header: {}", header);
        while let (Some(input), Some(value)) =
            (inputs.input.next().await, inputs.value.next().await)
        {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            let value = propagate_if_error!(value.decode(), outputs, continue);

            match input {
                types::http::RequestMiddlewareResponse::HttpRequest(mut req) => {
                    process_headers(&mut req.headers, &header, &value, true);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpRequest(req));
                }
                types::http::RequestMiddlewareResponse::HttpResponse(mut res) => {
                    process_headers(&mut res.headers, &header, &value, true);
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
