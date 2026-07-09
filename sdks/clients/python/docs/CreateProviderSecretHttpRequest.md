# CreateProviderSecretHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**display_name** | **str** |  |
**provider** | **str** |  |
**secret_value** | **str** |  |

## Example

```python
from beater_client.models.create_provider_secret_http_request import CreateProviderSecretHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateProviderSecretHttpRequest from a JSON string
create_provider_secret_http_request_instance = CreateProviderSecretHttpRequest.from_json(json)
# print the JSON string representation of the object
print(CreateProviderSecretHttpRequest.to_json())

# convert the object into a dict
create_provider_secret_http_request_dict = create_provider_secret_http_request_instance.to_dict()
# create an instance of CreateProviderSecretHttpRequest from a dict
create_provider_secret_http_request_from_dict = CreateProviderSecretHttpRequest.from_dict(create_provider_secret_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
