# ConnectConnectorRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**toolkit** | **str** | Toolkit slug to connect (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;, &#x60;slack&#x60;). | 

## Example

```python
from beater_client.models.connect_connector_request import ConnectConnectorRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectConnectorRequest from a JSON string
connect_connector_request_instance = ConnectConnectorRequest.from_json(json)
# print the JSON string representation of the object
print(ConnectConnectorRequest.to_json())

# convert the object into a dict
connect_connector_request_dict = connect_connector_request_instance.to_dict()
# create an instance of ConnectConnectorRequest from a dict
connect_connector_request_from_dict = ConnectConnectorRequest.from_dict(connect_connector_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


