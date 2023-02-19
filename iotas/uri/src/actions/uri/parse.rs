use crate::actions::uri_service::parse::*;
use std::path::Path;

pub(crate) async fn task(input: Input) -> Result<Output, crate::Error> {
    println!("input: {:?}", input.url);

    let extension = Path::new(&input.url)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let extension_no_query = extension
        .split_once("?")
        .map(|(ext, _)| ext)
        .unwrap_or(extension);
    let extension_no_query_no_fragment = extension_no_query
        .split_once("#")
        .map(|(ext, _)| ext)
        .unwrap_or(extension_no_query);

    let result = url::Url::parse(&input.url);
    let uri = match result {
        Ok(url) => Uri {
            scheme: url.scheme().to_string(),
            host: match url.host() {
                Some(url::Host::Domain(domain)) => domain.to_string(),
                Some(url::Host::Ipv4(ip)) => ip.to_string(),
                Some(url::Host::Ipv6(ip)) => ip.to_string(),
                None => String::new(),
            },
            host_segments: url
                .host_str()
                .unwrap()
                .split(".")
                .map(|s| s.to_string())
                .collect(),
            port: url.port_or_known_default().map(|i: u16| i as u32),
            path: url.path().to_string(),
            path_segments: url
                .path_segments()
                .unwrap()
                .map(|s| s.to_string())
                .collect(),
            path_extension: extension_no_query_no_fragment.to_string(),
            query: url.query().map_or_else(String::new, |s| s.to_string()),
            query_params: url
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            fragment: url.fragment().map(|s| s.to_string()),
            username: Option::from(url.username().to_string()),
            password: url.password().map(|s| s.to_string()),
        },
        Err(_) => {
            return Err(PayloadError {
                code: 400,
                msg: "Invalid URL".to_string(),
            }
            .into());
        }
    };

    Ok(uri)
}
