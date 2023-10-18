# common/array

This is generated documentation for the array component.


## Operations

### `includes`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `array` | object[] |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `value` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `result` | bool |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "array": "XXX"
}
```

```bash
$ wick invoke common/array:0.1.0 includes --op-with=@op-config.json -- --value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/array:0.1.0 includes \
  --op-with='{ "array":"XXX" }' \
  -- --value="XXX"
```

### `includes_glob`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `array` | object[] |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `value` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `result` | bool |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "array": "XXX"
}
```

```bash
$ wick invoke common/array:0.1.0 includes_glob --op-with=@op-config.json -- --value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/array:0.1.0 includes_glob \
  --op-with='{ "array":"XXX" }' \
  -- --value="XXX"
```

