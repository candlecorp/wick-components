mod wick {
    wick_component::wick_import!();
}
use regex::Regex;
use serde_xml_rs::{from_str, to_string};
use wick::*;

use provided::usps_http;

#[async_trait::async_trait(?Send)]
impl verify::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = verify::Inputs;
    type Outputs = verify::Outputs;
    type Config = verify::Config;

    async fn verify(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let config: &RootConfig = ctx.root_config();
        while let Some(request) = inputs.address.next().await {
            let mut request = propagate_if_error!(request.decode(), outputs, continue);
            let user_id = config.user_id.clone();

            if request.address1.is_none() {
                request.address1 = Some("".to_string());
            }
            if request.zip4.is_none() {
                request.zip4 = Some(0);
            }

            let address_validate_request =
                types::usps_types::AddressValidateRequest { address: request };

            let xml = to_string(&address_validate_request).unwrap();

            let re = Regex::new(r"<AddressValidateRequest>").unwrap();
            let replace_string = format!(
                r#"<AddressValidateRequest USERID="{}"><Revision>1</Revision>"#,
                user_id
            );

            let xml_with_userid = re.replace_all(&xml, replace_string).to_string();

            println!("xml_with_userid: {:?}", xml_with_userid);

            // //call token http component using verify function
            let mut response_stream = ctx.provided().usps_http.verify(
                usps_http::verify::Config::default(),
                usps_http::verify::Request {
                    request: xml_with_userid.to_string(),
                },
            )?;
            let mut response_buffer: Vec<u8> = Vec::new();
            while let Some(packet) = response_stream.body.next().await {
                let body = propagate_if_error!(packet.decode(), outputs, continue);
                response_buffer.extend(&body);
            }

            println!("response: {:?}", response_buffer);

            let body_string = std::str::from_utf8(&response_buffer).expect("Found invalid UTF-8");

            println!("body_string: {:?}", body_string);

            let validate_response: types::usps_types::AddressValidateResponse =
                from_str(&body_string).unwrap();

            let response_address = validate_response.address.clone();
            println!("response_address: {:?}", response_address);

            outputs.verified_address.send(&response_address);
        }
        outputs.verified_address.done();
        Ok(())
    }
}
