

# Scenario

A reusable failure scenario mined from production traces.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**createdAt** | **OffsetDateTime** | When the scenario was created. |  |
|**exemplarTraceId** | **String** |  |  |
|**expectedOutcome** | **String** | Expected outcome for replay assertions, if known. |  [optional] |
|**failureMode** | **FailureMode** | The dominant failure mode this scenario reproduces. |  |
|**perturbationKnobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |  |
|**recurrenceCount** | **Integer** | How many traces exhibited this scenario. |  |
|**redactionClass** | **RedactionClass** | Redaction classification of the scenario payload. |  |
|**scenarioId** | **String** | Stable, deterministic identifier for the scenario. |  |
|**scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |  |
|**sourceTraceIds** | **List&lt;String&gt;** | Trace ids the scenario was mined from, sorted ascending. |  |
|**title** | **String** | Human-readable title. |  |
