mod wick {
    wick_component::wick_import!();
}
use wick::*;

use regex::Regex;

#[async_trait::async_trait(?Send)]
impl get_stargazers::Operation for Component {
    type Error = anyhow::Error;
    type Outputs = get_stargazers::Outputs;
    type Config = get_stargazers::Config;

    async fn get_stargazers(
        mut organization: WickStream<String>,
        mut repository: WickStream<String>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let mut pages = 1;
        let mut stargazers: Vec<String> = Vec::new();

        while let (Some(organization), Some(repository)) =
            (organization.next().await, repository.next().await)
        {
            let organization = propagate_if_error!(organization, outputs, continue);
            let repository = propagate_if_error!(repository, outputs, continue);

            println!("organization: {:?}", organization);
            println!("repository: {:?}", repository);

            let (mut get_stargazers_response, mut get_stargazers_body) =
                ctx.provided().github_client.get_stargazers(
                    once(organization.clone()),
                    once(repository.clone()),
                    once(1),
                )?;

            while let (Some(get_stargazers_response), Some(get_stargazers_body)) = (
                get_stargazers_response.next().await,
                get_stargazers_body.next().await,
            ) {
                let get_stargazers_response =
                    propagate_if_error!(get_stargazers_response, outputs, continue);
                let get_stargazers_body =
                    propagate_if_error!(get_stargazers_body, outputs, continue);

                let link_header = get_stargazers_response.headers.get("link");
                if link_header.is_some() {
                    let link_header = link_header.unwrap()[0].as_str();
                    println!("link_header: {:?}", link_header);

                    let re = Regex::new(r"page=(\d+)>; rel=.last").unwrap();

                    if let Some(captures) = re.captures(link_header) {
                        pages = captures[1].parse::<u32>().unwrap();
                    }
                }

                stargazers = get_stargazers_body
                    .iter()
                    .map(|x| x.login.clone())
                    .collect::<Vec<String>>();

                println!("pages: {:?}", pages);
                if pages > 1 {
                    let mut page = 2;
                    while page <= pages {
                        println!("page: {:?}", page);
                        let (mut get_stargazers_response, mut get_stargazers_body) =
                            ctx.provided().github_client.get_stargazers(
                                once(organization.clone()),
                                once(repository.clone()),
                                once(page),
                            )?;
                        while let (Some(get_stargazers_response), Some(get_stargazers_body)) = (
                            get_stargazers_response.next().await,
                            get_stargazers_body.next().await,
                        ) {
                            let _get_stargazers_response =
                                propagate_if_error!(get_stargazers_response, outputs, continue);
                            let get_stargazers_body =
                                propagate_if_error!(get_stargazers_body, outputs, continue);
                            page += 1;
                            stargazers.extend(
                                get_stargazers_body
                                    .iter()
                                    .map(|x| x.login.clone())
                                    .collect::<Vec<String>>(),
                            );
                        }
                    }
                }
                outputs.stargazers.send(&stargazers.clone());
            }
        }
        outputs.stargazers.done();
        Ok(())
    }
}
