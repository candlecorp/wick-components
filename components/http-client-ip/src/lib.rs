mod wick {
    wick_component::wick_import!();
}
use wick::{types::http::HttpRequest, *};

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl get_ip::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = get_ip::Outputs;
    type Config = get_ip::Config;

    async fn get_ip(
        mut request: WickStream<HttpRequest>,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(request) = request.next().await {
            let request = propagate_if_error!(request, outputs, continue);
            let xff = request
                .headers
                .get("x-forwarded-for")
                .and_then(|x| Some(x[0].as_str()))
                .unwrap_or("");
            if xff.is_empty() {
                outputs.ip.send(&request.remote_addr);
                continue;
            } else {
                let ips = xff.split(',').collect::<Vec<_>>();
                let ip = ips[0].trim();
                outputs.ip.send(&ip.to_string());
                continue;
            }
        }
        outputs.ip.done();
        Ok(())
    }
}
