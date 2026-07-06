# Beater.Client.Model.ScenarioCluster
A cluster of failing traces that share a similar failure signature.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DominantFailureMode** | **FailureMode** | The most common failure mode across members. |
**ExemplarTraceId** | **string** |  |
**MemberTraceIds** | **List&lt;string&gt;** | All member trace ids, sorted ascending. |
**Signature** | [**Signature**](Signature.md) | The signature of the cluster&#39;s exemplar. |
**Size** | **int** | Number of member traces. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
