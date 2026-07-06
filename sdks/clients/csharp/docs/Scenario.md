# Beater.Client.Model.Scenario
A reusable failure scenario mined from production traces.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CreatedAt** | **DateTime** | When the scenario was created. |
**ExemplarTraceId** | **string** |  |
**ExpectedOutcome** | **string** | Expected outcome for replay assertions, if known. | [optional]
**FailureMode** | **FailureMode** | The dominant failure mode this scenario reproduces. |
**PerturbationKnobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |
**RecurrenceCount** | **int** | How many traces exhibited this scenario. |
**RedactionClass** | **RedactionClass** | Redaction classification of the scenario payload. |
**ScenarioId** | **string** | Stable, deterministic identifier for the scenario. |
**Scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |
**SourceTraceIds** | **List&lt;string&gt;** | Trace ids the scenario was mined from, sorted ascending. |
**Title** | **string** | Human-readable title. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
