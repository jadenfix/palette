# ConnectionLink

One-time login link returned when initiating a managed-OAuth connection.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connected_account_id** | **str** | Composio connection id (&#x60;ca_…&#x60;) created for this handshake. | 
**expires_at** | **str** | When the link expires (RFC 3339), if provided. | [optional] 
**redirect_url** | **str** | URL the end user opens once to authorize the app. | 

## Example

```python
from beater_client.models.connection_link import ConnectionLink

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectionLink from a JSON string
connection_link_instance = ConnectionLink.from_json(json)
# print the JSON string representation of the object
print(ConnectionLink.to_json())

# convert the object into a dict
connection_link_dict = connection_link_instance.to_dict()
# create an instance of ConnectionLink from a dict
connection_link_from_dict = ConnectionLink.from_dict(connection_link_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


