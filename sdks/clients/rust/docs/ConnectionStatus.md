# ConnectionStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connected** | **bool** | `true` only when an account exists and is `ACTIVE`. | 
**connected_account_id** | Option<**String**> | The connected-account id, when one exists. | [optional]
**status** | **String** | Raw Composio status (`ACTIVE`, `INITIALIZING`, `FAILED`, …) or `not_connected` when no account exists yet. | 
**toolkit** | **String** | Toolkit slug this status is for. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


