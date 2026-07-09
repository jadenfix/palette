# SamplingDecision


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**reason** | [**SamplingReason**](SamplingReason.md) |  |
**selected** | **bool** |  |
**stable_score_per_mille** | **int** |  |

## Example

```python
from beater_client.models.sampling_decision import SamplingDecision

# TODO update the JSON string below
json = "{}"
# create an instance of SamplingDecision from a JSON string
sampling_decision_instance = SamplingDecision.from_json(json)
# print the JSON string representation of the object
print(SamplingDecision.to_json())

# convert the object into a dict
sampling_decision_dict = sampling_decision_instance.to_dict()
# create an instance of SamplingDecision from a dict
sampling_decision_from_dict = SamplingDecision.from_dict(sampling_decision_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
