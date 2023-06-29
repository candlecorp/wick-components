mod wick {
    wick_component::wick_import!();
}
use cookie::time::OffsetDateTime as CookieDateTime;

use cookie::Cookie;
use wick::{
    types::http::{HttpRequest, HttpResponse},
    *,
};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use wick_component::{datetime::chrono::Duration, once};

use std::collections::HashMap;
use urlencoding::encode;
use wick_component::propagate_if_error;

struct OauthCookies {
    auth_state: Option<String>,
    return_url: Option<String>,
    session_id: Option<String>,
}

fn parse_cookies_header(
    headers: HashMap<String, Vec<String>>,
    session_cookie_name: &str,
) -> OauthCookies {
    let cookie_header = headers.get("cookie");
    let cookies: HashMap<String, String> = match cookie_header {
        Some(cookie_header) => Cookie::split_parse(&cookie_header[0])
            .filter_map(|c| c.ok())
            .map(|c| (c.name().to_string(), c.value().to_string()))
            .collect(),
        None => HashMap::new(),
    };
    OauthCookies {
        auth_state: cookies.get("auth_state").cloned(),
        return_url: cookies.get("return_url").cloned(),
        session_id: cookies.get(session_cookie_name).cloned(),
    }
}

fn build_error_response(msg: &str) -> HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();
    headers.insert("x-oauth-error".to_string(), vec![msg.to_string()]);

    HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: types::http::StatusCode::BadRequest,
        headers: headers,
    }
}

fn build_redirect_response(location: &str, cookies: Option<Vec<Cookie>>) -> HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();
    let mut cookies_vec = Vec::new();

    match cookies {
        Some(cookies) => {
            for cookie in cookies {
                cookies_vec.push(cookie.to_string());
            }
        }
        None => {}
    }

    if cookies_vec.len() > 0 {
        headers.insert("set-cookie".to_string(), cookies_vec);
    }

    headers.insert("location".to_string(), vec![location.to_string()]);

    HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: types::http::StatusCode::TemporaryRedirect,
        headers: headers,
    }
}

fn build_auth_redirect_response(
    config: RootConfig,
    state: &str,
    return_url: &str,
    timestamp: wick_packet::DateTime,
) -> HttpResponse {
    //expiraton time for new cookies
    let expires_at_new = timestamp + Duration::seconds(300);
    //expiration time to clear old session cookie
    let expires_at_old = timestamp + Duration::seconds(-1);

    let mut cookies = vec![];

    let state_cookie = Cookie::build("auth_state", state)
        .path("/")
        .http_only(true)
        .expires(CookieDateTime::from_unix_timestamp(expires_at_new.timestamp()).unwrap())
        .finish();

    let return_cookie = Cookie::build("return_url", return_url)
        .path("/")
        .http_only(true)
        .expires(CookieDateTime::from_unix_timestamp(expires_at_new.timestamp()).unwrap())
        .finish();

    let session_cookie = Cookie::build(config.session_cookie_name, "expired")
        .path("/")
        .http_only(true)
        .expires(CookieDateTime::from_unix_timestamp(expires_at_old.timestamp()).unwrap())
        .finish();
    cookies.push(state_cookie);
    cookies.push(return_cookie);
    cookies.push(session_cookie);

    let location = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        config.auth_endpoint,
        config.client_id,
        config.redirect_uri,
        encode(config.scope.as_str()),
        state
    );

    build_redirect_response(&location, Some(cookies))
}

fn get_oidc_claims(id_token: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = id_token.split('.').collect();

    let engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    let decoded = engine.decode(parts[1]).unwrap();

    let claims = String::from_utf8(decoded).unwrap();

    Ok(claims)
}

#[async_trait::async_trait(?Send)]
impl AuthOperation for Component {
    type Error = anyhow::Error;
    type Outputs = auth::Outputs;
    type Config = auth::Config;

    async fn auth(
        mut input: WickStream<HttpRequest>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        // initial setup for frequently used variables
        let config: &RootConfig = ctx.root_config();
        let rng = &ctx.inherent.rng;
        let timestamp = ctx.inherent.timestamp;

        while let Some(input) = input.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input, outputs, continue);

