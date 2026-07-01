

# ToolExecution

Result of executing a tool — Composio's `{successful, data, error}` envelope.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**data** | **Object** | Tool output payload (shape is tool-specific). |  [optional] |
|**error** | **String** | Error message when &#x60;successful&#x60; is false. |  [optional] |
|**logId** | **String** | Composio execution log id, for tracing. |  [optional] |
|**successful** | **Boolean** | Whether the tool reported success. |  |



