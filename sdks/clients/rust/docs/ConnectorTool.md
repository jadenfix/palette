# ConnectorTool

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | Option<**String**> | What the tool does. | [optional]
**input_schema** | Option<[**serde_json::Value**](.md)> | JSON Schema of the tool's `arguments`, verbatim from Composio. The agent loop uses this to construct valid calls; [`crate::skill`] renders it. | [optional]
**name** | **String** | Human display name. |
**no_auth** | Option<**bool**> | `true` when the tool executes without a connected account. | [optional]
**slug** | **String** | Tool slug passed to [`ComposioClient::execute`] (e.g. `GITHUB_CREATE_AN_ISSUE`). |
**tags** | Option<**Vec<String>**> | Free-form tags Composio assigns (categories, importance, …). | [optional]
**toolkit** | Option<**String**> | Owning toolkit slug (e.g. `github`), when known. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
