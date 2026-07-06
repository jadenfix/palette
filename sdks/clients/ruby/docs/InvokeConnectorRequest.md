# BeaterClient::InvokeConnectorRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **arguments** | **Object** | Arguments object matching the tool&#39;s input schema. | [optional] |
| **tool** | **String** | Tool slug to execute (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::InvokeConnectorRequest.new(
  arguments: null,
  tool: null
)
```
