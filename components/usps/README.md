# <missing>/usps

This is generated documentation for the usps component.


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `user_id` | string |  |


## Operations

### `verify`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `address` | usps_types::RequestAddress |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `verified_address` | usps_types::ResponseAddress |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "user_id": "XXX"
}
```

```bash
$ wick invoke <missing>/usps:0.0.1 verify --with=@config.json -- --address="XXX"
```

Or with inline configuration:

```bash
$ wick invoke <missing>/usps:0.0.1 verify \
  --with='{ "user_id":"XXX" }' \
  -- --address="XXX"
```

