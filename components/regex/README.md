# common/regex

A regular expression component for matching and capturing strings.


## Operations

### `match`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `pattern` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `result` | bool |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "pattern": "XXX"
}
```

```bash
$ wick invoke common/regex:0.2.1 match --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/regex:0.2.1 match \
  --op-with='{ "pattern":"XXX" }' \
  -- --input="XXX"
```

### `capture`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `pattern` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `result` | string[] |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "pattern": "XXX"
}
```

```bash
$ wick invoke common/regex:0.2.1 capture --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/regex:0.2.1 capture \
  --op-with='{ "pattern":"XXX" }' \
  -- --input="XXX"
```

