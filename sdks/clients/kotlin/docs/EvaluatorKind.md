
# EvaluatorKind

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **type** | [**inline**](#Type) |  |  |
| **pattern** | **kotlin.String** |  |  |
| **abs** | **kotlin.Double** |  |  |
| **rel** | **kotlin.Double** |  |  |
| **maxMicros** | **kotlin.Long** |  |  |
| **maxMs** | **kotlin.Long** |  |  |
| **model** | **kotlin.String** |  |  |
| **rubric** | **kotlin.String** |  |  |
| **maxSteps** | **kotlin.Long** |  |  |
| **minRatio** | **kotlin.Double** |  |  |
| **domContains** | **kotlin.String** |  |  [optional] |
| **urlContains** | **kotlin.String** |  |  [optional] |


<a id="Type"></a>
## Enum: type
| Name | Value |
| ---- | ----- |
| type | exact_match, regex_match, numeric_tolerance, json_object, cost_budget, latency_budget_ms, llm_judge, browser_task_success, browser_step_efficiency, browser_grounding, browser_recovery |
