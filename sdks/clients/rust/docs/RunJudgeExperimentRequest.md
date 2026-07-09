# RunJudgeExperimentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_outputs** | [**Vec<models::CaseOutputOverrideRequest>**](CaseOutputOverrideRequest.md) |  |
**baseline_release_id** | **String** |  |
**candidate_outputs** | [**Vec<models::CaseOutputOverrideRequest>**](CaseOutputOverrideRequest.md) |  |
**candidate_release_id** | **String** |  |
**evaluator_id** | **String** |  |
**evaluator_version_id** | **String** |  |
**gate_policy** | Option<[**models::GatePolicy**](GatePolicy.md)> |  | [optional]
**kind** | [**models::EvaluatorKind**](EvaluatorKind.md) |  |
**provider_secret_id** | **String** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
