

# PerturbationKnobs

Tunable knobs describing how a scenario may be perturbed during replay.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**authFailure** | **Boolean** | Force an auth failure on a dependency. |  |
|**contradictorySource** | **Boolean** | Inject a contradictory context source. |  |
|**promptInjection** | **Boolean** | Attempt a prompt-injection payload. |  |
|**staleSource** | **Boolean** | Serve a stale version of a context source. |  |
|**timeout** | **Boolean** | Force a timeout on a dependency. |  |
|**toolSchemaMismatch** | **Boolean** | Present a tool whose schema mismatches expectations. |  |



