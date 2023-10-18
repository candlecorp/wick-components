# common/azurevault

Manipulate Entires in Azure Vault


## Component-wide configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `tenant_id` | string |  |
| `client_id` | string |  |
| `client_secret` | string |  |
| `vault_url` | string |  |


## Operations

### `create_secret`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `name` | string |  |
| `value` | string |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "tenant_id": "XXX",
  "client_id": "XXX",
  "client_secret": "XXX",
  "vault_url": "XXX"
}
```

```bash
$ wick invoke common/azurevault:0.2.0 create_secret --with=@config.json -- --name="XXX"--value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/azurevault:0.2.0 create_secret \
  --with='{ "tenant_id":"XXX","client_id":"XXX","client_secret":"XXX","vault_url":"XXX" }' \
  -- --name="XXX"--value="XXX"
```

### `delete_secret`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `name` | string |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "tenant_id": "XXX",
  "client_id": "XXX",
  "client_secret": "XXX",
  "vault_url": "XXX"
}
```

```bash
$ wick invoke common/azurevault:0.2.0 delete_secret --with=@config.json -- --name="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/azurevault:0.2.0 delete_secret \
  --with='{ "tenant_id":"XXX","client_id":"XXX","client_secret":"XXX","vault_url":"XXX" }' \
  -- --name="XXX"
```

### `purge_secret`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `name` | string |  |

#### Usage

Given the following configuration:

Component-wide configuration as `config.json`:

```json
{ 
  "tenant_id": "XXX",
  "client_id": "XXX",
  "client_secret": "XXX",
  "vault_url": "XXX"
}
```

```bash
$ wick invoke common/azurevault:0.2.0 purge_secret --with=@config.json -- --name="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/azurevault:0.2.0 purge_secret \
  --with='{ "tenant_id":"XXX","client_id":"XXX","client_secret":"XXX","vault_url":"XXX" }' \
  -- --name="XXX"
```

