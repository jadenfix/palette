# BeaterClient::UsageSummary

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **totals** | [**Hash&lt;String, UsageTotal&gt;**](UsageTotal.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::UsageSummary.new(
  project_id: null,
  tenant_id: null,
  totals: null
)
```
