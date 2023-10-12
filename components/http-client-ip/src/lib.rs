mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl get_ip::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = get_ip::Inputs;
    type Outputs = get_ip::Outputs;
    type Config = get_ip::Config;

    async fn get_ip(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        while let Some(request) = inputs.request.next().await {
            let request = propagate_if_error!(request.decode(), outputs, continue);
            println!("request: {:?}", request);
            let xff = request
                .headers
                .get("x-forwarded-for")
                .and_then(|x| Some(x[0].as_str()))
                .unwrap_or("");
            println!("xff: {:?}", xff);
            if xff.is_empty() {
                outputs.ip.send(&request.remote_addr);
                println!("remote_addr: {:?}", request.remote_addr);
                continue;
            } else {
                let ips = xff.split(',').collect::<Vec<_>>();
                let ip = ips[0].trim();
                outputs.ip.send(&ip.to_string());
                println!("ip: {:?}", ip);
                continue;
            }
        }
        outputs.ip.done();
        Ok(())
    }
}
