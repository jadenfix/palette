# BeaterClient::CreatePromptRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **created_by** | **String** |  | [optional] |
| **description** | **String** |  | [optional] |
| **message** | **String** |  | [optional] |
| **name** | **String** |  |  |
| **template** | [**PromptTemplate**](PromptTemplate.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CreatePromptRequest.new(
  created_by: null,
  description: null,
  message: null,
  name: null,
  template: null
)
```
