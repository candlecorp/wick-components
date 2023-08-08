use std::{borrow::Cow, time::SystemTime};

mod wick {
    #![allow(
        unused_imports,
        missing_debug_implementations,
        clippy::needless_pass_by_value
    )]
    wick_component::wick_import!();
}
use wick::*;

use self::wick::types::http::HttpMethod;

#[derive(rust_embed::RustEmbed)]
#[folder = "package/"]
struct Assets;

#[derive(serde::Deserialize, serde::Serialize)]
struct Request {
    message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Response {
    output_message: String,
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl serve::Operation for Component {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Outputs = serve::Outputs;
    type Config = serve::Config;

    async fn serve(
        mut request: WickStream<types::http::HttpRequest>,
        _body: WickStream<Bytes>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(req) = request.next().await {
            let req = propagate_if_error!(req, outputs, continue);
            let path = if req.path == "/" {
                "/index.html".to_owned()
            } else {
                req.path.clone()
            };
            let path = path.trim_start_matches('/');
            let asset = Assets::get(path);
            if let Some(asset) = asset {
                if req.method != HttpMethod::Get {
                    let res = types::http::HttpResponse {
                        version: types::http::HttpVersion::Http11,
                        status: types::http::StatusCode::MethodNotAllowed,
                        headers: std::collections::HashMap::new(),
                    };
                    outputs.response.send(&res);
                    outputs.body.send(&Bytes::default());
                    continue;
                }
                let mut res = types::http::HttpResponse {
                    version: types::http::HttpVersion::Http11,
                    status: types::http::StatusCode::Ok,
                    headers: std::collections::HashMap::new(),
                };
                if let Some(secs) = asset.metadata.last_modified() {
                    let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs);
                    let time = wick_component::datetime::DateTime::from(time);

                    res.headers
                        .insert("last-modified".to_owned(), vec![time.to_rfc2822()]);
                }

                let (_, ext) = path.split_once('.').unwrap_or((path, ""));
                let content_type = match ext {
                    "" => "text/plain",
                    "html" => "text/html",
                    "png" => "image/png",
                    "css" => "text/css",
                    "md" => "text/plain",
                    "js" => "text/javascript",
                    "map" => "application/json",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                };

                let data = if path == "swagger-initializer.js" {
                    let data = std::str::from_utf8(&asset.data).unwrap();
                    let data = data.replace(
                        "https://petstore.swagger.io/v2/swagger.json",
                        ctx.root_config().schema_url.as_str(),
                    );
                    Cow::Owned(data.as_bytes().to_vec())
                } else {
                    asset.data
                };

                res.headers
                    .insert("content-type".to_owned(), vec![content_type.to_owned()]);
                res.headers
                    .insert("content-length".to_owned(), vec![data.len().to_string()]);
                outputs.response.send(&res);
                if let Cow::Borrowed(data) = data {
                    outputs
                        .body
                        .send(&Bytes::new(wick_component::bytes::Bytes::from_static(data)));
                } else {
                    outputs
                        .body
                        .send(&Bytes::new(wick_component::bytes::Bytes::copy_from_slice(
                            &data,
                        )));
                }
            } else {
                let res = types::http::HttpResponse {
                    version: types::http::HttpVersion::Http11,
                    status: types::http::StatusCode::NotFound,
                    headers: std::collections::HashMap::new(),
                };
                outputs.response.send(&res);
                outputs.body.send(&Bytes::default());
                continue;
            }
        }

        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}
