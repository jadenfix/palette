
# Scenario

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **createdAt** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) | When the scenario was created. |  |
| **exemplarTraceId** | **kotlin.String** |  |  |
| **failureMode** | [**FailureMode**](FailureMode.md) | The dominant failure mode this scenario reproduces. |  |
| **perturbationKnobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |  |
| **recurrenceCount** | **kotlin.Int** | How many traces exhibited this scenario. |  |
| **redactionClass** | [**RedactionClass**](RedactionClass.md) | Redaction classification of the scenario payload. |  |
| **scenarioId** | **kotlin.String** | Stable, deterministic identifier for the scenario. |  |
| **scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |  |
| **sourceTraceIds** | **kotlin.collections.List&lt;kotlin.String&gt;** | Trace ids the scenario was mined from, sorted ascending. |  |
| **title** | **kotlin.String** | Human-readable title. |  |
| **expectedOutcome** | **kotlin.String** | Expected outcome for replay assertions, if known. |  [optional] |
