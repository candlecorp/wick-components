# common/string

This is generated documentation for the string component.


## Operations

### `concatenate`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `left` | string |  |
| `right` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string |  |

#### Usage

```bash
$ wick invoke common/string:0.5.0 concatenate -- --left="XXX"--right="XXX"
```

### `split`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `separator` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string[] |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "separator": "XXX"
}
```

```bash
$ wick invoke common/string:0.5.0 split --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/string:0.5.0 split \
  --op-with='{ "separator":"XXX" }' \
  -- --input="XXX"
```

### `lowercase`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string |  |

#### Usage

```bash
$ wick invoke common/string:0.5.0 lowercase -- --input="XXX"
```

