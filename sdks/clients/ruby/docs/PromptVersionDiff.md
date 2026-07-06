# BeaterClient::PromptVersionDiff

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **from_version_id** | **String** |  |  |
| **lines** | [**Array&lt;DiffLine&gt;**](DiffLine.md) |  |  |
| **to_version_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::PromptVersionDiff.new(
  from_version_id: null,
  lines: null,
  to_version_id: null
)
```
