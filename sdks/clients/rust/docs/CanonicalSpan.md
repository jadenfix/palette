# CanonicalSpan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  |
**cost** | Option<[**models::Money**](Money.md)> |  | [optional]
**end_time** | Option<**String**> |  | [optional]
**environment_id** | **String** |  |
**input_ref** | Option<[**models::ArtifactRef**](ArtifactRef.md)> |  | [optional]
**kind** | **String** | Canonical agent span kind such as agent.run or llm.call |
**model** | Option<[**models::ModelRef**](ModelRef.md)> |  | [optional]
**name** | **String** |  |
**normalizer_version** | **String** |  |
**output_ref** | Option<[**models::ArtifactRef**](ArtifactRef.md)> |  | [optional]
**parent_span_id** | Option<**String**> |  | [optional]
**project_id** | **String** |  |
**raw_ref** | [**models::ArtifactRef**](ArtifactRef.md) |  |
**schema_version** | **i32** |  |
**seq** | **i64** |  |
**span_id** | **String** |  |
**start_time** | **String** |  |
**status** | [**models::SpanStatus**](SpanStatus.md) |  |
**tenant_id** | **String** |  |
**tokens** | Option<[**models::TokenCounts**](TokenCounts.md)> |  | [optional]
**trace_id** | **String** |  |
**unmapped_attrs** | Option<[**serde_json::Value**](.md)> |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
