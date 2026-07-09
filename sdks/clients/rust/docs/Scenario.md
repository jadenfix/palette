# Scenario

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **String** | When the scenario was created. |
**exemplar_trace_id** | **String** |  |
**expected_outcome** | Option<**String**> | Expected outcome for replay assertions, if known. | [optional]
**failure_mode** | [**models::FailureMode**](FailureMode.md) | The dominant failure mode this scenario reproduces. |
**perturbation_knobs** | [**models::PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |
**recurrence_count** | **i32** | How many traces exhibited this scenario. |
**redaction_class** | [**models::RedactionClass**](RedactionClass.md) | Redaction classification of the scenario payload. |
**scenario_id** | **String** | Stable, deterministic identifier for the scenario. |
**scope** | [**models::TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |
**source_trace_ids** | **Vec<String>** | Trace ids the scenario was mined from, sorted ascending. |
**title** | **String** | Human-readable title. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
