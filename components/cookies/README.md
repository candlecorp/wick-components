# jsoverson/cookies

This is generated documentation for the cookies component.


## Operations

### `get`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `name` | string |  |


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
  "name": "XXX"
}
```

```bash
$ wick invoke jsoverson/cookies:0.1.0 get --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke jsoverson/cookies:0.1.0 get \
  --op-with='{ "name":"XXX" }' \
  -- --input="XXX"
```

