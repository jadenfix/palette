

# ScenarioCluster

A cluster of failing traces that share a similar failure signature.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**dominantFailureMode** | **FailureMode** | The most common failure mode across members. |  |
|**exemplarTraceId** | **String** |  |  |
|**memberTraceIds** | **List&lt;String&gt;** | All member trace ids, sorted ascending. |  |
|**signature** | [**Signature**](Signature.md) | The signature of the cluster&#39;s exemplar. |  |
|**size** | **Integer** | Number of member traces. |  |



