# common/wickdoc

A documentation generator for wick component definitions.


## Operations

### `generate_readme`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `input` | string | The wick configuration YAML to render. |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `output` | string | The generated readme. |

#### Usage

```bash
$ wick invoke common/wickdoc:0.1.0 generate_readme -- --input="XXX"
```

