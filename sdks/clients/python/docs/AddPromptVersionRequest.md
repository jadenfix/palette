# AddPromptVersionRequest

Request body for `addPromptVersion`: a new immutable template revision.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_by** | **str** |  | [optional]
**message** | **str** |  | [optional]
**template** | [**PromptTemplate**](PromptTemplate.md) |  |

## Example

```python
from beater_client.models.add_prompt_version_request import AddPromptVersionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AddPromptVersionRequest from a JSON string
add_prompt_version_request_instance = AddPromptVersionRequest.from_json(json)
# print the JSON string representation of the object
print(AddPromptVersionRequest.to_json())

# convert the object into a dict
add_prompt_version_request_dict = add_prompt_version_request_instance.to_dict()
# create an instance of AddPromptVersionRequest from a dict
add_prompt_version_request_from_dict = AddPromptVersionRequest.from_dict(add_prompt_version_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
