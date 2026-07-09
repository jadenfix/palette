# PromptVersionDiff


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from_version_id** | **str** |  |
**lines** | [**List[DiffLine]**](DiffLine.md) |  |
**to_version_id** | **str** |  |

## Example

```python
from beater_client.models.prompt_version_diff import PromptVersionDiff

# TODO update the JSON string below
json = "{}"
# create an instance of PromptVersionDiff from a JSON string
prompt_version_diff_instance = PromptVersionDiff.from_json(json)
# print the JSON string representation of the object
print(PromptVersionDiff.to_json())

# convert the object into a dict
prompt_version_diff_dict = prompt_version_diff_instance.to_dict()
# create an instance of PromptVersionDiff from a dict
prompt_version_diff_from_dict = PromptVersionDiff.from_dict(prompt_version_diff_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
