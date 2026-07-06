# Beater.Client.Model.ConnectionStatus
Connection status of one app for one entity.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Connected** | **bool** | &#x60;true&#x60; only when an account exists and is &#x60;ACTIVE&#x60;. |
**ConnectedAccountId** | **string** | The connected-account id, when one exists. | [optional]
**Status** | **string** | Raw Composio status (&#x60;ACTIVE&#x60;, &#x60;INITIALIZING&#x60;, &#x60;FAILED&#x60;, …) or &#x60;not_connected&#x60; when no account exists yet. |
**Toolkit** | **string** | Toolkit slug this status is for. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
