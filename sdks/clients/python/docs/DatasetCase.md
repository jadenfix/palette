# DatasetCase


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**case_id** | **str** |  |
**created_at** | **datetime** |  |
**dataset_id** | **str** |  |
**input** | **object** |  |
**input_artifact_hashes** | **List[str]** |  |
**normalizer_version** | **str** |  |
**output** | **object** |  |
**project_id** | **str** |  |
**reference** | **object** |  | [optional]
**source_environment_id** | **str** |  |
**source_span_id** | **str** |  |
**source_trace_id** | **str** |  |
**tenant_id** | **str** |  |
**trace** | **object** |  |
**trace_schema_version** | **int** |  |

## Example

```python
from beater_client.models.dataset_case import DatasetCase

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetCase from a JSON string
dataset_case_instance = DatasetCase.from_json(json)
# print the JSON string representation of the object
print(DatasetCase.to_json())

# convert the object into a dict
dataset_case_dict = dataset_case_instance.to_dict()
# create an instance of DatasetCase from a dict
dataset_case_from_dict = DatasetCase.from_dict(dataset_case_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
