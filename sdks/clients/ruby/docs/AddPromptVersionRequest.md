# BeaterClient::AddPromptVersionRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **created_by** | **String** |  | [optional] |
| **message** | **String** |  | [optional] |
| **template** | [**PromptTemplate**](PromptTemplate.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::AddPromptVersionRequest.new(
  created_by: null,
  message: null,
  template: null
)
```
