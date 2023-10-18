# <missing>/github

A component that connects to github api


## Operations

### `get_stargazers`

#### Inputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `organization` | string |  |
| `repository` | string |  |


#### Outputs

| Name | Type | Description |
| ---- | ---- | ----------- |
| `stargazers` | string[] |  |

#### Usage

```bash
$ wick invoke <missing>/github:0.1.0 get_stargazers -- --organization="XXX"--repository="XXX"
```

