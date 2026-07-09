# EvalResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cost** | Option<[**models::Money**](Money.md)> |  | [optional]
**created_at** | **String** |  |
**eval_result_id** | **String** |  |
**evidence** | Option<[**serde_json::Value**](.md)> |  |
**label** | Option<**String**> |  | [optional]
**non_reproducible_reason** | Option<**String**> |  | [optional]
**project_id** | **String** |  |
**reproducibility** | [**models::EvalReproducibility**](EvalReproducibility.md) |  |
**score** | **f64** |  |
**span_id** | Option<**String**> |  | [optional]
**tenant_id** | **String** |  |
**tokens** | Option<[**models::TokenCounts**](TokenCounts.md)> |  | [optional]
**trace_id** | **String** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
