mod wick {
    wick_component::wick_import!();
}
use wick::*;

use regex::Regex;

#[async_trait::async_trait(?Send)]
impl get_stargazers::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = get_stargazers::Inputs;
    type Outputs = get_stargazers::Outputs;
    type Config = get_stargazers::Config;

    async fn get_stargazers(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let mut pages = 1;

        while let (Some(organization), Some(repository)) = (
            inputs.organization.next().await,
            inputs.repository.next().await,
        ) {
            let organization = propagate_if_error!(organization.decode(), outputs, continue);
            let repository = propagate_if_error!(repository.decode(), outputs, continue);

            println!("organization: {:?}", organization);
            println!("repository: {:?}", repository);

            use provided::github_client::get_stargazers;

            let mut gh_res = ctx.provided().github_client.get_stargazers(
                get_stargazers::Config::default(),
                get_stargazers::Request {
                    organization: organization.clone(),
                    repository: repository.clone(),
                    page: 1,
                },
            )?;

            while let (Some(get_stargazers_response), Some(get_stargazers_body)) =
                (gh_res.response.next().await, gh_res.body.next().await)
            {
                let get_stargazers_response =
                    propagate_if_error!(get_stargazers_response.decode(), outputs, continue);
                let get_stargazers_body =
                    propagate_if_error!(get_stargazers_body.decode(), outputs, continue);

                let link_header = get_stargazers_response.headers.get("link");
                if link_header.is_some() {
                    let link_header = link_header.unwrap()[0].as_str();
                    println!("link_header: {:?}", link_header);

                    let re = Regex::new(r"page=(\d+)>; rel=.last").unwrap();

                    if let Some(captures) = re.captures(link_header) {
                        pages = captures[1].parse::<u32>().unwrap();
                    }
                }

                let mut stargazers = get_stargazers_body
                    .iter()
                    .map(|x| x.login.clone())
                    .collect::<Vec<String>>();

                println!("pages: {:?}", pages);
                if pages > 1 {
                    let mut page = 2;
                    while page <= pages {
                        println!("page: {:?}", page);
                        let mut gh_res = ctx.provided().github_client.get_stargazers(
                            get_stargazers::Config::default(),
                            get_stargazers::Request {
                                organization: organization.clone(),
                                repository: repository.clone(),
                                page,
                            },
                        )?;

                        while let (Some(get_stargazers_response), Some(get_stargazers_body)) =
                            (gh_res.response.next().await, gh_res.body.next().await)
                        {
                            let _get_stargazers_response = propagate_if_error!(
                                get_stargazers_response.decode(),
                                outputs,
                                continue
                            );
                            let get_stargazers_body = propagate_if_error!(
                                get_stargazers_body.decode(),
                                outputs,
                                continue
                            );
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
