# CreateApiKeyHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**scopes** | [**List[ApiScope]**](ApiScope.md) |  |

## Example

```python
from beater_client.models.create_api_key_http_request import CreateApiKeyHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateApiKeyHttpRequest from a JSON string
create_api_key_http_request_instance = CreateApiKeyHttpRequest.from_json(json)
# print the JSON string representation of the object
print(CreateApiKeyHttpRequest.to_json())

# convert the object into a dict
create_api_key_http_request_dict = create_api_key_http_request_instance.to_dict()
# create an instance of CreateApiKeyHttpRequest from a dict
create_api_key_http_request_from_dict = CreateApiKeyHttpRequest.from_dict(create_api_key_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
