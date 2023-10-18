# common/openapi-ui

This is generated documentation for the openapi-ui component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `schema_url` | string |  |


## Operations

### `serve`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `request` | http::HttpRequest |  |
| `body` | bytes |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `body` | bytes |  |
| `response` | http::HttpResponse |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "schema_url": "XXX"
}
```

```bash
$ wick invoke common/openapi-ui:0.4.0 serve --with=@config.json -- --request="XXX"--body="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/openapi-ui:0.4.0 serve \
  --with='{ "schema_url":"XXX" }' \
  -- --request="XXX"--body="XXX"
```

