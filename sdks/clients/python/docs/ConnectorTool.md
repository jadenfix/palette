# ConnectorTool

A single executable tool within a toolkit, carrying the metadata an agent needs to actually *call* it: the input JSON Schema, tags, and toolkit. This is the raw material for the prompting scaffold in [`crate::skill`].

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | What the tool does. | [optional] 
**input_schema** | **object** | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. | [optional] 
**name** | **str** | Human display name. | 
**no_auth** | **bool** | &#x60;true&#x60; when the tool executes without a connected account. | [optional] 
**slug** | **str** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). | 
**tags** | **List[str]** | Free-form tags Composio assigns (categories, importance, …). | [optional] 
**toolkit** | **str** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. | [optional] 

## Example

```python
from beater_client.models.connector_tool import ConnectorTool

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectorTool from a JSON string
connector_tool_instance = ConnectorTool.from_json(json)
# print the JSON string representation of the object
print(ConnectorTool.to_json())

# convert the object into a dict
connector_tool_dict = connector_tool_instance.to_dict()
# create an instance of ConnectorTool from a dict
connector_tool_from_dict = ConnectorTool.from_dict(connector_tool_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


