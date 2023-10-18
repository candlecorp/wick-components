# candle/azure-openai

This is generated documentation for the azure-openai component.


## Operations

### `parse_completion`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `event` | http::HttpEvent |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `event` | http::HttpEvent |  |
| `tokens` | u32 |  |

#### Usage

```bash
$ wick invoke candle/azure-openai:0.2.1 parse_completion -- --event="XXX"
```

