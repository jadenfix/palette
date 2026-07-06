# Beater.Client.Model.ConnectorTool
A single executable tool within a toolkit, carrying the metadata an agent needs to actually *call* it: the input JSON Schema, tags, and toolkit. This is the raw material for the prompting scaffold in [`crate::skill`].

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Description** | **string** | What the tool does. | [optional]
**InputSchema** | **Object** | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. | [optional]
**Name** | **string** | Human display name. |
**NoAuth** | **bool** | &#x60;true&#x60; when the tool executes without a connected account. | [optional]
**Slug** | **string** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |
**Tags** | **List&lt;string&gt;** | Free-form tags Composio assigns (categories, importance, …). | [optional]
**Toolkit** | **string** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
