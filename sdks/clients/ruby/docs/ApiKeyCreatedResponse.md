# BeaterClient::ApiKeyCreatedResponse

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **active** | **Boolean** |  |  |
| **api_key_id** | **String** |  |  |
| **created_at** | **Time** |  |  |
| **environment_id** | **String** |  |  |
| **project_id** | **String** |  |  |
| **scopes** | [**Array&lt;ApiScope&gt;**](ApiScope.md) |  |  |
| **secret** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ApiKeyCreatedResponse.new(
  active: null,
  api_key_id: null,
  created_at: null,
  environment_id: null,
  project_id: null,
  scopes: null,
  secret: null,
  tenant_id: null
)
```
