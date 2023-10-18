# common/azureblob

Access files on azure blob storage


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `storage_account` | string |  |
| `container_name` | string |  |
| `access_key` | bytes |  |


## Operations

### `get`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `resource` | string |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "storage_account": "XXX",
  "container_name": "XXX",
  "access_key": "XXX"
}
```

```bash
$ wick invoke common/azureblob:0.5.0 get --with=@config.json -- --resource="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/azureblob:0.5.0 get \
  --with='{ "storage_account":"XXX","container_name":"XXX","access_key":"XXX" }' \
  -- --resource="XXX"
```

