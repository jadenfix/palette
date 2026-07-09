# connector_tool_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **char \*** | What the tool does. | [optional]
**input_schema** | [**object_t**](.md) \* | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. | [optional]
**name** | **char \*** | Human display name. |
**no_auth** | **int** | &#x60;true&#x60; when the tool executes without a connected account. | [optional]
**slug** | **char \*** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |
**tags** | **list_t \*** | Free-form tags Composio assigns (categories, importance, …). | [optional]
**toolkit** | **char \*** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
