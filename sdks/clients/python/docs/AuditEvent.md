# AuditEvent


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | [**AuditAction**](AuditAction.md) |  |
**actor_api_key_id** | **str** |  | [optional]
**attributes** | **object** |  |
**audit_event_id** | **str** |  |
**created_at** | **datetime** |  |
**environment_id** | **str** |  | [optional]
**outcome** | [**AuditOutcome**](AuditOutcome.md) |  |
**project_id** | **str** |  |
**reason** | **str** |  | [optional]
**resource_id** | **str** |  |
**resource_type** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.audit_event import AuditEvent

# TODO update the JSON string below
json = "{}"
# create an instance of AuditEvent from a JSON string
audit_event_instance = AuditEvent.from_json(json)
# print the JSON string representation of the object
print(AuditEvent.to_json())

# convert the object into a dict
audit_event_dict = audit_event_instance.to_dict()
# create an instance of AuditEvent from a dict
audit_event_from_dict = AuditEvent.from_dict(audit_event_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
