mod wick {
    wick_component::wick_import!();
}
use std::collections::HashMap;

use base64::{engine::general_purpose, Engine};
use wick::*;

fn build_error_response(msg: &str) -> types::http::HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();
    headers.insert("x-auth-error".to_string(), vec![msg.to_string()]);

    types::http::HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: types::http::StatusCode::BadRequest,
        headers: headers,
    }
}

#[async_trait::async_trait(?Send)]
impl basic::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = basic::Inputs;
    type Outputs = basic::Outputs;
    type Config = basic::Config;

    async fn basic(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(input) = inputs.request.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input.decode(), outputs, continue);

            let auth_header = input.headers.get("authorization");
            if auth_header.is_some() {
                let auth_header = auth_header.unwrap();
                let auth_value = auth_header.get(0);
                if auth_value.is_some() {
                    let auth_value = auth_value.unwrap();
                    let auth_val = auth_value.split(" ").collect::<Vec<_>>();
                    if auth_val.len() == 2 {
                        let auth_type = auth_val[0];
                        let auth_value = auth_val[1];
                        if auth_type == "Basic" {
                            println!("auth_value: {}", auth_value);
                            let auth_value = general_purpose::STANDARD.decode(auth_value)?;
                            let auth_credentials = String::from_utf8(auth_value)?;

                            println!("auth_credentials: {}", auth_credentials);
                            let allowed_user = ctx.config.username.clone();
                            let allowed_pass = ctx.config.password.clone();

                            if auth_credentials == format!("{}:{}", allowed_user, allowed_pass) {
                                outputs.output.send(
                                    &types::http::RequestMiddlewareResponse::HttpRequest(input),
                                );
                                continue;
                            } else {
                                let response = build_error_response("invalid credentials");
                                outputs.output.send(
                                    &types::http::RequestMiddlewareResponse::HttpResponse(response),
                                );
                                continue;
                            }
                        } else {
                            let response = build_error_response("authorization type not supported");
                            outputs.output.send(
                                &types::http::RequestMiddlewareResponse::HttpResponse(response),
                            );
                            continue;
                        }
                    } else {
                        let response = build_error_response("invalid authorization header");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }
                }
            }

            let response = build_error_response("missing authorization header");
            outputs
                .output
                .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                    response,
                ));
        }
        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.output.done();
        println!("done");
        Ok(())
    }
}
