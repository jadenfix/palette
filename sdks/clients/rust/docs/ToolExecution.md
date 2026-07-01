# ToolExecution

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**data** | Option<[**serde_json::Value**](.md)> | Tool output payload (shape is tool-specific). | [optional]
**error** | Option<**String**> | Error message when `successful` is false. | [optional]
**log_id** | Option<**String**> | Composio execution log id, for tracing. | [optional]
**successful** | **bool** | Whether the tool reported success. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


