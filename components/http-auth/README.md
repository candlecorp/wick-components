# candle/http-auth

This is generated documentation for the http-auth component.


## Operations

### `basic`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `username` | string |  |
| `password` | string |  |


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

Operation configuration as `op-config.json`:

```json
{ 
  "username": "XXX",
  "password": "XXX"
}
```

```bash
$ wick invoke candle/http-auth:0.1.0 basic --op-with=@op-config.json -- --request="XXX"
```

Or with inline configuration:

```bash
$ wick invoke candle/http-auth:0.1.0 basic \
  --op-with='{ "username":"XXX","password":"XXX" }' \
  -- --request="XXX"
```

