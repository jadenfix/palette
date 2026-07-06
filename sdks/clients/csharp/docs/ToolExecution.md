# Beater.Client.Model.ToolExecution
Result of executing a tool — Composio's `{successful, data, error}` envelope.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Data** | **Object** | Tool output payload (shape is tool-specific). | [optional]
**Error** | **string** | Error message when &#x60;successful&#x60; is false. | [optional]
**LogId** | **string** | Composio execution log id, for tracing. | [optional]
**Successful** | **bool** | Whether the tool reported success. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
