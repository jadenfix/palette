# Beater.Client.Model.PerturbationKnobs
Tunable knobs describing how a scenario may be perturbed during replay.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AuthFailure** | **bool** | Force an auth failure on a dependency. |
**ContradictorySource** | **bool** | Inject a contradictory context source. |
**PromptInjection** | **bool** | Attempt a prompt-injection payload. |
**StaleSource** | **bool** | Serve a stale version of a context source. |
**Timeout** | **bool** | Force a timeout on a dependency. |
**ToolSchemaMismatch** | **bool** | Present a tool whose schema mismatches expectations. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
