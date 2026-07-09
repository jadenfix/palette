# JudgeAuditRecord


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cached** | **bool** |  |
**charged_cost** | [**Money**](Money.md) |  |
**created_at** | **datetime** |  |
**evaluator_id** | **str** |  |
**judge_call_id** | **str** |  |
**model** | **str** |  |
**project_id** | **str** |  |
**provider** | **str** |  |
**provider_cost** | [**Money**](Money.md) |  |
**provider_secret_id** | **str** |  |
**request_hash** | **str** |  |
**response_hash** | **str** |  |
**score** | **float** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.judge_audit_record import JudgeAuditRecord

# TODO update the JSON string below
json = "{}"
# create an instance of JudgeAuditRecord from a JSON string
judge_audit_record_instance = JudgeAuditRecord.from_json(json)
# print the JSON string representation of the object
print(JudgeAuditRecord.to_json())

# convert the object into a dict
judge_audit_record_dict = judge_audit_record_instance.to_dict()
# create an instance of JudgeAuditRecord from a dict
judge_audit_record_from_dict = JudgeAuditRecord.from_dict(judge_audit_record_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
