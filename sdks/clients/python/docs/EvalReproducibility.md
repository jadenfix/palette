# EvalReproducibility


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_release_id** | **str** |  |
**code_hash** | **str** |  | [optional]
**dataset_case_id** | **str** |  |
**dataset_version_id** | **str** |  |
**evaluator_version_id** | **str** |  |
**input_artifact_hashes** | **List[str]** |  |
**judge_model_id** | **str** |  | [optional]
**judge_parameters** | **object** |  |
**judge_provider** | **str** |  | [optional]
**judge_rubric_version** | **str** |  | [optional]
**judge_seed** | **int** |  | [optional]
**normalizer_version** | **str** |  |
**prompt_version_id** | **str** |  | [optional]
**trace_schema_version** | **int** |  |
**wasi_abi_version** | **str** |  | [optional]
**wasm_hash** | **str** |  | [optional]

## Example

```python
from beater_client.models.eval_reproducibility import EvalReproducibility

# TODO update the JSON string below
json = "{}"
# create an instance of EvalReproducibility from a JSON string
eval_reproducibility_instance = EvalReproducibility.from_json(json)
# print the JSON string representation of the object
print(EvalReproducibility.to_json())

# convert the object into a dict
eval_reproducibility_dict = eval_reproducibility_instance.to_dict()
# create an instance of EvalReproducibility from a dict
eval_reproducibility_from_dict = EvalReproducibility.from_dict(eval_reproducibility_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
