# PromptVersion


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metadata** | [**PromptVersionMetadata**](PromptVersionMetadata.md) |  |
**project_id** | **str** |  |
**prompt_id** | **str** |  |
**template** | [**PromptTemplate**](PromptTemplate.md) |  |
**tenant_id** | **str** |  |
**version_id** | **str** |  |
**version_number** | **int** |  |

## Example

```python
from beater_client.models.prompt_version import PromptVersion

# TODO update the JSON string below
json = "{}"
# create an instance of PromptVersion from a JSON string
prompt_version_instance = PromptVersion.from_json(json)
# print the JSON string representation of the object
print(PromptVersion.to_json())

# convert the object into a dict
prompt_version_dict = prompt_version_instance.to_dict()
# create an instance of PromptVersion from a dict
prompt_version_from_dict = PromptVersion.from_dict(prompt_version_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
