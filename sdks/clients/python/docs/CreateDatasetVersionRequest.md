# CreateDatasetVersionRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**case_ids** | **List[str]** |  | [optional]

## Example

```python
from beater_client.models.create_dataset_version_request import CreateDatasetVersionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateDatasetVersionRequest from a JSON string
create_dataset_version_request_instance = CreateDatasetVersionRequest.from_json(json)
# print the JSON string representation of the object
print(CreateDatasetVersionRequest.to_json())

# convert the object into a dict
create_dataset_version_request_dict = create_dataset_version_request_instance.to_dict()
# create an instance of CreateDatasetVersionRequest from a dict
create_dataset_version_request_from_dict = CreateDatasetVersionRequest.from_dict(create_dataset_version_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
