# common/jinja

This is generated documentation for the jinja component.


## Operations

### `render`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `template` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `data` | object |  |


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
$ wick invoke common/jinja:0.1.0 render --op-with=@op-config.json -- --data="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/jinja:0.1.0 render \
  --op-with='{ "template":"XXX" }' \
  -- --data="XXX"
```

