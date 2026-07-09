

# ConnectorTool

A single executable tool within a toolkit, carrying the metadata an agent needs to actually *call* it: the input JSON Schema, tags, and toolkit. This is the raw material for the prompting scaffold in [`crate::skill`].

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**description** | **String** | What the tool does. |  [optional] |
|**inputSchema** | **Object** | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. |  [optional] |
|**name** | **String** | Human display name. |  |
|**noAuth** | **Boolean** | &#x60;true&#x60; when the tool executes without a connected account. |  [optional] |
|**slug** | **String** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |  |
|**tags** | **List&lt;String&gt;** | Free-form tags Composio assigns (categories, importance, …). |  [optional] |
|**toolkit** | **String** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. |  [optional] |
