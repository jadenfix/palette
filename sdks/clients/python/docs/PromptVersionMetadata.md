# PromptVersionMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**created_by** | **str** |  | [optional] 
**message** | **str** |  | [optional] 

## Example

```python
from beater_client.models.prompt_version_metadata import PromptVersionMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of PromptVersionMetadata from a JSON string
prompt_version_metadata_instance = PromptVersionMetadata.from_json(json)
# print the JSON string representation of the object
print(PromptVersionMetadata.to_json())

# convert the object into a dict
prompt_version_metadata_dict = prompt_version_metadata_instance.to_dict()
# create an instance of PromptVersionMetadata from a dict
prompt_version_metadata_from_dict = PromptVersionMetadata.from_dict(prompt_version_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


