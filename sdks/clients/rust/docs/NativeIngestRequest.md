# NativeIngestRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  |
**auth_context** | Option<[**models::AuthContext**](AuthContext.md)> |  | [optional]
**cost** | Option<[**models::Money**](Money.md)> |  | [optional]
**end_time** | Option<**String**> |  | [optional]
**idempotency_key** | Option<**String**> |  | [optional]
**input** | Option<[**serde_json::Value**](.md)> |  | [optional]
**kind** | **String** | Canonical agent span kind such as agent.run or llm.call |
**model** | Option<[**models::ModelRef**](ModelRef.md)> |  | [optional]
**name** | **String** |  |
**output** | Option<[**serde_json::Value**](.md)> |  | [optional]
**parent_span_id** | Option<**String**> |  | [optional]
**redaction_class** | [**models::RedactionClass**](RedactionClass.md) |  |
**scope** | [**models::TenantScope**](TenantScope.md) |  |
**seq** | **i64** |  |
**span_id** | **String** |  |
**start_time** | Option<**String**> |  | [optional]
**status** | [**models::SpanStatus**](SpanStatus.md) |  |
**tokens** | Option<[**models::TokenCounts**](TokenCounts.md)> |  | [optional]
**trace_id** | **String** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
