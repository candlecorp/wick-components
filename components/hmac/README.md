# common/hmac

This is generated documentation for the hmac component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `secret` | bytes |  |


## Operations

### `from_bytes`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | bytes |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | bytes |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "secret": "XXX"
}
```

```bash
$ wick invoke common/hmac:0.3.0 from_bytes --with=@config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/hmac:0.3.0 from_bytes \
  --with='{ "secret":"XXX" }' \
  -- --input="XXX"
```

### `from_string`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | bytes |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "secret": "XXX"
}
```

```bash
$ wick invoke common/hmac:0.3.0 from_string --with=@config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/hmac:0.3.0 from_string \
  --with='{ "secret":"XXX" }' \
  -- --input="XXX"
```

