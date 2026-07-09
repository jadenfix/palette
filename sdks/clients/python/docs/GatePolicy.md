# GatePolicy


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**alpha** | **float** |  |
**comparison_count** | **int** |  |
**max_regression** | **float** |  |
**min_sample_size** | **int** |  |

## Example

```python
from beater_client.models.gate_policy import GatePolicy

# TODO update the JSON string below
json = "{}"
# create an instance of GatePolicy from a JSON string
gate_policy_instance = GatePolicy.from_json(json)
# print the JSON string representation of the object
print(GatePolicy.to_json())

# convert the object into a dict
gate_policy_dict = gate_policy_instance.to_dict()
# create an instance of GatePolicy from a dict
gate_policy_from_dict = GatePolicy.from_dict(gate_policy_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
