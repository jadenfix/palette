
# ConnectorTool

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **name** | **kotlin.String** | Human display name. |  |
| **slug** | **kotlin.String** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |  |
| **description** | **kotlin.String** | What the tool does. |  [optional] |
| **inputSchema** | [**kotlin.Any**](.md) | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. |  [optional] |
| **noAuth** | **kotlin.Boolean** | &#x60;true&#x60; when the tool executes without a connected account. |  [optional] |
| **tags** | **kotlin.collections.List&lt;kotlin.String&gt;** | Free-form tags Composio assigns (categories, importance, …). |  [optional] |
| **toolkit** | **kotlin.String** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. |  [optional] |
