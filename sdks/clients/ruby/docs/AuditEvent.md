# BeaterClient::AuditEvent

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **action** | [**AuditAction**](AuditAction.md) |  |  |
| **actor_api_key_id** | **String** |  | [optional] |
| **attributes** | **Object** |  |  |
| **audit_event_id** | **String** |  |  |
| **created_at** | **Time** |  |  |
| **environment_id** | **String** |  | [optional] |
| **outcome** | [**AuditOutcome**](AuditOutcome.md) |  |  |
| **project_id** | **String** |  |  |
| **reason** | **String** |  | [optional] |
| **resource_id** | **String** |  |  |
| **resource_type** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::AuditEvent.new(
  action: null,
  actor_api_key_id: null,
  attributes: null,
  audit_event_id: null,
  created_at: null,
  environment_id: null,
  outcome: null,
  project_id: null,
  reason: null,
  resource_id: null,
  resource_type: null,
  tenant_id: null
)
```
