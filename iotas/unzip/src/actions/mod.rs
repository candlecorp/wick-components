/************************************************
 * THIS FILE IS GENERATED, DO NOT EDIT          *
 *                                              *
 * See https://apexlang.io for more information *
 ***********************************************/
pub(crate) mod unzip {
    pub(crate) mod unzip;
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

pub(crate) struct UnzipComponent();

impl UnzipComponent {
    fn unzip_wrapper(input: IncomingMono) -> Result<OutgoingMono, GenericError> {
        let (tx, rx) = runtime::oneshot();

        let input = deserialize_helper(input);

        let task = async move {
            let input_payload = match input.await {
                Ok(i) => i,
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };

            #[allow(unused)]
            fn des(
                mut map: std::collections::BTreeMap<String, Value>,
            ) -> Result<unzip_service::unzip::Inputs, Error> {
                Ok(unzip_service::unzip::Inputs {
                    source: <String as serde::Deserialize>::deserialize(
                        map.remove("source").ok_or_else(|| {
                            wasmrs_guest::Error::MissingInput("source".to_owned())
                        })?,
                    )
                    .map_err(|e| wasmrs_guest::Error::Decode(e.to_string()))?,
                })
            }

            let input = match des(input_payload) {
                Ok(i) => i,
                Err(e) => {
                    let _ = tx.send(Err(PayloadError::application_error(e.to_string())));
                    return;
                }
            };

            let _ = UnzipComponent::unzip(input)
                .await
                .map(|result| {
                    Ok(serialize(&result)
                        .map(|bytes| Payload::new_data(None, Some(bytes.into())))?)
                })
                .map(|output| {
                    let _ = tx.send(output);
                });
        };

        spawn(task);

        Ok(Mono::from_future(async move { rx.await? }))
    }
}

#[async_trait::async_trait(?Send)]

pub(crate) trait UnzipService {
    async fn unzip(
        inputs: unzip_service::unzip::Inputs,
    ) -> Result<unzip_service::unzip::Outputs, GenericError>;
}

#[async_trait::async_trait(?Send)]
impl UnzipService for UnzipComponent {
    async fn unzip(
        inputs: unzip_service::unzip::Inputs,
    ) -> Result<unzip_service::unzip::Outputs, GenericError> {
        Ok(crate::actions::unzip::unzip::task(inputs).await?)
    }
}

pub mod unzip_service {
    #[allow(unused_imports)]
    pub(crate) use super::*;

    pub mod unzip {
        #[allow(unused_imports)]
        pub(crate) use super::*;

        pub(crate) struct Inputs {
            pub(crate) source: String,
        }

        pub(crate) type Outputs = ();
    }
}

static READER_READ_INDEX_BYTES: [u8; 4] = 0u32.to_be_bytes();

pub mod reader {
    use super::*;

    pub(crate) fn read(
        inputs: read::Inputs,
    ) -> impl Stream<Item = Result<read::Outputs, PayloadError>> {
        //) -> wasmrs_guest::Flux<read::Outputs, PayloadError> {
        let op_id_bytes = READER_READ_INDEX_BYTES.as_slice();
        let payload = match wasmrs_guest::serialize(&inputs) {
            Ok(bytes) => Payload::new([op_id_bytes, &[0, 0, 0, 0]].concat().into(), bytes.into()),
            Err(_) => unreachable!(),
        };
        Host::default().request_stream(payload).map(|result| {
            result.map(|payload| Ok(deserialize::<read::Outputs>(&payload.data.unwrap())?))?
        })
    }

    pub(crate) mod read {
        use super::*;

        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct Inputs {
            pub(crate) source: String,
        }

        pub(crate) type Outputs = wasmrs_guest::Bytes;
    }
}

static WRITER_WRITE_INDEX_BYTES: [u8; 4] = 1u32.to_be_bytes();

pub mod writer {
    use super::*;

    pub(crate) fn write(
        mut inputs: write::Inputs,
    ) -> impl Stream<Item = Result<write::Outputs, PayloadError>> {
        //) -> wasmrs_guest::Flux<write::Outputs, PayloadError> {
        let op_id_bytes = WRITER_WRITE_INDEX_BYTES.as_slice();

        let (tx, rx) = Flux::new_channels();

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(untagged)]
        enum OpInputs {
            Params(write::InputFirst),
            Contents(wasmrs_guest::Bytes),
        }

        let first = OpInputs::Params(write::InputFirst { dest: inputs.dest });

        let tx_inner = tx.clone();
        spawn(async move {
            while let Some(payload) = inputs.contents.next().await {
                if let Err(e) = payload {
                    tx_inner.error(e);
                    continue;
                }
                let payload = payload.unwrap();
                let message = OpInputs::Contents(payload);
                let payload = wasmrs_guest::serialize(&message)
                    .map(|b| Payload::new_data(None, Some(b.into())))
                    .map_err(|e| PayloadError::application_error(e.to_string()));
                let _ = tx_inner.send_result(payload);
            }
        });

        let payload = wasmrs_guest::serialize(&first)
            .map(|b| Payload::new([op_id_bytes, &[0, 0, 0, 0]].concat().into(), b.into()))
            .map_err(|e| PayloadError::application_error(e.to_string()));
        tx.send_result(payload);

        Host::default().request_channel(rx).map(|result| {
            result.map(|payload| Ok(deserialize::<write::Outputs>(&payload.data.unwrap())?))?
        })
    }

    pub(crate) mod write {
        use super::*;

        pub struct Inputs {
            pub(crate) dest: String,

            pub(crate) contents: FluxReceiver<wasmrs_guest::Bytes, PayloadError>,
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct InputFirst {
            pub(crate) dest: String,
        }

        pub(crate) type Outputs = ();
    }
}

pub(crate) fn init_imports() {
    wasmrs_guest::add_import(
        u32::from_be_bytes(READER_READ_INDEX_BYTES),
        OperationType::RequestStream,
        "unzip.Reader",
        "read",
    );

    wasmrs_guest::add_import(
        u32::from_be_bytes(WRITER_WRITE_INDEX_BYTES),
        OperationType::RequestChannel,
        "unzip.Writer",
        "write",
    );
}
pub(crate) fn init_exports() {
    wasmrs_guest::register_request_response("unzip.Unzip", "unzip", UnzipComponent::unzip_wrapper);
}
