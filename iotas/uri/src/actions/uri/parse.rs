use crate::actions::uri_service::parse::*;

pub(crate) async fn task(input: Input) -> Result<Output, crate::Error> {
    println!("input: {:?}", input.url);
    let url = url::Url::parse(&input.url);

    println!("url: {:?}", url);

    Ok(Uri {
        scheme: "".to_string(),
        host: "".to_string(),
        port: None,
        path: "".to_string(),
        path_components: vec![],
        path_extension: "".to_string(),
        query: "".to_string(),
        query_params: std::collections::HashMap::new(),
        fragment: "".to_string(),
        username: None,
        password: None,
    })
}
