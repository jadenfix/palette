# scenario_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **char \*** | When the scenario was created. |
**exemplar_trace_id** | **char \*** |  |
**expected_outcome** | **char \*** | Expected outcome for replay assertions, if known. | [optional]
**failure_mode** | **failure_mode_t \*** | The dominant failure mode this scenario reproduces. |
**perturbation_knobs** | [**perturbation_knobs_t**](perturbation_knobs.md) \* | Suggested perturbation knobs for replay. |
**recurrence_count** | **int** | How many traces exhibited this scenario. |
**redaction_class** | **redaction_class_t \*** | Redaction classification of the scenario payload. |
**scenario_id** | **char \*** | Stable, deterministic identifier for the scenario. |
**scope** | [**tenant_scope_t**](tenant_scope.md) \* | Tenant/project/environment scope this scenario belongs to. |
**source_trace_ids** | **list_t \*** | Trace ids the scenario was mined from, sorted ascending. |
**title** | **char \*** | Human-readable title. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
