# Beater.Client.Model.Toolkit
A connectable third-party app (Composio \"toolkit\"), flattened from the v3 `GET /toolkits` shape into the fields Beater exposes.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AuthSchemes** | **List&lt;string&gt;** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). | [optional]
**Description** | **string** | Short description, if the catalog provides one. | [optional]
**Name** | **string** | Human display name. |
**NoAuth** | **bool** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. | [optional]
**Slug** | **string** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). |
**ToolsCount** | **int?** | Number of tools the toolkit exposes, if known. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
