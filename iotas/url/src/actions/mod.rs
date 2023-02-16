pub(crate) mod url {
    pub(crate) use super::*;
    pub(crate) mod parse;
}
/************************************************
 * THIS FILE IS GENERATED, DO NOT EDIT          *
 *                                              *
 * See https://apexlang.io for more information *
 ***********************************************/
use wasmrs_guest::FutureExt;

use wasmrs_guest::*;

#[no_mangle]
extern "C" fn __wasmrs_init(
    guest_buffer_size: u32,
    host_buffer_size: u32,
    max_host_frame_len: u32,
) {
    init_exports();
    init_imports();
    wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
}

fn deserialize_helper<T: serde::de::DeserializeOwned + 'static>(
    i: Mono<ParsedPayload, PayloadError>,
) -> Mono<T, PayloadError> {
    Mono::from_future(async move {
        match i.await {
            Ok(bytes) => match deserialize(&bytes.data) {
                Ok(v) => Ok(v),
                Err(e) => Err(PayloadError::application_error(e.to_string())),
            },
            Err(e) => Err(PayloadError::application_error(e.to_string())),
        }
    })
}

pub(crate) struct UrlComponent();

impl UrlComponent {
    fn parse_wrapper(input: IncomingMono) -> Result<OutgoingMono, GenericError> {
        let (tx, rx) = runtime::oneshot();

        let input = Mono::from_future(input.map(|r| r.map(|v| Ok(deserialize(&v.data)?))?));
        let task = UrlComponent::parse(input)
            .map(|result| {
                let output = result?;
                Ok(serialize(&output).map(|bytes| Payload::new_data(None, Some(bytes.into())))?)
            })
            .map(|output| tx.send(output).unwrap());

        spawn(task);

        Ok(Mono::from_future(async move { rx.await? }))
    }
}

#[async_trait::async_trait(?Send)]

pub(crate) trait UrlService {
    async fn parse(
        inputs: Mono<url_service::parse::Inputs, PayloadError>,
    ) -> Result<url_service::parse::Outputs, GenericError>;
}

#[async_trait::async_trait(?Send)]
impl UrlService for UrlComponent {
    async fn parse(
        inputs: Mono<url_service::parse::Inputs, PayloadError>,
    ) -> Result<url_service::parse::Outputs, GenericError> {
        Ok(crate::actions::url::parse::task(inputs.await?).await?)
    }
}

pub mod url_service {
    #[allow(unused_imports)]
    pub(crate) use super::*;

    pub mod parse {
        #[allow(unused_imports)]
        pub(crate) use super::*;
        #[derive(serde::Deserialize)]
        pub(crate) struct Inputs {
            #[serde(rename = "url")]
            pub(crate) url: String,
        }

        pub(crate) type Outputs = Url;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Url {
    #[serde(rename = "protocol")]
    pub protocol: String,
    #[serde(rename = "host")]
    pub host: String,
    #[serde(rename = "port")]
    pub port: u32,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "path_fragements")]
    pub path_fragements: Vec<String>,
    #[serde(rename = "query")]
    pub query: String,
    #[serde(rename = "query_params")]
    pub query_params: std::collections::HashMap<String, String>,
}
pub(crate) fn init_imports() {}
pub(crate) fn init_exports() {
    wasmrs_guest::register_request_response("url.v1.Url", "parse", UrlComponent::parse_wrapper);
}
