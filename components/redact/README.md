# common/redact

This is generated documentation for the redact component.


## Operations

### `regex`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `patterns` | string[] |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "patterns": "XXX"
}
```

```bash
$ wick invoke common/redact:0.0.1 regex --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/redact:0.0.1 regex \
  --op-with='{ "patterns":"XXX" }' \
  -- --input="XXX"
```

