# DeadLetterReplayReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ack** | [**PublishAck**](PublishAck.md) |  |
**message_id** | **str** |  |
**project_id** | **str** |  |
**reset_attempts** | **bool** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.dead_letter_replay_report import DeadLetterReplayReport

# TODO update the JSON string below
json = "{}"
# create an instance of DeadLetterReplayReport from a JSON string
dead_letter_replay_report_instance = DeadLetterReplayReport.from_json(json)
# print the JSON string representation of the object
print(DeadLetterReplayReport.to_json())

# convert the object into a dict
dead_letter_replay_report_dict = dead_letter_replay_report_instance.to_dict()
# create an instance of DeadLetterReplayReport from a dict
dead_letter_replay_report_from_dict = DeadLetterReplayReport.from_dict(dead_letter_replay_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
