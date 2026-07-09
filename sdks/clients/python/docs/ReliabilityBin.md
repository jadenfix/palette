# ReliabilityBin


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accuracy** | **float** |  | [optional]
**bin_index** | **int** |  |
**calibration_gap** | **float** |  | [optional]
**lower_bound** | **float** |  |
**mean_confidence** | **float** |  | [optional]
**sample_count** | **int** |  |
**upper_bound** | **float** |  |

## Example

```python
from beater_client.models.reliability_bin import ReliabilityBin

# TODO update the JSON string below
json = "{}"
# create an instance of ReliabilityBin from a JSON string
reliability_bin_instance = ReliabilityBin.from_json(json)
# print the JSON string representation of the object
print(ReliabilityBin.to_json())

# convert the object into a dict
reliability_bin_dict = reliability_bin_instance.to_dict()
# create an instance of ReliabilityBin from a dict
reliability_bin_from_dict = ReliabilityBin.from_dict(reliability_bin_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
