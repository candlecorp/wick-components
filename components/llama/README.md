# candle_ml/llama2

This is generated documentation for the llama2 component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `model_dir` | string |  |
| `model` | string |  |
| `tokenizer` | string |  |


## Operations

### `generate`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `max_length` | uint? |  |
| `temperature` | float? |  |
| `top_p` | float? |  |
| `repeat_penalty` | float? |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `prompt` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "model_dir": "XXX",
  "model": "XXX",
  "tokenizer": "XXX"
}
```

Operation configuration as `op-config.json`:

```json
{ 
  "max_length": "XXX",
  "temperature": "XXX",
  "top_p": "XXX",
  "repeat_penalty": "XXX"
}
```

```bash
$ wick invoke candle_ml/llama2:0.0.1 generate --with=@config.json --op-with=@op-config.json -- --prompt="XXX"
```

Or with inline configuration:

```bash
$ wick invoke candle_ml/llama2:0.0.1 generate \
  --with='{ "model_dir":"XXX","model":"XXX","tokenizer":"XXX" }' \
  --op-with='{ "max_length":"XXX","temperature":"XXX","top_p":"XXX","repeat_penalty":"XXX" }' \
  -- --prompt="XXX"
```

