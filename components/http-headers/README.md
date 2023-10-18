# candle/http-headers

Operations to manipulate HTTP headers for both request and response


## Operations

### `add`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `header` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | http::RequestMiddlewareResponse |  |
| `value` | Strings |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | http::RequestMiddlewareResponse |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "header": "XXX"
}
```

```bash
$ wick invoke candle/http-headers:0.1.0 add --op-with=@op-config.json -- --input="XXX"--value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke candle/http-headers:0.1.0 add \
  --op-with='{ "header":"XXX" }' \
  -- --input="XXX"--value="XXX"
```

