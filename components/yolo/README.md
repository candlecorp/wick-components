# candle_ml/yolo

This is generated documentation for the yolo component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `model_dir` | string |  |
| `model` | string |  |


## Operations

### `detect`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `confidence` | f32? |  |
| `iou` | f32? |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `image_data` | bytes |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "model_dir": "XXX",
  "model": "XXX"
}
```

Operation configuration as `op-config.json`:

```json
{ 
  "confidence": "XXX",
  "iou": "XXX"
}
```

```bash
$ wick invoke candle_ml/yolo:0.0.2 detect --with=@config.json --op-with=@op-config.json -- --image_data="XXX"
```

Or with inline configuration:

```bash
$ wick invoke candle_ml/yolo:0.0.2 detect \
  --with='{ "model_dir":"XXX","model":"XXX" }' \
  --op-with='{ "confidence":"XXX","iou":"XXX" }' \
  -- --image_data="XXX"
```

