# <missing>/oauth_engine

Oauth Middleware component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `auth_endpoint` | string |  |
| `logout_endpoint` | string |  |
| `redirect_uri` | string |  |
| `session_cookie_name` | string |  |
| `client_id` | string |  |
| `scope` | string |  |
| `email_claim` | string |  |
| `groups_claim` | string? |  |


## Operations

### `auth`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `request` | http::HttpRequest |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | http::RequestMiddlewareResponse |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "auth_endpoint": "XXX",
  "logout_endpoint": "XXX",
  "redirect_uri": "XXX",
  "session_cookie_name": "XXX",
  "client_id": "XXX",
  "scope": "XXX",
  "email_claim": "XXX",
  "groups_claim": "XXX"
}
```

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 auth --with=@config.json -- --request="XXX"
```

Or with inline configuration:

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 auth \
  --with='{ "auth_endpoint":"XXX","logout_endpoint":"XXX","redirect_uri":"XXX","session_cookie_name":"XXX","client_id":"XXX","scope":"XXX","email_claim":"XXX","groups_claim":"XXX" }' \
  -- --request="XXX"
```

### `oidc`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `request` | http::HttpRequest |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | http::RequestMiddlewareResponse |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "auth_endpoint": "XXX",
  "logout_endpoint": "XXX",
  "redirect_uri": "XXX",
  "session_cookie_name": "XXX",
  "client_id": "XXX",
  "scope": "XXX",
  "email_claim": "XXX",
  "groups_claim": "XXX"
}
```

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 oidc --with=@config.json -- --request="XXX"
```

Or with inline configuration:

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 oidc \
  --with='{ "auth_endpoint":"XXX","logout_endpoint":"XXX","redirect_uri":"XXX","session_cookie_name":"XXX","client_id":"XXX","scope":"XXX","email_claim":"XXX","groups_claim":"XXX" }' \
  -- --request="XXX"
```

### `get_user`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `request` | http::HttpRequest |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `response` | http::HttpResponse |  |
| `body` | UserInfo |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "auth_endpoint": "XXX",
  "logout_endpoint": "XXX",
  "redirect_uri": "XXX",
  "session_cookie_name": "XXX",
  "client_id": "XXX",
  "scope": "XXX",
  "email_claim": "XXX",
  "groups_claim": "XXX"
}
```

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 get_user --with=@config.json -- --request="XXX"
```

Or with inline configuration:

```bash
$ wick invoke <missing>/oauth_engine:0.1.0 get_user \
  --with='{ "auth_endpoint":"XXX","logout_endpoint":"XXX","redirect_uri":"XXX","session_cookie_name":"XXX","client_id":"XXX","scope":"XXX","email_claim":"XXX","groups_claim":"XXX" }' \
  -- --request="XXX"
```

