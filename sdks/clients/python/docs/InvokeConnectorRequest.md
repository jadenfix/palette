# InvokeConnectorRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**arguments** | **object** | Arguments object matching the tool&#39;s input schema. | [optional]
**tool** | **str** | Tool slug to execute (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |

## Example

```python
from beater_client.models.invoke_connector_request import InvokeConnectorRequest

# TODO update the JSON string below
json = "{}"
# create an instance of InvokeConnectorRequest from a JSON string
invoke_connector_request_instance = InvokeConnectorRequest.from_json(json)
# print the JSON string representation of the object
print(InvokeConnectorRequest.to_json())

# convert the object into a dict
invoke_connector_request_dict = invoke_connector_request_instance.to_dict()
# create an instance of InvokeConnectorRequest from a dict
invoke_connector_request_from_dict = InvokeConnectorRequest.from_dict(invoke_connector_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
