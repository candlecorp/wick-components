# common/liquid

A template engine for creating strings using Liquid syntax.


## Operations

### `render`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `template` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `context` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "template": "XXX"
}
```

```bash
$ wick invoke common/liquid:0.3.0 render --op-with=@op-config.json -- --context="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/liquid:0.3.0 render \
  --op-with='{ "template":"XXX" }' \
  -- --context="XXX"
```

