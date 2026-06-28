

# EvaluatorKindOneOf7

Browser world-state success: asserts the final step's observed page (url and/or DOM) matches the configured target — NOT the agent's self-reported \"done\". Reads `trace.browser_steps`.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**domContains** | **String** |  |  [optional] |
|**type** | [**TypeEnum**](#TypeEnum) |  |  |
|**urlContains** | **String** |  |  [optional] |



## Enum: TypeEnum

| Name | Value |
|---- | -----|
| BROWSER_TASK_SUCCESS | &quot;browser_task_success&quot; |



