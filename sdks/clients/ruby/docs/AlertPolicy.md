# BeaterClient::AlertPolicy

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **dedupe_window_seconds** | **Integer** |  |  |
| **endpoint_url** | **String** |  |  |
| **fire_when_score_at_or_below** | **Float** |  |  |
| **maintenance_windows** | [**Array&lt;MaintenanceWindow&gt;**](MaintenanceWindow.md) |  |  |
| **policy_id** | **String** |  |  |
| **severity** | [**AlertSeverity**](AlertSeverity.md) |  |  |
| **signing_secret** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::AlertPolicy.new(
  dedupe_window_seconds: null,
  endpoint_url: null,
  fire_when_score_at_or_below: null,
  maintenance_windows: null,
  policy_id: null,
  severity: null,
  signing_secret: null
)
```
