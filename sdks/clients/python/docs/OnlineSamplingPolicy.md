# OnlineSamplingPolicy


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**high_cost_micros_threshold** | **int** |  | [optional]
**keep_errors** | **bool** |  |
**sample_rate_per_mille** | **int** |  |
**slow_ms_threshold** | **int** |  | [optional]

## Example

```python
from beater_client.models.online_sampling_policy import OnlineSamplingPolicy

# TODO update the JSON string below
json = "{}"
# create an instance of OnlineSamplingPolicy from a JSON string
online_sampling_policy_instance = OnlineSamplingPolicy.from_json(json)
# print the JSON string representation of the object
print(OnlineSamplingPolicy.to_json())

# convert the object into a dict
online_sampling_policy_dict = online_sampling_policy_instance.to_dict()
# create an instance of OnlineSamplingPolicy from a dict
online_sampling_policy_from_dict = OnlineSamplingPolicy.from_dict(online_sampling_policy_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
