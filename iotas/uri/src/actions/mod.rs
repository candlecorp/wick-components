/************************************************
 * THIS FILE IS GENERATED, DO NOT EDIT          *
 *                                              *
 * See https://apexlang.io for more information *
 ***********************************************/
pub(crate) mod uri {
    pub(crate) mod parse;
}

use wasmrs_guest::*;

#[no_mangle]
extern "C" fn __wasmrs_init(
    guest_buffer_size: u32,
    host_buffer_size: u32,
    max_host_frame_len: u32,
) {
    wasmrs_guest::init_logging();

    init_exports();
    init_imports();
    wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
}

fn deserialize_helper(
    i: Mono<ParsedPayload, PayloadError>,
) -> Mono<std::collections::BTreeMap<String, wasmrs_guest::Value>, PayloadError> {
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

pub(crate) struct UriComponent();

impl UriComponent {
    fn parse_wrapper(input: IncomingMono) -> Result<OutgoingMono, GenericError> {
        let (tx, rx) = runtime::oneshot();
        let input = deserialize_helper(input);
        spawn(async move {
            let input_payload = match input.await {
                Ok(o) => o,
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };
            fn des(
                mut map: std::collections::BTreeMap<String, Value>,
            ) -> Result<uri_service::parse::Input, Error> {
                Ok(uri_service::parse::Input {
                    url: <String as serde::Deserialize>::deserialize(
                        map.remove("url")
                            .ok_or_else(|| wasmrs_guest::Error::MissingInput("url".to_owned()))?,
                    )
                    .map_err(|e| wasmrs_guest::Error::Decode(e.to_string()))?,
                })
            }
            let _ = UriComponent::parse(match des(input_payload) {
                Ok(o) => o,
                Err(e) => {
                    let _ = tx.send(Err(PayloadError::application_error(e.to_string())));
                    return;
                }
            })
            .await
            .map(|result| {
                serialize(&result)
                    .map(|b| Payload::new_data(None, Some(b.into())))
                    .map_err(|e| PayloadError::application_error(e.to_string()))
            })
            .map(|output| {
                let _ = tx.send(output);
            });
        });
        Ok(Mono::from_future(async move { rx.await? }))
    }
}

#[async_trait::async_trait(?Send)]

pub(crate) trait UriService {
    async fn parse(
        input: uri_service::parse::Input,
    ) -> Result<uri_service::parse::Output, GenericError>;
}

#[async_trait::async_trait(?Send)]
impl UriService for UriComponent {
    async fn parse(
        input: uri_service::parse::Input,
    ) -> Result<uri_service::parse::Output, GenericError> {
        Ok(crate::actions::uri::parse::task(input).await?)
    }
}

pub mod uri_service {
    #[allow(unused_imports)]
    pub(crate) use super::*;

    pub mod parse {
        #[allow(unused_imports)]
        pub(crate) use super::*;

        #[allow(unused)]
        pub(crate) struct Input {
            pub(crate) url: String,
        }

        pub(crate) type Output = Uri;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Uri {
    #[serde(rename = "scheme")]
    pub scheme: String,
    #[serde(rename = "host")]
    pub host: String,
    #[serde(rename = "host_segments")]
    pub host_segments: Vec<String>,
    #[serde(rename = "port")]
    pub port: Option<u32>,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "path_segments")]
    pub path_segments: Vec<String>,
    #[serde(rename = "path_extension")]
    pub path_extension: String,
    #[serde(rename = "query")]
    pub query: String,
    #[serde(rename = "query_params")]
    pub query_params: std::collections::HashMap<String, String>,
    #[serde(rename = "fragment")]
    pub fragment: Option<String>,
    #[serde(rename = "username")]
    pub username: Option<String>,
    #[serde(rename = "password")]
    pub password: Option<String>,
}
pub(crate) fn init_imports() {}
pub(crate) fn init_exports() {
    wasmrs_guest::register_request_response("uri.v1.Uri", "parse", UriComponent::parse_wrapper);
}
