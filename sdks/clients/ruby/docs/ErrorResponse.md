# BeaterClient::ErrorResponse

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **error** | **String** | Human-readable error message. |  |
| **status** | **Integer** | HTTP status code, duplicated in the body for convenience. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ErrorResponse.new(
  error: null,
  status: null
)
```
