# common/liquid-json

A component for rendering liquid templates as structured JSON.


## Operations

### `render`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `template` | object |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "template": "XXX"
}
```

```bash
$ wick invoke common/liquid-json:0.2.0 render --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/liquid-json:0.2.0 render \
  --op-with='{ "template":"XXX" }' \
  -- --input="XXX"
```

