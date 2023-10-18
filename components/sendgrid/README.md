# candle/sendgrid

This is generated documentation for the sendgrid component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `token` | string |  |


## Operations

### `template_send`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `from` | string |  |
| `to` | object |  |
| `template_id` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `template_data` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "token": "XXX"
}
```

Operation configuration as `op-config.json`:

```json
{ 
  "from": "XXX",
  "to": "XXX",
  "template_id": "XXX"
}
```

```bash
$ wick invoke candle/sendgrid:0.1.0 template_send --with=@config.json --op-with=@op-config.json -- --template_data="XXX"
```

Or with inline configuration:

```bash
$ wick invoke candle/sendgrid:0.1.0 template_send \
  --with='{ "token":"XXX" }' \
  --op-with='{ "from":"XXX","to":"XXX","template_id":"XXX" }' \
  -- --template_data="XXX"
```

