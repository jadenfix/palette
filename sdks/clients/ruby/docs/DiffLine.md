# BeaterClient::DiffLine

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **kind** | [**DiffLineKind**](DiffLineKind.md) |  |  |
| **new_line** | **Integer** |  | [optional] |
| **old_line** | **Integer** |  | [optional] |
| **text** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DiffLine.new(
  kind: null,
  new_line: null,
  old_line: null,
  text: null
)
```
