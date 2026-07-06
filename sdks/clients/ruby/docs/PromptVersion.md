# BeaterClient::PromptVersion

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **metadata** | [**PromptVersionMetadata**](PromptVersionMetadata.md) |  |  |
| **project_id** | **String** |  |  |
| **prompt_id** | **String** |  |  |
| **template** | [**PromptTemplate**](PromptTemplate.md) |  |  |
| **tenant_id** | **String** |  |  |
| **version_id** | **String** |  |  |
| **version_number** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::PromptVersion.new(
  metadata: null,
  project_id: null,
  prompt_id: null,
  template: null,
  tenant_id: null,
  version_id: null,
  version_number: null
)
```
