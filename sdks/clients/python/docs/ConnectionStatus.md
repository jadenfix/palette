# ConnectionStatus

Connection status of one app for one entity.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connected** | **bool** | &#x60;true&#x60; only when an account exists and is &#x60;ACTIVE&#x60;. | 
**connected_account_id** | **str** | The connected-account id, when one exists. | [optional] 
**status** | **str** | Raw Composio status (&#x60;ACTIVE&#x60;, &#x60;INITIALIZING&#x60;, &#x60;FAILED&#x60;, …) or &#x60;not_connected&#x60; when no account exists yet. | 
**toolkit** | **str** | Toolkit slug this status is for. | 

## Example

```python
from beater_client.models.connection_status import ConnectionStatus

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectionStatus from a JSON string
connection_status_instance = ConnectionStatus.from_json(json)
# print the JSON string representation of the object
print(ConnectionStatus.to_json())

# convert the object into a dict
connection_status_dict = connection_status_instance.to_dict()
# create an instance of ConnectionStatus from a dict
connection_status_from_dict = ConnectionStatus.from_dict(connection_status_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


