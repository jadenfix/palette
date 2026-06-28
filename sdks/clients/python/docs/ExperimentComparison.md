# ExperimentComparison


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**adjusted_alpha** | **float** |  | 
**baseline_mean** | **float** |  | 
**candidate_mean** | **float** |  | 
**ci_high** | **float** |  | 
**ci_low** | **float** |  | 
**decision** | [**GateDecision**](GateDecision.md) |  | 
**delta** | **float** |  | 
**p_value** | **float** | Real two-sided p-value from &#x60;test&#x60;. The previous normal-approximation path reported no p-value at all. | 
**sample_size** | **int** |  | 
**test** | [**StatisticalTest**](StatisticalTest.md) |  | 

## Example

```python
from beater_client.models.experiment_comparison import ExperimentComparison

# TODO update the JSON string below
json = "{}"
# create an instance of ExperimentComparison from a JSON string
experiment_comparison_instance = ExperimentComparison.from_json(json)
# print the JSON string representation of the object
print(ExperimentComparison.to_json())

# convert the object into a dict
experiment_comparison_dict = experiment_comparison_instance.to_dict()
# create an instance of ExperimentComparison from a dict
experiment_comparison_from_dict = ExperimentComparison.from_dict(experiment_comparison_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


