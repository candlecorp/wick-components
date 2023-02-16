
use crate::actions::url_service::parse::*;

pub(crate) async fn task(input: Inputs) -> Result<Outputs, crate::Error> {
    //parse url into parts
    let url = url::Url::parse(&input.url)?;
    println("url: {:?}", url);
}

//create test
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task() {
        let input = Inputs {
            url: "https://www.google.com".to_string(),
        };
        let output = task(input).await.unwrap();
        assert_eq!(output.url, "https://www.google.com");
    }
}