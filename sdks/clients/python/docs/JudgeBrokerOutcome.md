# JudgeBrokerOutcome


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**audit** | [**JudgeAuditRecord**](JudgeAuditRecord.md) |  |
**remaining_budget** | [**Money**](Money.md) |  |
**result** | [**ScoreResult**](ScoreResult.md) |  |

## Example

```python
from beater_client.models.judge_broker_outcome import JudgeBrokerOutcome

# TODO update the JSON string below
json = "{}"
# create an instance of JudgeBrokerOutcome from a JSON string
judge_broker_outcome_instance = JudgeBrokerOutcome.from_json(json)
# print the JSON string representation of the object
print(JudgeBrokerOutcome.to_json())

# convert the object into a dict
judge_broker_outcome_dict = judge_broker_outcome_instance.to_dict()
# create an instance of JudgeBrokerOutcome from a dict
judge_broker_outcome_from_dict = JudgeBrokerOutcome.from_dict(judge_broker_outcome_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
