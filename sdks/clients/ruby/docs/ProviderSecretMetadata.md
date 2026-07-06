# BeaterClient::ProviderSecretMetadata

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **active** | **Boolean** |  |  |
| **created_at** | **Time** |  |  |
| **display_name** | **String** |  |  |
| **project_id** | **String** |  |  |
| **provider** | **String** |  |  |
| **provider_secret_id** | **String** |  |  |
| **rotated_at** | **Time** |  | [optional] |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ProviderSecretMetadata.new(
  active: null,
  created_at: null,
  display_name: null,
  project_id: null,
  provider: null,
  provider_secret_id: null,
  rotated_at: null,
  tenant_id: null
)
```
