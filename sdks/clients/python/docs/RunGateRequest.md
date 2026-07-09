# RunGateRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**experiment_run_id** | **str** |  | [optional]

## Example

```python
from beater_client.models.run_gate_request import RunGateRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RunGateRequest from a JSON string
run_gate_request_instance = RunGateRequest.from_json(json)
# print the JSON string representation of the object
print(RunGateRequest.to_json())

# convert the object into a dict
run_gate_request_dict = run_gate_request_instance.to_dict()
# create an instance of RunGateRequest from a dict
run_gate_request_from_dict = RunGateRequest.from_dict(run_gate_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