            //get cookies
            let cookies = parse_cookies_header(input.headers.clone(), &config.session_cookie_name);

            // change loging from if to match
            match input.path.as_str() {
                "/oidc/callback" => {
                    //verify state and code parameters exist
                    if ["state", "code"]
                        .iter()
                        .any(|&param| !input.query_parameters.contains_key(param))
                    {
                        let response =
                            build_error_response("State or code parameter does not exist");
                        outputs.response.send(&response);
                        continue;
                    }

                    //get state cookie value
                    if cookies.auth_state.is_none() {
                        //state cookie does not exist
                        let response = build_error_response("State cookie does not exist");
                        outputs.response.send(&response);
                        continue;
                    }

                    //check state cookie value
                    if cookies.auth_state.unwrap()
                        != input.query_parameters.get("state").unwrap()[0]
                    {
                        //state cookie value does not match
                        let response =
                            build_error_response("State cookie and response don't match");
                        outputs.response.send(&response);
                        continue;
                    }

                    //handle callback
                    let access_code = input.query_parameters.get("code").unwrap()[0].clone();

                    //call token http component using get_token function
                    let (mut get_token_response, mut get_token_response_body) =
                        ctx.provided().httpclient.get_token(
                            once(access_code.to_string()),
                            once(config.redirect_uri.clone()),
                        )?;

                    let mut response: Option<HttpResponse> = None;
                    let mut body: Option<types::OAuthTokenResponse> = None;

                    //intentionally this is not a multi-response stream so will only get the first response
                    while let (Some(token_response), Some(token_response_body)) = (
                        get_token_response.next().await,
                        get_token_response_body.next().await,
                    ) {
                        //ensure response is not an error
                        response = Some(propagate_if_error!(token_response, outputs, continue));
                        body = Some(propagate_if_error!(token_response_body, outputs, continue));
                    }

                    if response.is_none() || body.is_none() {
                        let response = build_error_response("Token endpoint returned error");
                        outputs.response.send(&response);
                        continue;
                    }

                    let response = response.unwrap();
                    let body = body.unwrap();

                    //ensure response is 200
                    if response.status != types::http::StatusCode::Ok {
                        let response = build_error_response("Token endpoint returned error");
                        outputs.response.send(&response);
                        continue;
                    }

                    //ensure body is not empty
                    if body.access_token.is_empty()
                        || body.expires_in == 0
                        || body.token_type.is_empty()
                        || body.id_token.is_empty()
                    {
                        let response =
                            build_error_response("Token endpoint returned invalid response");
                        outputs.response.send(&response);
                        continue;
                    }

                    //process id_token and extract claims
                    let claims = get_oidc_claims(body.id_token.as_str())?;
                    let session_id = &rng.uuid();

                    let expires_at = timestamp + Duration::seconds(body.expires_in as _);

                    let mut insert_token_response = ctx.provided().dbclient.insert_token(
                        once(session_id.to_string()),
                        once(body.token_type),
                        once(body.access_token),
                        once("".to_string()),
                        once(expires_at),
                    )?;

                    while let Some(insert_response) = insert_token_response.next().await {
                        let _response = propagate_if_error!(insert_response, outputs, continue);
                    }

                    let mut insert_claims_response = ctx
                        .provided()
                        .dbclient
                        .insert_claims(once(session_id.to_string()), once(claims))?;

                    while let Some(insert_response) = insert_claims_response.next().await {
                        let _response = propagate_if_error!(insert_response, outputs, continue);
                    }

                    //on error redirect to "/"
                    let location = cookies.return_url.unwrap_or("/".to_string());

                    let mut cookies: Vec<Cookie> = vec![];
                    let session_cookie =
                        Cookie::build(config.session_cookie_name.clone(), session_id.to_string())
                            .path("/")
                            .http_only(true)
                            .expires(
                                CookieDateTime::from_unix_timestamp(expires_at.timestamp())
                                    .unwrap(),
                            )
                            .finish();
                    cookies.push(session_cookie);

                    let response = build_redirect_response(&location, Some(cookies));
                    println!("response: {:?}", response);
                    outputs.response.send(&response);
                    continue;
                }
                "/oidc/logout" => {}
                _ => {
                    //handle all other requests

                    //session cookie does not exist redirect to login
                    if cookies.session_id.is_none() {
                        //create state cookie
                        let state = rng.uuid().to_string();
                        // redirect to auth endpoint
                        let response = build_auth_redirect_response(
                            config.clone(),
                            &state,
                            &input.uri,
                            timestamp,
                        );
                        outputs.response.send(&response);
                        println!("response: {:?}", response);
                        continue;
                    }

                    //session cookie exists lookup session to see if its valid
                    let session_id = cookies.session_id.unwrap();
                    let mut get_session_response = ctx
                        .provided()
                        .dbclient
                        .get_session(once(session_id.clone()))?;

                    while let Some(response) = get_session_response.next().await {
                        let response = propagate_if_error!(response, outputs, continue);

                        //session does not exist redirect to login
                        if response.access_token.is_empty() {
                            //create state cookie
                            let state = rng.uuid().to_string();
                            // redirect to auth endpoint
                            let response = build_auth_redirect_response(
                                config.clone(),
                                &state,
                                &input.uri,
                                timestamp,
                            );
                            outputs.response.send(&response);
                            continue;
                        }

                        //check if session is expired
                        if response.expires_at < timestamp {
                            //create state cookie
                            let state = rng.uuid().to_string();
                            // redirect to auth endpoint
                            let response = build_auth_redirect_response(
                                config.clone(),
                                &state,
                                &input.uri,
                                timestamp,
                            );
                            outputs.response.send(&response);
                            continue;
                        }

                        //session is valid
                        outputs.request.send(&input);
                    }
                    //session does not exist redirect to login

                    //create state cookie
                    let state = rng.uuid().to_string();
                    // redirect to auth endpoint
                    let response =
                        build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                    outputs.response.send(&response);
                    continue;
                }
            }
        }
        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.response.done();
        outputs.request.done();
        println!("done");
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl OidcOperation for Component {
    type Error = anyhow::Error;
    type Outputs = oidc::Outputs;
    type Config = oidc::Config;

    async fn oidc(
        mut input: WickStream<HttpRequest>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        // initial setup for frequently used variables
        let config: &RootConfig = ctx.root_config();
        let rng = &ctx.inherent.rng;
        let timestamp = ctx.inherent.timestamp;

        while let Some(input) = input.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input, outputs, continue);

            //get cookies
            let cookies = parse_cookies_header(input.headers.clone(), &config.session_cookie_name);

            if cookies.session_id.is_none() {
                //create state cookie
                let state = rng.uuid().to_string();
                // redirect to auth endpoint
                let response =
                    build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                outputs.response.send(&response);
                continue;
            }

            //session cookie exists lookup session to see if its valid
            let session_id = cookies.session_id.unwrap();
            let mut get_oidc_claims_response = ctx
                .provided()
                .dbclient
                .get_oidc_claims(once(session_id.clone()))?;

            while let Some(response) = get_oidc_claims_response.next().await {
                let response = propagate_if_error!(response, outputs, continue);

                //session is valid
                let mut request = input.clone();

                //extract scope from claims
                let scope = response.get("claims");
                if scope.is_none() {
                    //create state cookie
                    let state = rng.uuid().to_string();
                    // redirect to auth endpoint
                    let response =
                        build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                    outputs.response.send(&response);
                    continue;
                }

                let claims: Result<types::OidcClaims, _> =
                    wick_component::from_value(scope.unwrap().to_owned());

                if claims.is_err() {
                    outputs.response.error("invalid claims");
                    continue;
                }

                let claims = claims.unwrap();

                //add scope to request
                request
                    .headers
                    .insert("X-OIDC-EMAIL".to_string(), vec![claims.email]);

                request
                    .headers
                    .insert("X-OIDC-Group".to_string(), claims.groups);

                outputs.request.send(&request);
            }
        }

        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.response.done();
        outputs.request.done();
        Ok(())
    }
}
