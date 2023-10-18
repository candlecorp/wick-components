# common/object

A component that will take differnet types of contents and parse them into a common object that can be used in the wick flow. Can also create and manipulate new / existing objects. Uses JsonPath notation.


## Operations

### `serialize`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `content_type` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `content` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "content_type": "XXX"
}
```

```bash
$ wick invoke common/object:0.5.0 serialize --op-with=@op-config.json -- --content="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/object:0.5.0 serialize \
  --op-with='{ "content_type":"XXX" }' \
  -- --content="XXX"
```

### `new`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `key` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `value` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "key": "XXX"
}
```

```bash
$ wick invoke common/object:0.5.0 new --op-with=@op-config.json -- --value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/object:0.5.0 new \
  --op-with='{ "key":"XXX" }' \
  -- --value="XXX"
```

### `select`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `path` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "path": "XXX"
}
```

```bash
$ wick invoke common/object:0.5.0 select --op-with=@op-config.json -- --input="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/object:0.5.0 select \
  --op-with='{ "path":"XXX" }' \
  -- --input="XXX"
```

### `push`

#### Operation Configuration

| Name | Type | Description |
| ---- | ---- | ----------- |
| `path` | string |  |


#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | object |  |
| `value` | object |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | object |  |

#### Usage

Given the following configuration:

Operation configuration as `op-config.json`:

```json
{ 
  "path": "XXX"
}
```

```bash
$ wick invoke common/object:0.5.0 push --op-with=@op-config.json -- --input="XXX"--value="XXX"
```

Or with inline configuration:

```bash
$ wick invoke common/object:0.5.0 push \
  --op-with='{ "path":"XXX" }' \
  -- --input="XXX"--value="XXX"
```

