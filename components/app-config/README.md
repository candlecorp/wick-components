# common/app-config

This is generated documentation for the app-config component.


## Operations

### `simple`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `app_name` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `dir` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `yaml` | string |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "app_name": "XXX"
}
```

```bash
$ wick invoke common/app-config:0.0.1 simple --op-with=@op-config.json -- --dir="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/app-config:0.0.1 simple \
  --op-with='{ "app_name":"XXX" }' \
  -- --dir="XXX"
```

