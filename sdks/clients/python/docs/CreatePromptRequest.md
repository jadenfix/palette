# CreatePromptRequest

Request body for `createPrompt`: the new prompt's metadata plus its initial (version 1) template.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_by** | **str** |  | [optional]
**description** | **str** |  | [optional]
**message** | **str** |  | [optional]
**name** | **str** |  |
**template** | [**PromptTemplate**](PromptTemplate.md) |  |

## Example

```python
from beater_client.models.create_prompt_request import CreatePromptRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreatePromptRequest from a JSON string
create_prompt_request_instance = CreatePromptRequest.from_json(json)
# print the JSON string representation of the object
print(CreatePromptRequest.to_json())

# convert the object into a dict
create_prompt_request_dict = create_prompt_request_instance.to_dict()
# create an instance of CreatePromptRequest from a dict
create_prompt_request_from_dict = CreatePromptRequest.from_dict(create_prompt_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
