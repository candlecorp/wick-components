mod wick {
    wick_component::wick_import!();
}
use cookie::time::OffsetDateTime as CookieDateTime;

use cookie::Cookie;
use wick::{types::http::HttpResponse, *};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use wick_component::datetime::chrono::Duration;

use std::collections::HashMap;
use urlencoding::encode;
use wick_component::propagate_if_error;

use provided::dbclient;
use provided::httpclient;

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

    let decoded = engine.decode(parts[1])?;

    let claims = String::from_utf8(decoded)?;

    Ok(claims)
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl auth::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = auth::Inputs;
    type Outputs = auth::Outputs;
    type Config = auth::Config;

    async fn auth(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        // initial setup for frequently used variables
        let config: &RootConfig = ctx.root_config();
        let rng = &ctx.inherent.rng;
        let timestamp = ctx.inherent.timestamp;

        while let Some(input) = inputs.request.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input.decode(), outputs, continue);

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
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //get state cookie value
                    if cookies.auth_state.is_none() {
                        //state cookie does not exist
                        let response = build_error_response("State cookie does not exist");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //check state cookie value
                    if cookies.auth_state.unwrap()
                        != input.query_parameters.get("state").unwrap()[0]
                    {
                        //state cookie value does not match
                        let response =
                            build_error_response("State cookie and response don't match");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //handle callback
                    let access_code = input.query_parameters.get("code").unwrap()[0].clone();

                    //call token http component using get_token function
                    let mut res = ctx.provided().httpclient.get_token(
                        httpclient::get_token::Config::default(),
                        httpclient::get_token::Request {
                            access_code: access_code.to_string(),
                            redirect_uri: config.redirect_uri.clone(),
                        },
                    )?;

                    let mut response: Option<HttpResponse> = None;
                    let mut body: Option<types::OAuthTokenResponse> = None;

                    //intentionally this is not a multi-response stream so will only get the first response
                    while let (Some(token_response), Some(token_response_body)) =
                        (res.response.next().await, res.body.next().await)
                    {
                        //ensure response is not an error
                        response = Some(propagate_if_error!(
                            token_response.decode(),
                            outputs,
                            continue
                        ));
                        body = Some(propagate_if_error!(
                            token_response_body.decode(),
                            outputs,
                            continue
                        ));
                    }

                    if response.is_none() || body.is_none() {
                        let response = build_error_response("Token endpoint returned error");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    let response = response.unwrap();
                    let body = body.unwrap();

                    //ensure response is 200
                    if response.status != types::http::StatusCode::Ok {
                        let response = build_error_response("Token endpoint returned error");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //ensure body is not empty
                    if body.access_token.is_empty() || body.token_type.is_empty() {
                        let response =
                            build_error_response("Token endpoint returned invalid response");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //process id_token and extract claims
                    let claims_access = get_oidc_claims(body.access_token.as_str());
                    let claims_access = match claims_access {
                        Ok(claims) => claims,
                        Err(_) => "{}".to_string(),
                    };
                    println!("body: {:?}", body);
                    let claims_id = match body.id_token.clone() {
                        Some(id_token) => get_oidc_claims(id_token.as_str())?,
                        None => "{}".to_string(),
                    };

                    println!("claims_id: {:?}", claims_id);

                    let claims_access: Value = wick_component::from_str(&claims_access)?;
                    let claims_id: Value = wick_component::from_str(&claims_id)?;

                    let claims: Value = match (claims_access, claims_id) {
                        (
                            wick_component::Value::Object(mut map1),
                            wick_component::Value::Object(map2),
                        ) => {
                            map1.extend(map2);
                            wick_component::Value::Object(map1) // Now claims contains both sets of claims
                        }
                        _ => {
                            let response = build_error_response(
                                "Access or ID token claims are not valid JSON",
                            );
                            outputs.output.send(
                                &types::http::RequestMiddlewareResponse::HttpResponse(response),
                            );
                            continue;
                        }
                    };

                    println!("claims: {:?}", claims);

                    let session_id = &rng.uuid();

                    let mut expires: u32 = 3600;

                    if body.expires_in.is_some() {
                        expires = body.expires_in.unwrap();
                    }

                    if config.session_cookie_duration_minutes > 0 {
                        expires = config.session_cookie_duration_minutes * 60;
                    }

                    let expires_at = timestamp + Duration::seconds(expires as _);

                    let mut insert_token_response = ctx.provided().dbclient.insert_token(
                        dbclient::insert_token::Config::default(),
                        dbclient::insert_token::Request {
                            session_id: session_id.to_string(),
                            token_type: body.token_type,
                            access_token: body.access_token,
                            id_token: body.id_token.unwrap_or("".to_string()),
                            refresh_token: "".to_string(),
                            expires_at: expires_at,
                        },
                    )?;

                    while let Some(insert_response) = insert_token_response.output.next().await {
                        let _response =
                            propagate_if_error!(insert_response.decode(), outputs, continue);
                        println!("insert_response: {:?}", _response);
                    }

                    let mut insert_claims_response = ctx.provided().dbclient.insert_claims(
                        dbclient::insert_claims::Config::default(),
                        dbclient::insert_claims::Request {
                            session_id: session_id.to_string(),
                            claims: claims.to_string(),
                        },
                    )?;

                    while let Some(insert_response) = insert_claims_response.output.next().await {
                        let _response =
                            propagate_if_error!(insert_response.decode(), outputs, continue);
                        println!("insert_response: {:?}", _response);
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
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                            response,
                        ));
                    continue;
                }
                "/oidc/logout" => {
                    let location = config.logout_endpoint.clone();

                    //get session cookie value
                    if cookies.session_id.is_none() {
                        //session cookie does not exist
                        let response = build_error_response("Session cookie does not exist");
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    let expires_at_old = timestamp + Duration::seconds(-1);
                    let session_cookie =
                        Cookie::build(config.session_cookie_name.clone(), "expired")
                            .path("/")
                            .http_only(true)
                            .expires(
                                CookieDateTime::from_unix_timestamp(expires_at_old.timestamp())
                                    .unwrap(),
                            )
                            .finish();

                    let mut resp_cookies: Vec<Cookie> = vec![];
                    resp_cookies.push(session_cookie);

                    let redirect_logout = config.redirect_logout.clone();

                    if redirect_logout == "true" {
                        let logout_redirect_uri = config.logout_redirect_uri.clone();
                        let mut location = format!(
                            "{}?post_logout_redirect_uri={}",
                            location, logout_redirect_uri
                        );

                        let mut get_login_hint_response =
                            ctx.provided().dbclient.get_login_hint_claim(
                                dbclient::get_login_hint_claim::Config::default(),
                                dbclient::get_login_hint_claim::Request {
                                    session_id: cookies.session_id.clone().unwrap().to_string(),
                                },
                            )?;

                        while let Some(login_hint_response) =
                            get_login_hint_response.output.next().await
                        {
                            let login_hint = login_hint_response.decode()?;
                            println!("insert_response: {:?}", login_hint);

                            let login_hint_val = match login_hint.get("login_hint") {
                                Some(login_hint) => {
                                    let login_hint = login_hint.as_str();
                                    match login_hint {
                                        Some(login_hint) => login_hint,
                                        None => "",
                                    }
                                }
                                None => "",
                            };

                            if login_hint_val != "" {
                                //append login_hint to logout endpoint
                                location = format!("{}&logout_hint={}", location, login_hint_val);

                                let response =
                                    build_redirect_response(&location, Some(resp_cookies.clone()));
                                println!("response: {:?}", response);
                                outputs.output.send(
                                    &types::http::RequestMiddlewareResponse::HttpResponse(response),
                                );
                                outputs.output.done();
                                return Ok(());
                            }
                            continue;
                        }

                        let mut get_id_token_response = ctx.provided().dbclient.get_id_token(
                            dbclient::get_id_token::Config::default(),
                            dbclient::get_id_token::Request {
                                session_id: cookies.session_id.clone().unwrap().to_string(),
                            },
                        )?;

                        while let Some(id_token) = get_id_token_response.output.next().await {
                            let id_token: types::IdToken = id_token.decode()?;
                            location =
                                format!("{}&id_token_hint={}", location, id_token.id_token.clone());
                        }

                        //if login hint does not exist redirect to logout endpoint
                        let response =
                            build_redirect_response(&location, Some(resp_cookies.clone()));
                        println!("response: {:?}", response);
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        outputs.output.done();
                        return Ok(());
                    }

                    //if login hint does not exist redirect to logout endpoint
                    let response = build_redirect_response(&location, Some(resp_cookies.clone()));
                    println!("response: {:?}", response);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                            response,
                        ));
                    outputs.output.done();
                    return Ok(());
                }
                _ => {
                    //handle all other requests

                    //session cookie does not exist redirect to login
                    if cookies.session_id.is_none() {
                        println!("No Session Cookie. Redirecting to auth.");
                        //create state cookie
                        let state = rng.uuid().to_string();
                        // redirect to auth endpoint
                        let response = build_auth_redirect_response(
                            config.clone(),
                            &state,
                            &input.uri,
                            timestamp,
                        );
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                                response,
                            ));
                        continue;
                    }

                    //session cookie exists lookup session to see if its valid
                    let session_id = cookies.session_id.unwrap();
                    let mut get_session_response = ctx.provided().dbclient.get_session(
                        dbclient::get_session::Config::default(),
                        dbclient::get_session::Request {
                            session_id: session_id.clone(),
                        },
                    )?;

                    while let Some(response) = get_session_response.output.next().await {
                        let response: types::SessionDetails =
                            propagate_if_error!(response.decode(), outputs, continue);

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
                            outputs.output.send(
                                &types::http::RequestMiddlewareResponse::HttpResponse(response),
                            );
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
                            outputs.output.send(
                                &types::http::RequestMiddlewareResponse::HttpResponse(response),
                            );
                            continue;
                        }

                        //session is valid
                        outputs
                            .output
                            .send(&types::http::RequestMiddlewareResponse::HttpRequest(
                                input.clone(),
                            ));
                    }
                    //session does not exist redirect to login

                    //create state cookie
                    let state = rng.uuid().to_string();
                    // redirect to auth endpoint
                    let response =
                        build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                            response,
                        ));
                    continue;
                }
            }
        }
        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.output.done();
        println!("done");
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl oidc::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = oidc::Inputs;
    type Outputs = oidc::Outputs;
    type Config = oidc::Config;

    async fn oidc(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        // initial setup for frequently used variables
        let config: &RootConfig = ctx.root_config();
        let rng = &ctx.inherent.rng;
        let timestamp = ctx.inherent.timestamp;

        while let Some(input) = inputs.request.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input.decode(), outputs, continue);

            //get cookies
            let cookies = parse_cookies_header(input.headers.clone(), &config.session_cookie_name);

            if cookies.session_id.is_none() {
                //create state cookie
                let state = rng.uuid().to_string();
                // redirect to auth endpoint
                let response =
                    build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                outputs
                    .output
                    .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                        response,
                    ));
                continue;
            }

            //session cookie exists lookup session to see if its valid
            let session_id = cookies.session_id.unwrap();

            println!("session_id: {:?}", session_id);

            let mut get_oidc_claims_response = ctx.provided().dbclient.get_oidc_claims(
                dbclient::get_oidc_claims::Config::default(),
                dbclient::get_oidc_claims::Request {
                    session_id: session_id.clone(),
                },
            )?;

            while let Some(response) = get_oidc_claims_response.output.next().await {
                let response = propagate_if_error!(response.decode(), outputs, continue);

                //session is valid
                let mut request = input.clone();

                //extract scope from claims
                let claims = response.get("claims");
                if claims.is_none() {
                    //create state cookie
                    let state = rng.uuid().to_string();
                    // redirect to auth endpoint
                    let response =
                        build_auth_redirect_response(config.clone(), &state, &input.uri, timestamp);
                    outputs
                        .output
                        .send(&types::http::RequestMiddlewareResponse::HttpResponse(
                            response,
                        ));
                    continue;
                }

                println!("scope: {:?}", claims.unwrap());

                let mut parsed_claims = claims.unwrap().to_owned();

                if parsed_claims.is_string() {
                    parsed_claims =
                        wick_component::from_str::<Value>(parsed_claims.as_str().unwrap()).unwrap();
                }

                println!("parsed_claims: {:?}", parsed_claims);

                let claims: Result<Value, _> = wick_component::from_value(parsed_claims);

                if claims.is_err() {
                    outputs.output.error("invalid claims");
                    continue;
                }

                let claims = claims.unwrap();

                let _sub = claims
                    .get("sub")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let email = claims
                    .get(&config.email_claim)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let mut groups = Vec::new();

                if config.groups_claim.is_some() {
                    groups = match claims.get(&config.groups_claim.as_ref().unwrap()) {
                        Some(groups) => wick_component::from_value(groups.to_owned()).unwrap(),
                        None => Vec::new(),
                    };
                }

                //add scope to request
                request
                    .headers
                    .insert("x-oidc-email".to_string(), vec![email]);

                request.headers.insert("x-oidc-group".to_string(), groups);

                outputs
                    .output
                    .send(&types::http::RequestMiddlewareResponse::HttpRequest(
                        request,
                    ));
            }
        }

        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.output.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl get_user::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = get_user::Inputs;
    type Outputs = get_user::Outputs;
    type Config = get_user::Config;

    async fn get_user(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        // initial setup for frequently used variables
        let config: &RootConfig = ctx.root_config();

        while let Some(input) = inputs.request.next().await {
            //ensure request is not an error
            let input = propagate_if_error!(input.decode(), outputs, continue);

            //get cookies
            let cookies = parse_cookies_header(input.headers.clone(), &config.session_cookie_name);

            if cookies.session_id.is_none() {
                let response = build_error_response("Session Cookie does not exist");
                outputs.response.send(&response);
                continue;
            }

            //session cookie exists lookup session to see if its valid
            let session_id = cookies.session_id.unwrap();
            let mut get_oidc_claims_response = ctx.provided().dbclient.get_oidc_claims(
                dbclient::get_oidc_claims::Config::default(),
                dbclient::get_oidc_claims::Request {
                    session_id: session_id.clone(),
                },
            )?;

            let mut claims_responses = 0;

            while let Some(response) = get_oidc_claims_response.output.next().await {
                let response = propagate_if_error!(response.decode(), outputs, continue);

                //extract scope from claims
                let scope = response.get("claims");
                if scope.is_none() {
                    let response = build_error_response("Invalid OIDC Claims");
                    let user_info = types::UserInfo {
                        sub: "".to_string(),
                        email: "".to_string(),
                        groups: vec![],
                    };
                    outputs.response.send(&response);
                    outputs.body.send(&user_info);
                    continue;
                }

                let mut parsed_scope = scope.unwrap().to_owned();

                if parsed_scope.is_string() {
                    parsed_scope =
                        wick_component::from_str::<Value>(parsed_scope.as_str().unwrap()).unwrap();
                }

                let claims: Result<Value, _> = wick_component::from_value(parsed_scope);

                if claims.is_err() {
                    let response = build_error_response("Invalid OIDC Claims");
                    outputs.response.send(&response);
                    let user_info = types::UserInfo {
                        sub: "".to_string(),
                        email: "".to_string(),
                        groups: vec![],
                    };
                    outputs.body.send(&user_info);
                    continue;
                }

                let claims = claims.unwrap();

                let sub = claims
                    .get("sub")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                println!("email claim: {:?}", config.email_claim);

                let email = claims
                    .get(&config.email_claim)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let mut groups = Vec::new();

                if config.groups_claim.is_some() {
                    groups = match claims.get(&config.groups_claim.as_ref().unwrap()) {
                        Some(groups) => wick_component::from_value(groups.to_owned()).unwrap(),
                        None => Vec::new(),
                    };
                }

                let user_info: types::UserInfo = {
                    types::UserInfo {
                        sub: sub,
                        email: email,
                        groups: groups,
                    }
                };

                let response = HttpResponse {
                    version: types::http::HttpVersion::Http11,
                    status: types::http::StatusCode::Ok,
                    headers: HashMap::new(),
                };
                outputs.response.send(&response);
                outputs.body.send(&user_info);
                claims_responses += 1;
            }

            if claims_responses == 0 {
                let response = build_error_response("Session Not Found");
                let user_info = types::UserInfo {
                    sub: "".to_string(),
                    email: "".to_string(),
                    groups: vec![],
                };
                outputs.response.send(&response);
                outputs.body.send(&user_info);
            }
        }

        //This should always be at the end. This lets the downstream components know that the stream is finished and there will not be any more messages.
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}
